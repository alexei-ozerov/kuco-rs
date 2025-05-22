use std::sync::{Arc, Mutex};

use color_eyre::eyre::{Result, eyre};
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};

#[derive(Default, Clone)]
pub struct ArcConn {
    pub arc: Arc<Mutex<Option<MultiplexedConnection>>>,
}

#[derive(Clone)]
pub struct CacheStore {
    pub client: Option<Client>,
    pub connection: ArcConn,
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            client: None,
            connection: ArcConn::default(),
        }
    }

    pub fn create_client(&mut self) -> Result<()> {
        self.client = Some(Client::open("redis://127.0.0.1/")?);
        Ok(())
    }

    pub async fn open_connection(&mut self) -> Result<()> {
        match &self.client {
            Some(client_ref) => {
                let new_connection = client_ref.get_multiplexed_async_connection().await?;
                tracing::info!("Successfully fetched multiplexed connection.");

                let mut locked_guard = self.connection.arc.lock().unwrap();
                *locked_guard = Some(new_connection);
                tracing::info!("Opened connection to Valkey and stored it.");
            }
            None => {
                tracing::warn!("Client has not been initialized, unable to set connection.");
                let mut locked_guard = self.connection.arc.lock().unwrap();
                *locked_guard = None;
            }
        };
        Ok(())
    }

    pub async fn set<
        'a,
        K: redis::ToRedisArgs + std::marker::Send + std::marker::Sync + 'a,
        V: redis::ToRedisArgs + std::marker::Send + std::marker::Sync + 'a,
    >(
        &self,
        key: K,
        value: V,
    ) -> Result<()> {
        let connection_arc = Arc::clone(&self.connection.arc);
        let mut instanced_connection: MultiplexedConnection;

        // Scope for the MutexGuard
        {
            let guard = connection_arc.lock().unwrap();
            match guard.as_ref() {
                Some(conn_ref) => {
                    instanced_connection = conn_ref.clone();
                }
                None => {
                    tracing::error!(
                        "Attempted to use `set` but Redis connection is not established."
                    );
                    return Err(eyre!("Redis connection not established"));
                }
            }
        }

        let _: () = instanced_connection.set(key, value).await?;
        tracing::debug!("Successfully set key in Redis.");

        Ok(())
    }
}
