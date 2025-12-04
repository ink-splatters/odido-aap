use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

mod manager;
mod sqlite;

pub use manager::{CacheManager, CacheTtl};
pub use sqlite::SqliteBackend;

/// Cache backend trait for pluggable storage implementations
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get cached value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set cached value with TTL
    async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()>;

    /// Delete cached value by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Clear all cached entries
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
}

/// Cache key builder following {context}:{resource}:{id} format
pub struct CacheKey;

impl CacheKey {
    /// Build key for user-specific resource
    pub fn user_resource(user_id: &str, resource: &str, id: &str) -> String {
        format!("user:{}:{}:{}", user_id, resource, id)
    }

    /// Build key for global resource
    pub fn global(resource: &str) -> String {
        format!("global:{}", resource)
    }

    /// Build key for subscription data
    pub fn subscription(user_id: &str, msisdn: &str) -> String {
        Self::user_resource(user_id, "subscription", msisdn)
    }

    /// Build key for bundle data
    pub fn bundles(user_id: &str, msisdn: &str) -> String {
        Self::user_resource(user_id, "bundles", msisdn)
    }

    /// Build key for linked subscriptions
    pub fn linked_subscriptions(user_id: &str) -> String {
        format!("user:{}:linkedsubscriptions", user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        assert_eq!(
            CacheKey::subscription("123", "456"),
            "user:123:subscription:456"
        );
        assert_eq!(
            CacheKey::bundles("123", "456"),
            "user:123:bundles:456"
        );
        assert_eq!(
            CacheKey::linked_subscriptions("123"),
            "user:123:linkedsubscriptions"
        );
        assert_eq!(
            CacheKey::global("countries"),
            "global:countries"
        );
    }
}
