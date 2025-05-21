use kube::Client;

use crate::error::KucoBackendError;
use crate::get_client;

#[derive(Default, Clone)]
pub struct KubeContext {
    pub client: Option<Client>,
}

impl KubeContext {
    pub async fn init_context(&mut self) -> Result<(), KucoBackendError> {
        self.client = Some(get_client().await?);

        Ok(())
    }
}
