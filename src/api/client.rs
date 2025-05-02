use serde::{Deserialize, Serialize};
use reqwest::Client;
use thiserror::Error;

/// Error type for all Odido API operations.
#[derive(Debug, Error)]
pub enum OdidoError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    Api(String),
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

}

/// Main Odido API client.
#[derive(Clone, Debug)]
pub struct OdidoClient {
    client: Client,
    pub token: String,
    pub base_url: String,
}

impl OdidoClient {
    /// Create a new OdidoClient with the provided bearer token.
    pub fn new(token: String) -> Self {
        Self {
            client: Client::builder().cookie_store(true).build().unwrap(),
            token,
            base_url: "https://capi.odido.nl/c88084b603f5".to_string(),
        }
    }

    /// Get all linked subscriptions for the user.
    pub async fn get_linked_subscriptions(&self) -> Result<LinkedSubscriptions, OdidoError> {
        let resp = self
            .client
            .get(format!("{}/linkedsubscriptions", self.base_url))
            .headers(self.auth_headers())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(OdidoError::Api(format!("linkedsubscriptions: {}", resp.status())));
        }
        let linked: LinkedSubscriptions = resp.json().await?;
        Ok(linked)
    }

    /// Get all roaming bundles for a subscription.
    pub async fn get_roaming_bundles(&self, subscription_url: &str) -> Result<RoamingBundles, OdidoError> {
        let resp = self
            .client
            .get(format!("{}/roamingbundles", subscription_url))
            .headers(self.auth_headers())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(OdidoError::Api(format!("roamingbundles: {}", resp.status())));
        }
        let bundles: RoamingBundles = resp.json().await?;
        Ok(bundles)
    }

    /// Top up a bundle for a subscription.
    pub async fn top_up(&self, subscription_url: &str, bundle_buying_code: &str) -> Result<(), OdidoError> {
        let data = serde_json::json!({"Bundles": [{"BuyingCode": bundle_buying_code}]});
        let resp = self
            .client
            .post(format!("{}/roamingbundles", subscription_url))
            .headers(self.auth_headers())
            .json(&data)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(OdidoError::Api(format!("topup: {}", resp.status())));
        }
        Ok(())
    }

    /// Build the required auth headers for all API calls.
    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT, ACCEPT};
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", mask_secret(&self.token))).unwrap(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    #[serde(rename = "SubscriptionURL")]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkedSubscriptions {
    pub subscriptions: Vec<Subscription>,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Remaining {
    #[serde(rename = "Value")]
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bundle {
    #[serde(rename = "ZoneColor")]
    pub zone_color: String,
    #[serde(rename = "Remaining")]
    pub remaining: Remaining,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoamingBundles {
    #[serde(rename = "Bundles")]
    pub bundles: Vec<Bundle>,
}
