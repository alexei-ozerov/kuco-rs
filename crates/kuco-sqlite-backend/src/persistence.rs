use color_eyre::eyre::{Result, WrapErr};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::fs;
use std::{path::Path, str::FromStr, time::Duration};

use crate::traits::KucoSqliteStore;

/// A persistent data store using SQLite with sqlx.
#[derive(Debug, Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    pub async fn new(path: impl AsRef<Path>, timeout: f64) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
        }

        let opts = SqliteConnectOptions::from_str(path.as_os_str().to_str().unwrap())?
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs_f64(timeout))
            .connect_with(opts)
            .await?;

        Self::setup_db(&pool).await?;

        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        // TODO: Figure this out and a good table structure! :3
        // sqlx::migrate!("./record-migrations").run(pool).await?;

        // TODO: remove, as this is for debug purposes only to mirror the in-memory DB
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS kv_cache (
                key TEXT PRIMARY KEY NOT NULL,
                value BLOB NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
        )
        .execute(pool)
        .await
        .wrap_err("Failed to create kv_cache table")?;

        Ok(())
    }

    async fn set_bytes_persistent(&self, table: String, key: String, value: Vec<u8>) -> Result<()> {
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

    async fn get_bytes_persistent(&self, table: String, key: String) -> Result<Option<Vec<u8>>> {
        let query_string = format!("SELECT value FROM {} WHERE key = ?", table.as_str());

        let row_option: Option<(Vec<u8>,)> = sqlx::query_as(&query_string)
            .bind(key.as_str())
            .fetch_optional(&self.pool)
            .await
            .wrap_err_with(|| format!("SqliteCache: Failed to get key '{}'", key))?;

        Ok(row_option.map(|(value,)| value))
    }

    async fn erase_all_persistent_kv(&self, table: String) -> Result<()> {
        let query_string = format!("DELETE FROM {}", table.as_str());

        sqlx::query(&query_string)
            .execute(&self.pool)
            .await
            .wrap_err("SqliteCache: Failed to clear kv_cache table")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl KucoSqliteStore for SqliteDb {
    async fn set_bytes(&self, table: String, key: String, value: Vec<u8>) -> Result<()> {
        self.set_bytes_persistent(table, key, value).await
    }

    async fn get_bytes(&self, table: String, key: String) -> Result<Option<Vec<u8>>> {
        self.get_bytes_persistent(table, key).await
    }

    async fn clear_all_kv(&self, table: String) -> Result<()> {
        self.erase_all_persistent_kv(table).await
    }
}
