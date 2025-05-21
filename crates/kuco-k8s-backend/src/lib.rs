pub mod error;
pub mod context;
pub mod namespaces;
pub mod pods;
pub mod containers;
pub mod logs;


use kube::Client;


use crate::error::KucoBackendError;

// Create a Kubernetes client. This will use your default kubeconfig.
async fn get_client() -> Result<Client, KucoBackendError> {
    let client = Client::try_default().await?;

    Ok(client)
}

