use color_eyre::eyre::Result;

#[cfg(feature = "serde_support")]
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "serde_support")]
use color_eyre::eyre::WrapErr;

#[async_trait::async_trait]
pub trait KucoSqliteStore: Send + Sync + 'static {
    async fn set_bytes(&self, table: String, key: String, value: Vec<u8>) -> Result<()>;
    async fn get_bytes(&self, table: String, key: String) -> Result<Option<Vec<u8>>>;
    async fn clear_all_kv(&self, table: String) -> Result<()>;

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
