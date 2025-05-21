use thiserror::Error;

#[derive(Error, Debug)]
pub enum KucoBackendError {
    #[error("unable to initialize kubernetes client - please verify you can access the cluster")]
    KubeConnectionError(#[from] kube::Error),
    #[error("unknown data store error")]
    Unknown,
}
