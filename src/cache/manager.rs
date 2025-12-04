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
    pub async fn get_or_fetch<T, F, Fut>(&self, key: &str, ttl: Duration, fetcher: F) -> Result<T>
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
}

/// TTL constants for different resource types
pub struct CacheTtl;

impl CacheTtl {
    pub const LINKED_SUBSCRIPTIONS: Duration = Duration::from_secs(12 * 3600); // 12 hours
    pub const ROAMING_BUNDLES: Duration = Duration::from_secs(3600); // 1 hour
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::SqliteBackend;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

    // ───────── Edge case tests ─────────

    #[tokio::test]
    async fn test_get_nonexistent_key_returns_none() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let result: Option<TestData> = manager.get("nonexistent:key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_key_succeeds() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        // Should not error
        manager.delete("nonexistent:key").await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_existing_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let data = TestData {
            value: "to_delete".to_string(),
        };

        manager
            .set("delete:key", &data, Duration::from_secs(60))
            .await
            .unwrap();

        // Verify it exists
        let retrieved: Option<TestData> = manager.get("delete:key").await.unwrap();
        assert!(retrieved.is_some());

        // Delete it
        manager.delete("delete:key").await.unwrap();

        // Verify it's gone
        let retrieved: Option<TestData> = manager.get("delete:key").await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_clear_removes_all_entries() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        // Set multiple entries
        for i in 0..5 {
            let data = TestData {
                value: format!("value{}", i),
            };
            manager
                .set(&format!("key:{}", i), &data, Duration::from_secs(60))
                .await
                .unwrap();
        }

        // Clear all
        manager.clear().await.unwrap();

        // Verify all are gone
        for i in 0..5 {
            let result: Option<TestData> = manager.get(&format!("key:{}", i)).await.unwrap();
            assert_eq!(result, None);
        }
    }

    #[tokio::test]
    async fn test_get_or_fetch_with_fetcher_error() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let result: Result<TestData> = manager
            .get_or_fetch("error:key", Duration::from_secs(60), || async {
                Err(anyhow::anyhow!("Fetch failed"))
            })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Fetch failed"));
    }

    #[tokio::test]
    async fn test_get_with_corrupted_json() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path.clone()).await.unwrap();

        // Directly set invalid JSON via backend
        backend
            .set(
                "corrupted:key",
                b"not valid json {{{",
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        let manager = CacheManager::new(backend);

        // Trying to deserialize should fail
        let result: Result<Option<TestData>> = manager.get("corrupted:key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_or_fetch_with_expired_cache_refetches() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        use std::sync::atomic::{AtomicU32, Ordering};
        let fetch_count = AtomicU32::new(0);

        // First fetch with short TTL
        let _: TestData = manager
            .get_or_fetch("expire:key", Duration::from_secs(1), || async {
                fetch_count.fetch_add(1, Ordering::SeqCst);
                Ok(TestData {
                    value: "first".to_string(),
                })
            })
            .await
            .unwrap();

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should refetch
        let result: TestData = manager
            .get_or_fetch("expire:key", Duration::from_secs(60), || async {
                fetch_count.fetch_add(1, Ordering::SeqCst);
                Ok(TestData {
                    value: "second".to_string(),
                })
            })
            .await
            .unwrap();

        assert_eq!(fetch_count.load(Ordering::SeqCst), 2);
        assert_eq!(result.value, "second");
    }

    #[tokio::test]
    async fn test_complex_nested_type() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Nested {
            items: Vec<TestData>,
            count: u32,
            metadata: std::collections::HashMap<String, String>,
        }

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let complex = Nested {
            items: vec![
                TestData {
                    value: "item1".to_string(),
                },
                TestData {
                    value: "item2".to_string(),
                },
            ],
            count: 42,
            metadata,
        };

        manager
            .set("complex:key", &complex, Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved: Option<Nested> = manager.get("complex:key").await.unwrap();
        assert_eq!(retrieved, Some(complex));
    }

    #[tokio::test]
    async fn test_unicode_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let data = TestData {
            value: "日本語テスト 🎉 émoji".to_string(),
        };

        manager
            .set("unicode:key", &data, Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved: Option<TestData> = manager.get("unicode:key").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_empty_string_value() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let data = TestData {
            value: "".to_string(),
        };

        manager
            .set("empty:key", &data, Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved: Option<TestData> = manager.get("empty:key").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_overwrite_preserves_only_latest() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let backend = SqliteBackend::new(db_path).await.unwrap();
        let manager = CacheManager::new(backend);

        let data1 = TestData {
            value: "first".to_string(),
        };
        let data2 = TestData {
            value: "second".to_string(),
        };

        manager
            .set("overwrite:key", &data1, Duration::from_secs(60))
            .await
            .unwrap();
        manager
            .set("overwrite:key", &data2, Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved: Option<TestData> = manager.get("overwrite:key").await.unwrap();
        assert_eq!(retrieved, Some(data2));
    }
}
