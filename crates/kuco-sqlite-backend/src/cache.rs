use color_eyre::eyre::{Result, WrapErr};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::traits::KucoSqliteStore;

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
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
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

    async fn set_bytes(&self, table: String, key: String, value: Vec<u8>) -> Result<()> {
        let query_string = format!(
            "REPLACE INTO {} (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now'))",
            table.as_str()
        );

        sqlx::query(&query_string)
            .bind(key.as_str())
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|sqlx_err| {
                tracing::error!(
                    "SQLxError in set_bytes for key '{}': {}.",
                    key,
                    sqlx_err, // Print the Display form of the error
                );
                if let Some(db_err) = sqlx_err.as_database_error() {
                    tracing::error!(
                        "Underlying SQLite error - Code: {:?}, Message: {}",
                        db_err.code().unwrap_or_default(),
                        db_err.message()
                    );
                }
                color_eyre::eyre::eyre!(
                    "Failed to set key '{}' in SQLite cache. Cause: {}",
                    key,
                    sqlx_err
                )
            })?;
        Ok(())
    }

    // TODO: update error handling to be more detailed like in setter
    async fn get_bytes(&self, table: String, key: String) -> Result<Option<Vec<u8>>> {
        let query_string = format!("SELECT value FROM {} WHERE key = ?", table.as_str());

        let row_option: Option<(Vec<u8>,)> = sqlx::query_as(&query_string)
            .bind(key.as_str())
            .fetch_optional(&self.pool)
            .await
            .wrap_err_with(|| format!("SqliteCache: Failed to get key '{}'", key))?;
        Ok(row_option.map(|(value,)| value))
    }

    async fn erase_all_kv(&self, table: String) -> Result<()> {
        let query_string = format!("DELETE FROM {}", table.as_str());

        sqlx::query(&query_string)
            .execute(&self.pool)
            .await
            .wrap_err("SqliteCache: Failed to clear kv_cache table")?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl KucoSqliteStore for SqliteCache {
    async fn set_bytes(&self, table: String, key: String, value: Vec<u8>) -> Result<()> {
        self.set_bytes(table, key, value).await
    }

    async fn get_bytes(&self, table: String, key: String) -> Result<Option<Vec<u8>>> {
        self.get_bytes(table, key).await
    }

    async fn clear_all_kv(&self, table: String) -> Result<()> {
        self.erase_all_kv(table).await
    }
}
