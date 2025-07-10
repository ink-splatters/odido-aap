use anyhow::{anyhow, Context, Result};
use clap::{ArgAction::Count, Parser};
use reqwest::{header, Client, StatusCode};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tracing::{error, info, trace};

mod log; // our pretty one-liners

/// Data-top-up helper for Odido (T-Mobile NL).
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Verbosity: -v = INFO, -vv = DEBUG, -vvv = TRACE
    #[arg(short, long, action = Count, global = true)]
    verbose: u8,

    /// Minimum MB that must remain, otherwise we top-up.
    #[arg(short, long, env = "ODIDO_THRESHOLD", default_value_t = 1_500)]
    threshold: u32,

    /// Bearer token (env ODIDO_TOKEN).
    #[arg(short = 't', long, env = "ODIDO_TOKEN")]
    token: String,

    /// Noisy wire-level traces.
    #[arg(long)]
    wire: bool,
}

/* ───────── models coming from the JSON API ───────── */

#[derive(Deserialize, Debug)]
struct LinkedSubscriptions {
    #[serde(rename = "subscriptions")]
    subs: Vec<Subscription>,
}

#[derive(Deserialize, Debug)]
struct Subscription {
    #[serde(rename = "SubscriptionURL")]
    url: String,
}

#[derive(Deserialize, Debug)]
struct BundleList {
    #[serde(rename = "Bundles")]
    bundles: Vec<Bundle>,
}

#[derive(Deserialize, Debug)]
struct Bundle {
    #[serde(rename = "ZoneColor")]
    zone_color: String,
    #[serde(rename = "Remaining")]
    remaining: Remaining,
}

#[derive(Deserialize, Debug)]
struct Remaining {
    #[serde(rename = "Value")]
    value: u32, // kB
}

/* ───────── main ───────── */

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.wire)?;
    let client = build_client()?;
    process(&client, &cli).await?;
    Ok(())
}

/* ───────── tracing setup ───────── */

fn init_tracing(verbosity: u8, wire: bool) -> Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let lvl = match verbosity {
        0 => "info",
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

    let fmt_layer = fmt::layer().compact().with_target(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

/* ───────── reqwest client ───────── */

fn build_client() -> Result<Client> {
    let mut h = header::HeaderMap::new();
    h.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("T-Mobile 5.3.28 (Android 10; 10)"),
    );
    h.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));

    Ok(Client::builder()
        .default_headers(h)
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(8)
        .http2_prior_knowledge() // OK for capi.odido.nl
        .build()?)
}

/* ───────── business logic ───────── */

async fn process(client: &Client, cli: &Cli) -> Result<()> {
    let bearer = format!("Bearer {}", cli.token);

    let subs = linked_subscriptions(client, &bearer).await?;
    let first = subs
        .subs
        .first()
        .ok_or_else(|| anyhow!("no subscription returned"))?;

    let bundles = roaming_bundles(client, &bearer, &first.url).await?;

    let remaining_kb: u64 = bundles
        .bundles
        .iter()
        .filter(|b| b.zone_color == "NL")
        .map(|b| b.remaining.value as u64)
        .sum();

    let remaining_mb = (remaining_kb / 1024) as u32;

    info!(threshold = cli.threshold, remaining_mb, "quota status");

    if remaining_mb < cli.threshold {
        top_up(client, &bearer, &first.url).await?;
        info!("✅  2000 MB bundle purchased");
    } else {
        info!("Nothing to do, {remaining_mb} MB still available (≥ threshold)");
    }
    Ok(())
}

/* ───────── helpers with pretty logging ───────── */

async fn linked_subscriptions(client: &Client, bearer: &str) -> Result<LinkedSubscriptions> {
    let url = "https://capi.odido.nl/c88084b603f5/linkedsubscriptions";
    log::outbound("GET", url);
    let started = Instant::now();

    let res = client
        .get(url)
        .header(header::AUTHORIZATION, bearer)
        .send()
        .await
        .context("GET linkedsubscriptions")?;

    let status = res.status();
    let bytes = res.content_length().unwrap_or(0) as usize;
    let res = check_status(res).await?;
    let body = res.json::<LinkedSubscriptions>().await?;

    log::inbound(status.as_u16(), bytes, started.elapsed());
    trace!(?body);
    Ok(body)
}

async fn roaming_bundles(client: &Client, bearer: &str, subs_url: &str) -> Result<BundleList> {
    let url = format!("{subs_url}/roamingbundles");
    log::outbound("GET", &url);
    let started = Instant::now();

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

    log::inbound(status.as_u16(), bytes, started.elapsed());
    trace!(?body);
    Ok(body)
}

async fn top_up(client: &Client, bearer: &str, subs_url: &str) -> Result<()> {
    let url = format!("{subs_url}/roamingbundles");
    let payload = &serde_json::json!({ "Bundles": [{ "BuyingCode": "A0DAY01" }] });

    log::outbound("POST", &url);
    let started = Instant::now();

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

    log::inbound(status.as_u16(), bytes, started.elapsed());
    Ok(())
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