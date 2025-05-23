use crate::cache::SqliteCache;
use crate::persistence::SqliteDb;

use color_eyre::eyre::Result;

#[cfg(feature = "serde_support")]
use serde::{Serialize, de::DeserializeOwned};

// Generic trait to describe shared functionality of the Sqlite Storage Options
pub trait SqlxStore {
    fn set_bytes(
        &self,
        key: String,
        value: Vec<u8>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn get_bytes(
        &self,
        key: String,
    ) -> impl std::future::Future<Output = Result<Option<Vec<u8>>>> + Send;

    #[cfg(feature = "serde_support")]
    fn set_json<S: Serialize + Send + Sync + 'static>(
        &self,
        key: String,
        value: &S,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    #[cfg(feature = "serde_support")]
    fn get_json<D: DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: String,
    ) -> impl std::future::Future<Output = Result<Option<D>>> + Send;

    fn clear_all(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}

#[derive(Clone)]
pub struct SqliteStorageInterface {
    pub cache: Option<SqliteCache>,
    pub db: Option<SqliteDb>,
}
