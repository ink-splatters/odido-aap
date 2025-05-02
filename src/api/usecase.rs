use crate::api::client::{LinkedSubscriptions, RoamingBundles};
use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::env;

#[derive(Debug)]
pub struct AutoTopUpConfig {
    pub token: String,
    pub threshold: u64, // MB
    pub bundle_buying_code: &'static str,
}

#[derive(Debug)]
pub enum AutoTopUpResult {
    NoTopUpNeeded { remaining_mb: u64 },
    ToppedUp,
}

#[derive(Debug)]
pub enum AutoTopUpError {
    Http(reqwest::Error),
    Api(String),
    Config(String),
    Deserialize(serde_json::Error),
}

impl fmt::Display for AutoTopUpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AutoTopUpError::Http(e) => write!(f, "HTTP error: {}", e),
            AutoTopUpError::Api(e) => write!(f, "API error: {}", e),
            AutoTopUpError::Config(e) => write!(f, "Config error: {}", e),
            AutoTopUpError::Deserialize(e) => write!(f, "Deserialize error: {}", e),
        }
    }
}

impl Error for AutoTopUpError {}

impl From<reqwest::Error> for AutoTopUpError {
    fn from(e: reqwest::Error) -> Self {
        AutoTopUpError::Http(e)
    }
}
impl From<serde_json::Error> for AutoTopUpError {
    fn from(e: serde_json::Error) -> Self {
        AutoTopUpError::Deserialize(e)
    }
}

pub struct AutoTopUp<'a> {
    pub client: &'a Client,
    pub config: AutoTopUpConfig,
}

impl<'a> AutoTopUp<'a> {
    async fn get_subscription_url(&self, headers: &reqwest::header::HeaderMap) -> Result<String, AutoTopUpError> {
        let resp = self
            .client
            .get("https://capi.odido.nl/c88084b603f5/linkedsubscriptions")
            .headers(headers.clone())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AutoTopUpError::Api(format!("linkedsubscriptions: {}", resp.status())));
        }
        let linked: LinkedSubscriptions = resp.json().await?;
        let subscription_url = linked
            .subscriptions
            .get(0)
            .ok_or_else(|| AutoTopUpError::Api("No subscriptions found".to_string()))?
            .url
            .clone();
        Ok(subscription_url)
    }

    async fn get_total_remaining_mb(&self, subscription_url: &str, headers: &reqwest::header::HeaderMap) -> Result<u64, AutoTopUpError> {
        let resp = self
            .client
            .get(format!("{}/roamingbundles", subscription_url))
            .headers(headers.clone())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AutoTopUpError::Api(format!("roamingbundles: {}", resp.status())));
        }
        let bundles: RoamingBundles = resp.json().await?;
        let total_remaining_kb: u64 = bundles.Bundles.iter()
            .filter(|b| b.ZoneColor == "NL")
            .map(|b| b.Remaining.Value)
            .sum();
        Ok(total_remaining_kb / 1024)
    }

    /// Only checks and returns remaining MB (never triggers top-up)
    pub async fn status(&self) -> Result<u64, AutoTopUpError> {
        let headers = self.auth_headers();
        let subscription_url = self.get_subscription_url(&headers).await?;
        self.get_total_remaining_mb(&subscription_url, &headers).await
    }

    /// Always triggers a top-up, regardless of remaining MB
    pub async fn aanvullen(&self) -> Result<(), AutoTopUpError> {
        let headers = self.auth_headers();
        let subscription_url = self.get_subscription_url(&headers).await?;
        let data = serde_json::json!({"Bundles": [{"BuyingCode": self.config.bundle_buying_code}]});
        let resp = self
            .client
            .post(format!("{}/roamingbundles", subscription_url))
            .headers(headers)
            .json(&data)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AutoTopUpError::Api(format!("topup: {}", resp.status())));
        }
        Ok(())
    }

    pub async fn execute(&self) -> Result<AutoTopUpResult, AutoTopUpError> {
        let headers = self.auth_headers();
        let subscription_url = self.get_subscription_url(&headers).await?;
        let total_remaining_mb = self.get_total_remaining_mb(&subscription_url, &headers).await?;
        // 3. Decide if top-up is needed
        if total_remaining_mb < self.config.threshold {
            // 4. Top up
            let data = serde_json::json!({"Bundles": [{"BuyingCode": self.config.bundle_buying_code}]});
            let resp = self
                .client
                .post(format!("{}/roamingbundles", subscription_url))
                .headers(headers)
                .json(&data)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(AutoTopUpError::Api(format!("topup: {}", resp.status())));
            }
            Ok(AutoTopUpResult::ToppedUp)
        } else {
            Ok(AutoTopUpResult::NoTopUpNeeded { remaining_mb: total_remaining_mb })
        }
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT, ACCEPT};
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", mask_secret(&self.config.token))).unwrap(),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("T-Mobile 5.3.28 (Android 10; 10)"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers
    }
}

fn mask_secret(secret: &str) -> String {
    if secret.len() <= 6 {
        "******".to_string()
    } else {
        format!("{}******{}", &secret[..3], &secret[secret.len()-3..])
    }
}
