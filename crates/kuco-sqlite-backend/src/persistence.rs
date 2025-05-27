use color_eyre::eyre::{Result, WrapErr};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::fs;
use std::sync::Arc;
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
    fn get_pool(&self) -> Result<Arc<&SqlitePool>> {
        let arc_pool = Arc::new(&self.pool);

        Ok(arc_pool)
    }

    async fn clear_all_kv(&self, table: String) -> Result<()> {
        self.erase_all_persistent_kv(table).await
    }
}
