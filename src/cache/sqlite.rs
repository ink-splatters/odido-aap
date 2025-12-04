use super::{CacheBackend, CacheStats};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{sqlite::SqlitePool, Row};
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

        let result = sqlx::query("DELETE FROM cache_entries WHERE expires_at < ?")
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
}
