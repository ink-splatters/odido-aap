use super::{CacheBackend, CacheStats};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Row, sqlite::SqlitePool};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, trace};

pub struct SqliteBackend {
    pool: SqlitePool,
}

impl SqliteBackend {
    /// Create new SQLite cache backend
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache directory: {:?}", parent))?;
        }

        let url = format!("sqlite:{}?mode=rwc", db_path.display());
        debug!("Opening cache database: {}", url);

        let pool = SqlitePool::connect(&url)
            .await
            .with_context(|| format!("Failed to connect to cache database: {}", url))?;

        // Enable WAL mode for better concurrent access
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .context("Failed to enable WAL mode")?;

        let backend = Self { pool };
        backend.init().await?;

        Ok(backend)
    }

    /// Initialize database schema
    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cache_entries (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                content_type TEXT DEFAULT 'json'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create cache_entries table")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_expires_at
            ON cache_entries(expires_at)
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create expires_at index")?;

        debug!("Cache database initialized");
        Ok(())
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = now_timestamp();

        // Use <= to match get()'s expires_at > now check (entry is expired when expires_at <= now)
        let result = sqlx::query("DELETE FROM cache_entries WHERE expires_at <= ?")
            .bind(now as i64)
            .execute(&self.pool)
            .await
            .context("Failed to cleanup expired entries")?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            debug!("Cleaned up {} expired cache entries", deleted);
        }

        Ok(deleted)
    }
}

#[async_trait]
impl CacheBackend for SqliteBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let now = now_timestamp();

        let row = sqlx::query(
            r#"
            SELECT value, expires_at
            FROM cache_entries
            WHERE key = ? AND expires_at > ?
            "#,
        )
        .bind(key)
        .bind(now as i64)
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("Failed to get cache entry: {}", key))?;

        match row {
            Some(row) => {
                let value: Vec<u8> = row.get("value");
                trace!("Cache HIT: {} ({} bytes)", key, value.len());
                Ok(Some(value))
            }
            None => {
                trace!("Cache MISS: {}", key);
                Ok(None)
            }
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let now = now_timestamp();
        let expires_at = now + ttl.as_secs();

        sqlx::query(
            r#"
            INSERT INTO cache_entries (key, value, created_at, expires_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                created_at = excluded.created_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now as i64)
        .bind(expires_at as i64)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Failed to set cache entry: {}", key))?;

        trace!("Cache SET: {} ({} bytes, ttl: {:?})", key, value.len(), ttl);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM cache_entries WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to delete cache entry: {}", key))?;

        trace!("Cache DELETE: {}", key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        sqlx::query("DELETE FROM cache_entries")
            .execute(&self.pool)
            .await
            .context("Failed to clear cache")?;

        debug!("Cache cleared");
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        let now = now_timestamp();
        let row = sqlx::query(
            "SELECT COUNT(*) as count, COALESCE(SUM(LENGTH(value)), 0) as size FROM cache_entries WHERE expires_at > ?"
        )
            .bind(now as i64)
            .fetch_one(&self.pool)
            .await
            .context("Failed to get cache stats")?;

        let total_entries: i64 = row.get("count");
        let total_size_bytes: i64 = row.get("size");

        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_size_bytes: total_size_bytes as u64,
        })
    }
}

/// Get current Unix timestamp in seconds
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX_EPOCH")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sqlite_backend_basic() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Test set and get
        cache
            .set("test:key", b"test value", Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get("test:key").await.unwrap();
        assert_eq!(value, Some(b"test value".to_vec()));

        // Test delete
        cache.delete("test:key").await.unwrap();
        let value = cache.get("test:key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_sqlite_backend_expiration() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Set with short TTL (1 second)
        cache
            .set("test:expire", b"value", Duration::from_secs(1))
            .await
            .unwrap();

        // Should exist immediately
        let value = cache.get("test:expire").await.unwrap();
        assert_eq!(value, Some(b"value".to_vec()));

        // Wait for expiration (1.1 seconds to be safe)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should be expired
        let value = cache.get("test:expire").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_sqlite_backend_stats() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        cache
            .set("key1", b"value1", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key2", b"value2", Duration::from_secs(60))
            .await
            .unwrap();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_size_bytes, 12); // "value1" + "value2"
    }

    // ───────── cleanup_expired tests ─────────

    #[tokio::test]
    async fn test_cleanup_expired_removes_expired_entries() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Set entry with very short TTL
        cache
            .set("expired:key", b"value", Duration::from_secs(1))
            .await
            .unwrap();

        // Set entry with long TTL
        cache
            .set("valid:key", b"value", Duration::from_secs(3600))
            .await
            .unwrap();

        // Wait for first to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Cleanup should remove 1 entry
        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Valid entry should still exist
        let value = cache.get("valid:key").await.unwrap();
        assert!(value.is_some());

        // Expired entry should be gone (already was via get, but also via cleanup)
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_cleanup_expired_on_empty_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 0);
    }

    #[tokio::test]
    async fn test_cleanup_expired_with_no_expired_entries() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        cache
            .set("key1", b"value1", Duration::from_secs(3600))
            .await
            .unwrap();
        cache
            .set("key2", b"value2", Duration::from_secs(3600))
            .await
            .unwrap();

        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 0);

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 2);
    }

    // ───────── Edge case tests ─────────

    #[tokio::test]
    async fn test_overwrite_existing_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        cache
            .set("key", b"value1", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key", b"value2", Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get("key").await.unwrap();
        assert_eq!(value, Some(b"value2".to_vec()));

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_unicode_keys_and_values() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        let unicode_key = "user:日本語:data";
        let unicode_value = "値は日本語です 🎉".as_bytes();

        cache
            .set(unicode_key, unicode_value, Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get(unicode_key).await.unwrap();
        assert_eq!(value, Some(unicode_value.to_vec()));
    }

    #[tokio::test]
    async fn test_large_value() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // 1MB value
        let large_value: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

        cache
            .set("large:key", &large_value, Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get("large:key").await.unwrap();
        assert_eq!(value, Some(large_value));
    }

    #[tokio::test]
    async fn test_empty_value() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        cache
            .set("empty:key", b"", Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get("empty:key").await.unwrap();
        assert_eq!(value, Some(vec![]));
    }

    #[tokio::test]
    async fn test_delete_nonexistent_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Should not error
        cache.delete("nonexistent:key").await.unwrap();
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        let value = cache.get("nonexistent:key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_clear_removes_all_entries() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        cache
            .set("key1", b"value1", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key2", b"value2", Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key3", b"value3", Duration::from_secs(60))
            .await
            .unwrap();

        cache.clear().await.unwrap();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);

        // Verify all keys are gone
        assert_eq!(cache.get("key1").await.unwrap(), None);
        assert_eq!(cache.get("key2").await.unwrap(), None);
        assert_eq!(cache.get("key3").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_stats_excludes_expired_entries() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Set one entry that will expire
        cache
            .set("expired:key", b"value", Duration::from_secs(1))
            .await
            .unwrap();

        // Set one entry that won't expire
        cache
            .set("valid:key", b"valid", Duration::from_secs(3600))
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Stats should only count valid entry
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_size_bytes, 5); // "valid"
    }

    #[tokio::test]
    async fn test_special_characters_in_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        let special_keys = vec![
            "key:with:colons",
            "key/with/slashes",
            "key with spaces",
            "key\twith\ttabs",
            "key\nwith\nnewlines",
            "key'with'quotes",
            "key\"with\"doublequotes",
            "key%with%percent",
            "key=with=equals",
            "key&with&ampersand",
        ];

        for key in &special_keys {
            cache
                .set(key, b"value", Duration::from_secs(60))
                .await
                .unwrap();

            let value = cache.get(key).await.unwrap();
            assert_eq!(value, Some(b"value".to_vec()), "Failed for key: {}", key);
        }

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_entries, special_keys.len());
    }

    #[tokio::test]
    async fn test_binary_value() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // Binary data with null bytes and all byte values
        let binary_value: Vec<u8> = (0..=255).collect();

        cache
            .set("binary:key", &binary_value, Duration::from_secs(60))
            .await
            .unwrap();

        let value = cache.get("binary:key").await.unwrap();
        assert_eq!(value, Some(binary_value));
    }

    #[tokio::test]
    async fn test_ttl_boundary_zero_seconds() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = SqliteBackend::new(db_path).await.unwrap();

        // TTL of 0 should expire immediately (or nearly so)
        cache
            .set("zero:ttl", b"value", Duration::from_secs(0))
            .await
            .unwrap();

        // Since expires_at = now + 0, it should be expired immediately
        // (or within the same second, which our get() check handles)
        tokio::time::sleep(Duration::from_millis(10)).await;
        let value = cache.get("zero:ttl").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_reopen_database_persists_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // First connection
        {
            let cache = SqliteBackend::new(db_path.clone()).await.unwrap();
            cache
                .set("persist:key", b"persistent", Duration::from_secs(3600))
                .await
                .unwrap();
        }

        // Second connection (reopening the database)
        {
            let cache = SqliteBackend::new(db_path).await.unwrap();
            let value = cache.get("persist:key").await.unwrap();
            assert_eq!(value, Some(b"persistent".to_vec()));
        }
    }
}
