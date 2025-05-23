use crate::interface::SqlxStore;

use color_eyre::eyre::{Result, WrapErr};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

#[cfg(feature = "serde_support")]
use serde::{Serialize, de::DeserializeOwned};

/// An in-memory cache store using SQLite with sqlx.
#[derive(Clone)]
pub struct SqliteCache {
    pool: SqlitePool,
}

impl SqliteCache {
    pub async fn new_in_memory() -> Result<Self> {
        let connect_options = SqliteConnectOptions::from_str("sqlite::memory:")
            .wrap_err("Failed to parse in-memory SQLite connection string")?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5) // TODO: Verify if this needs to be less or more ...
            .connect_with(connect_options)
            .await
            .wrap_err("Failed to create SQLite connection pool")?;

        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS kv_cache (
                key TEXT PRIMARY KEY NOT NULL,
                value BLOB NOT NULL,
                cached_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(&self.pool)
        .await
        .wrap_err("Failed to create kv_cache table")?;

        Ok(())
    }

    pub async fn dump_to_file(&self, file_path: &str) -> Result<()> {
        sqlx::query("BACKUP TO ?")
            .bind(file_path)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .wrap_err_with(|| format!("Failed to backup SQLite cache to file: {}", file_path))
    }
}

impl SqlxStore for SqliteCache {
    /// Sets a key with a byte value. The value should be pre-serialized.
    async fn set_bytes(&self, key: String, value: Vec<u8>) -> Result<()> {
        sqlx::query(
            "REPLACE INTO kv_cache (key, value, cached_at) VALUES (?, ?, strftime('%s', 'now'))",
        )
        .bind(key.as_str()) // Use as_str() if key is String for &str binding
        .bind(value)
        .execute(&self.pool)
        .await
        .wrap_err_with(|| format!("Failed to set key '{}' in SQLite cache", key))?;
        Ok(())
    }

    /// Gets a byte value for a key. Deserialization is the caller's responsibility.
    async fn get_bytes(&self, key: String) -> Result<Option<Vec<u8>>> {
        let row_option: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT value FROM kv_cache WHERE key = ?")
                .bind(key.as_str())
                .fetch_optional(&self.pool)
                .await
                .wrap_err_with(|| format!("Failed to get key '{}' from SQLite cache", key))?;

        Ok(row_option.map(|(value,)| value))
    }

    #[cfg(feature = "serde_support")]
    async fn set_json<S: Serialize + Send + Sync + 'static>(
        &self,
        key: String,
        value: &S,
    ) -> Result<()> {
        let json_bytes = serde_json::to_vec(value)
            .wrap_err_with(|| format!("Failed to serialize value for key '{}' to JSON", key))?;
        self.set_bytes(key, json_bytes).await
    }

    #[cfg(feature = "serde_support")]
    async fn get_json<D: DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: String,
    ) -> Result<Option<D>> {
        match self.get_bytes(key.clone()).await? {
            Some(bytes) => {
                let deserialized: D = serde_json::from_slice(&bytes).wrap_err_with(|| {
                    format!("Failed to deserialize JSON value for key '{}'", key)
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn clear_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM kv_cache")
            .execute(&self.pool)
            .await
            .wrap_err("Failed to clear SQLite cache")?;
        Ok(())
    }
}
