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
    /// Build key for bundle data
    pub fn bundles(user_id: &str, msisdn: &str) -> String {
        format!("user:{}:bundles:{}", user_id, msisdn)
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
        assert_eq!(CacheKey::bundles("123", "456"), "user:123:bundles:456");
        assert_eq!(
            CacheKey::linked_subscriptions("123"),
            "user:123:linkedsubscriptions"
        );
    }

    #[test]
    fn test_cache_key_with_realistic_values() {
        assert_eq!(
            CacheKey::bundles("user-abc-123", "0612345678"),
            "user:user-abc-123:bundles:0612345678"
        );
        assert_eq!(
            CacheKey::linked_subscriptions("user-abc-123"),
            "user:user-abc-123:linkedsubscriptions"
        );
    }

    #[test]
    fn test_cache_key_with_empty_strings() {
        // Empty strings should still produce valid keys (caller's responsibility to validate)
        assert_eq!(CacheKey::bundles("", ""), "user::bundles:");
        assert_eq!(
            CacheKey::linked_subscriptions(""),
            "user::linkedsubscriptions"
        );
    }

    #[test]
    fn test_cache_key_with_special_characters() {
        // Keys should handle special characters (they're just strings)
        assert_eq!(
            CacheKey::bundles("user/with/slashes", "phone:number"),
            "user:user/with/slashes:bundles:phone:number"
        );
    }

    #[test]
    fn test_cache_key_with_unicode() {
        assert_eq!(CacheKey::bundles("用户", "電話"), "user:用户:bundles:電話");
        assert_eq!(
            CacheKey::linked_subscriptions("émoji🎉"),
            "user:émoji🎉:linkedsubscriptions"
        );
    }

    #[test]
    fn test_cache_key_uniqueness() {
        // Different inputs should produce different keys
        let key1 = CacheKey::bundles("user1", "phone1");
        let key2 = CacheKey::bundles("user1", "phone2");
        let key3 = CacheKey::bundles("user2", "phone1");
        let key4 = CacheKey::linked_subscriptions("user1");

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_cache_key_with_long_strings() {
        let long_user_id = "a".repeat(1000);
        let long_msisdn = "0".repeat(1000);

        let key = CacheKey::bundles(&long_user_id, &long_msisdn);
        assert!(key.starts_with("user:"));
        assert!(key.contains(":bundles:"));
        assert_eq!(key.len(), 5 + 1000 + 9 + 1000); // "user:" + user_id + ":bundles:" + msisdn
    }

    #[test]
    fn test_cache_key_deterministic() {
        // Same inputs should always produce the same key
        for _ in 0..100 {
            assert_eq!(
                CacheKey::bundles("test_user", "test_phone"),
                "user:test_user:bundles:test_phone"
            );
        }
    }
}
