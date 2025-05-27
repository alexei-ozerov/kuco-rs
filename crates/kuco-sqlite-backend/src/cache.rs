use color_eyre::eyre::{Result, WrapErr};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::{str::FromStr, sync::Arc};

use crate::traits::KucoSqliteStore;

/// An in-memory cache store using SQLite with sqlx.
#[derive(Clone, Debug)]
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
    fn get_pool(&self) -> Result<Arc<&SqlitePool>> {
        let arc_pool = Arc::new(&self.pool);

        Ok(arc_pool)
    }

    async fn clear_all_kv(&self, table: String) -> Result<()> {
        self.erase_all_kv(table).await
    }
}
