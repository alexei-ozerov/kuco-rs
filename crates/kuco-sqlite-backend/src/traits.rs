use std::sync::Arc;

use color_eyre::eyre::Result;

#[cfg(feature = "serde_support")]
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "serde_support")]
use color_eyre::eyre::WrapErr;
use sqlx::SqlitePool;

#[async_trait::async_trait]
pub trait KucoSqliteStore: Send + Sync + 'static {
    fn get_pool(&self) -> Result<Arc<&SqlitePool>>;
    async fn clear_all_kv(&self, table: String) -> Result<()>;

    async fn set_bytes(&self, table: String, key: String, value: Vec<u8>) -> Result<()> {
        let pool = self.get_pool()?;

        let query_string = format!(
            "REPLACE INTO {} (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now'))",
            table.as_str()
        );

        sqlx::query(&query_string)
            .bind(key.as_str())
            .bind(value)
            .execute(*pool.as_ref())
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

    async fn get_bytes(&self, table: String, key: String) -> Result<Option<Vec<u8>>> {
        let pool = self.get_pool()?;

        let query_string = format!("SELECT value FROM {} WHERE key = ?", table.as_str());

        let row_option: Option<(Vec<u8>,)> = sqlx::query_as(&query_string)
            .bind(key.as_str())
            .fetch_optional(*pool.as_ref())
            .await
            .wrap_err_with(|| format!("SqliteCache: Failed to get key '{}'", key))?;

        Ok(row_option.map(|(value,)| value))
    }

    #[cfg(feature = "serde_support")]
    async fn set_json<S: Serialize + Send + Sync + 'static>(
        &self,
        table: String,
        key: String,
        value: &S,
    ) -> Result<()> {
        let json_bytes = serde_json::to_vec(value).wrap_err_with(|| {
            format!(
                "KvStore: Failed to serialize value for key '{}' to JSON",
                key
            )
        })?;
        self.set_bytes(table, key, json_bytes).await
    }

    #[cfg(feature = "serde_support")]
    async fn get_json<D: DeserializeOwned + Send + Sync + 'static>(
        &self,
        table: String,
        key: String,
    ) -> Result<Option<D>> {
        match self.get_bytes(table, key.clone()).await? {
            Some(bytes) => {
                let deserialized: D = serde_json::from_slice(&bytes).wrap_err_with(|| {
                    format!(
                        "KvStore: Failed to deserialize JSON value for key '{}'",
                        key
                    )
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
}
