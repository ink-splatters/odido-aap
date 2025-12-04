use anyhow::{Context, Result, anyhow};
use clap::{ArgAction::Count, Parser, Subcommand};
use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{error, info, trace};

mod cache;
mod log;

use cache::{CacheBackend, CacheKey, CacheManager, CacheTtl, SqliteBackend};

/* ───────── CLI ───────── */
#[derive(Parser, Debug)]
#[command(author, version, about = "Odido API client for data bundle management")]
struct Cli {
    #[arg(short, long, action = Count, global = true)]
    verbose: u8,

    #[arg(long, global = true)]
    wire: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check data usage and top-up if needed (default command)
    Check {
        #[arg(long, env = "ODIDO_THRESHOLD", default_value_t = 1_500)]
        threshold: u32,
        #[arg(short = 't', long, env = "ODIDO_TOKEN")]
        token: String,
        #[arg(short = 'u', long, env = "ODIDO_USER_ID")]
        user_id: String,
        #[arg(long, env = "ODIDO_TIMEOUT", default_value = "30")]
        timeout: u64,
        /// Force refresh cache (bypass cache, fetch from API)
        #[arg(long)]
        refresh: bool,
        /// Disable caching (always fetch from API)
        #[arg(long)]
        no_cache: bool,
    },
    /// Cache management commands
    Cache {
        #[command(subcommand)]
        action: CacheCommand,
    },
    /// Reset everything (cache and credentials)
    Purge {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum CacheCommand {
    /// Show cache statistics
    Stats,
    /// Remove only expired entries
    Clean,
    /// Clear entire cache
    Clear,
}

/* ───────── JSON models ───────── */
#[derive(Deserialize, Serialize, Debug, Clone)]
struct LinkedSubscriptions {
    #[serde(rename = "subscriptions")]
    subs: Vec<Subscription>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Subscription {
    #[serde(rename = "SubscriptionURL")]
    url: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct BundleList {
    #[serde(rename = "Bundles")]
    bundles: Vec<Bundle>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Bundle {
    #[serde(rename = "ZoneColor")]
    zone_color: String,
    #[serde(rename = "Remaining")]
    remaining: Remaining,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Remaining {
    #[serde(rename = "Value")]
    value: u32, // kB
}

/* ───────── main ───────── */
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.wire)?;

    match cli.command {
        Some(Command::Check { threshold, token, user_id, timeout, refresh, no_cache }) => {
            let client = build_client(timeout)?;
            let cache = init_cache(false).await?;
            check_and_topup(&client, &token, &user_id, threshold, &cache, refresh || no_cache).await?;
        }
        Some(Command::Cache { action }) => {
            run_cache_command(action).await?;
        }
        Some(Command::Purge { force }) => {
            run_purge(force).await?;
        }
        None => {
            // No subcommand: show help
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
    }
    Ok(())
}

/* ───────── paths ───────── */
fn cache_dir() -> Result<PathBuf> {
    Ok(dirs::cache_dir()
        .ok_or_else(|| anyhow!("Could not determine cache directory"))?
        .join("odido"))
}

fn data_dir() -> Result<PathBuf> {
    Ok(dirs::data_dir()
        .ok_or_else(|| anyhow!("Could not determine data directory"))?
        .join("odido"))
}

/* ───────── cache initialization ───────── */
async fn init_cache(cleanup: bool) -> Result<CacheManager<SqliteBackend>> {
    let db_path = cache_dir()?.join("cache.db");
    let backend = SqliteBackend::new(db_path).await?;

    if cleanup {
        let cleaned = backend.cleanup_expired().await?;
        if cleaned > 0 {
            info!("Cleaned up {} expired cache entries", cleaned);
        }
    }

    Ok(CacheManager::new(backend))
}

/* ───────── cache commands ───────── */
async fn run_cache_command(action: CacheCommand) -> Result<()> {
    let db_path = cache_dir()?.join("cache.db");

    if !db_path.exists() {
        println!("Cache database does not exist yet.");
        return Ok(());
    }

    let backend = SqliteBackend::new(db_path).await?;

    match action {
        CacheCommand::Stats => {
            let stats = backend.stats().await?;
            println!("Cache Statistics:");
            println!("  Entries: {}", stats.total_entries);
            println!("  Size:    {} bytes", stats.total_size_bytes);
        }
        CacheCommand::Clean => {
            let cleaned = backend.cleanup_expired().await?;
            println!("Removed {} expired entries", cleaned);
        }
        CacheCommand::Clear => {
            let manager = CacheManager::new(backend);
            manager.clear().await?;
            println!("Cache cleared");
        }
    }
    Ok(())
}

/* ───────── purge command ───────── */
async fn run_purge(force: bool) -> Result<()> {
    if !force {
        println!("This will delete all cached data and stored credentials.");
        println!("Run with --force to confirm.");
        return Ok(());
    }

    let mut removed = Vec::new();

    // Remove cache directory
    let cache = cache_dir()?;
    if cache.exists() {
        std::fs::remove_dir_all(&cache)?;
        removed.push(format!("cache: {}", cache.display()));
    }

    // Remove data directory (future: credentials)
    let data = data_dir()?;
    if data.exists() {
        std::fs::remove_dir_all(&data)?;
        removed.push(format!("data: {}", data.display()));
    }

    if removed.is_empty() {
        println!("Nothing to remove.");
    } else {
        println!("Removed:");
        for item in removed {
            println!("  {}", item);
        }
    }
    Ok(())
}

/* ───────── tracing setup ───────── */
fn init_tracing(verbosity: u8, wire: bool) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let lvl = match verbosity {
        0 => "warn", // our pretty println! handles user output
        1 => "debug",
        _ => "trace",
    };

    let mut filter = EnvFilter::builder()
        .with_default_directive(lvl.parse()?)
        .from_env_lossy();

    if wire {
        for m in ["reqwest", "hyper", "h2", "hyper::client"] {
            filter = filter.add_directive(format!("{m}=trace").parse()?);
        }
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().compact())
        .init();
    Ok(())
}

/* ───────── reqwest client ───────── */
fn build_client(timeout: u64) -> Result<Client> {
    let mut h = header::HeaderMap::new();
    h.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("T-Mobile 5.3.28 (Android 10; 10)"),
    );
    h.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json"),
    );

    Ok(Client::builder()
        .default_headers(h)
        .timeout(Duration::from_secs(timeout))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(8)
        .http2_prior_knowledge()
        .build()?)
}

/* ───────── business logic ───────── */
async fn check_and_topup(
    client: &Client,
    token: &str,
    user_id: &str,
    threshold: u32,
    cache: &CacheManager<SqliteBackend>,
    bypass_cache: bool,
) -> Result<()> {
    let bearer = format!("Bearer {}", token);

    let subs = linked_subscriptions(
        client,
        user_id,
        &bearer,
        cache,
        bypass_cache,
    )
    .await?;
    let first = subs
        .subs
        .first()
        .ok_or_else(|| anyhow!("no subscription returned"))?;

    let bundles = roaming_bundles(
        client,
        &bearer,
        &first.url,
        cache,
        user_id,
        bypass_cache,
    )
    .await?;

    let remaining_kb: u64 = bundles
        .bundles
        .iter()
        .filter(|b| b.zone_color == "NL")
        .map(|b| b.remaining.value as u64)
        .sum();
    let remaining_mb = (remaining_kb / 1024) as u32;

    info!(threshold, remaining_mb, "quota status");

    if remaining_mb < threshold {
        top_up(client, &bearer, &first.url, cache, user_id).await?;
        println!(
            "{} ✔  2000 MB bundle purchased",
            chrono::Local::now().format("[%H:%M:%S]")
        );
    } else {
        println!(
            "{} Nothing to do, {} MB still available (≥ threshold)",
            chrono::Local::now().format("[%H:%M:%S]"),
            remaining_mb
        );
    }
    Ok(())
}

/* ───────── helpers ───────── */
async fn linked_subscriptions(
    client: &Client,
    user_id: &str,
    bearer: &str,
    cache: &CacheManager<SqliteBackend>,
    bypass_cache: bool,
) -> Result<LinkedSubscriptions> {
    let cache_key = CacheKey::linked_subscriptions(user_id);

    if bypass_cache {
        trace!("Cache bypassed for: {}", cache_key);
        return fetch_linked_subscriptions(client, user_id, bearer).await;
    }

    cache
        .get_or_fetch(&cache_key, CacheTtl::LINKED_SUBSCRIPTIONS, || {
            fetch_linked_subscriptions(client, user_id, bearer)
        })
        .await
}

async fn fetch_linked_subscriptions(
    client: &Client,
    user_id: &str,
    bearer: &str,
) -> Result<LinkedSubscriptions> {
    let url = format!("https://capi.odido.nl/{}/linkedsubscriptions", &user_id);
    log::outbound("GET", &url);
    let start = Instant::now();

    let res = client
        .get(&url)
        .header(header::AUTHORIZATION, bearer)
        .send()
        .await
        .context("GET linkedsubscriptions")?;

    let status = res.status();
    let bytes = res.content_length().unwrap_or(0) as usize;
    let res = check_status(res).await?;
    let body = res.json::<LinkedSubscriptions>().await?;

    log::inbound(status.as_u16(), &url, bytes, start.elapsed());
    trace!(?body);
    Ok(body)
}

async fn roaming_bundles(
    client: &Client,
    bearer: &str,
    subs_url: &str,
    cache: &CacheManager<SqliteBackend>,
    user_id: &str,
    bypass_cache: bool,
) -> Result<BundleList> {
    let msisdn = extract_msisdn(subs_url)?;
    let cache_key = CacheKey::bundles(user_id, msisdn);

    if bypass_cache {
        trace!("Cache bypassed for: {}", cache_key);
        return fetch_roaming_bundles(client, bearer, subs_url).await;
    }

    cache
        .get_or_fetch(&cache_key, CacheTtl::ROAMING_BUNDLES, || {
            fetch_roaming_bundles(client, bearer, subs_url)
        })
        .await
}

async fn fetch_roaming_bundles(
    client: &Client,
    bearer: &str,
    subs_url: &str,
) -> Result<BundleList> {
    let url = format!("{subs_url}/roamingbundles");
    log::outbound("GET", &url);
    let start = Instant::now();

    let res = client
        .get(&url)
        .header(header::AUTHORIZATION, bearer)
        .send()
        .await
        .context("GET roamingbundles")?;

    let status = res.status();
    let bytes = res.content_length().unwrap_or(0) as usize;
    let res = check_status(res).await?;
    let body = res.json::<BundleList>().await?;

    log::inbound(status.as_u16(), &url, bytes, start.elapsed());
    trace!(?body);
    Ok(body)
}

async fn top_up(
    client: &Client,
    bearer: &str,
    subs_url: &str,
    cache: &CacheManager<SqliteBackend>,
    user_id: &str,
) -> Result<()> {
    let url = format!("{subs_url}/roamingbundles");
    let payload = &serde_json::json!({ "Bundles": [{ "BuyingCode": "A0DAY01" }] });

    log::outbound("POST", &url);
    let start = Instant::now();

    let res = client
        .post(&url)
        .header(header::AUTHORIZATION, bearer)
        .json(payload)
        .send()
        .await
        .context("POST top-up")?;

    let status = res.status();
    let bytes = res.content_length().unwrap_or(0) as usize;
    check_status(res).await?;

    log::inbound(status.as_u16(), &url, bytes, start.elapsed());

    // Invalidate bundle cache after purchase
    let msisdn = extract_msisdn(subs_url)?;
    let cache_key = CacheKey::bundles(user_id, msisdn);
    cache.delete(&cache_key).await?;
    trace!("Invalidated cache after top-up: {}", cache_key);

    Ok(())
}

/* ───────── URL helpers ───────── */
/// Extract MSISDN from subscription URL (last path segment)
fn extract_msisdn(subs_url: &str) -> Result<&str> {
    subs_url
        .split('/')
        .last()
        .ok_or_else(|| anyhow!("Invalid subscription URL: missing MSISDN"))
}

/* ───────── status helper ───────── */
async fn check_status(res: reqwest::Response) -> Result<reqwest::Response> {
    let status = res.status();
    if !(status == StatusCode::OK || status == StatusCode::ACCEPTED) {
        let text = res.text().await.unwrap_or_default();
        error!(status = %status, body = %text, "HTTP error");
        return Err(anyhow!("HTTP {} – {}", status, text));
    }
    Ok(res)
}
