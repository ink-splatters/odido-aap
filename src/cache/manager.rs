use super::CacheBackend;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// High-level cache manager with typed operations
pub struct CacheManager<B: CacheBackend> {
    backend: B,
}

impl<B: CacheBackend> CacheManager<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Get value from cache with automatic deserialization
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.backend.get(key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set value in cache with automatic serialization
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let bytes = serde_json::to_vec(value)?;
        self.backend.set(key, &bytes, ttl).await
    }

    /// Get value from cache or fetch with provided function
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetcher: F,
    ) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try cache first
        if let Some(cached) = self.get(key).await? {
            return Ok(cached);
        }

        // Cache miss - fetch from source
        debug!("Cache miss, fetching: {}", key);
        let value = fetcher().await?;

        // Store in cache
        self.set(key, &value, ttl).await?;

        Ok(value)
    }

    /// Delete entry from cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.backend.delete(key).await
    }

    /// Clear entire cache
    pub async fn clear(&self) -> Result<()> {
        self.backend.clear().await
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<super::CacheStats> {
        self.backend.stats().await
    }
}

/// TTL constants for different resource types
pub struct CacheTtl;

impl CacheTtl {
    pub const LINKED_SUBSCRIPTIONS: Duration = Duration::from_secs(12 * 3600); // 12 hours
    pub const SUBSCRIPTION_DETAILS: Duration = Duration::from_secs(6 * 3600); // 6 hours
    pub const ROAMING_BUNDLES: Duration = Duration::from_secs(1 * 3600); // 1 hour
    pub const AVAILABLE_BUNDLES: Duration = Duration::from_secs(12 * 3600); // 12 hours
    pub const COUNTRY_INFO: Duration = Duration::from_secs(7 * 24 * 3600); // 7 days
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::SqliteBackend;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        value: String,
    }

    #[tokio::test]
    async fn test_cache_manager_typed() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let data = TestData {
            value: "test".to_string(),
        };

        // Test set and get with type safety
        manager
            .set("test:key", &data, Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved: Option<TestData> = manager.get("test:key").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_cache_manager_get_or_fetch() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let mut fetch_count = 0;

        // First call should fetch
        let result: TestData = manager
            .get_or_fetch("test:fetch", Duration::from_secs(60), || async {
                fetch_count += 1;
                Ok(TestData {
                    value: "fetched".to_string(),
                })
            })
            .await
            .unwrap();

        assert_eq!(result.value, "fetched");
        assert_eq!(fetch_count, 1);

        // Second call should use cache (fetch_count stays 1)
        let result: TestData = manager
            .get_or_fetch("test:fetch", Duration::from_secs(60), || async {
                fetch_count += 1;
                Ok(TestData {
                    value: "fetched2".to_string(),
                })
            })
            .await
            .unwrap();

        assert_eq!(result.value, "fetched"); // Still original value
        assert_eq!(fetch_count, 1); // Fetcher not called
    }
}
