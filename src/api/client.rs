use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
pub enum OdidoApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    Api(String),
    #[error("Unexpected response structure")]
    UnexpectedResponse,
}

#[derive(Clone)]
pub struct OdidoApi {
    client: Arc<Client>,
    base_url: String,
} 

impl OdidoApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Arc::new(Client::builder().cookie_store(true).build().unwrap()),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    // Add more advanced, composable methods here as needed.
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
    pub Value: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bundle {
    pub ZoneColor: String,
    pub Remaining: Remaining,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoamingBundles {
    pub Bundles: Vec<Bundle>,
}
