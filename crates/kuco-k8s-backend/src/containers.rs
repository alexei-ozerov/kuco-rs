use k8s_openapi::api::core::v1::Pod;

use kube::{
    Client,
    api::Api,
};


use crate::error::KucoBackendError;

#[derive(Clone, Debug)]
pub struct ContainerData {
    pub names: Vec<String>,
}

impl Default for ContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerData {
    pub fn new() -> Self {
        ContainerData { names: Vec::new() }
    }

    pub async fn update(
        &mut self,
        client: Client,
        ns: &str,
        po_name: &str,
    ) -> Result<(), KucoBackendError> {
        let pod_api: Api<Pod> = Api::namespaced(client, ns);
        let pod: Pod = pod_api.get(po_name).await?;

        let container_names: Vec<String> = pod
            .spec
            .map(|spec| spec.containers)
            .unwrap_or_default() // If spec or containers is None, return empty vector
            .into_iter()
            .map(|container| container.name)
            .collect();

        self.names = container_names;

        Ok(())
    }
}
