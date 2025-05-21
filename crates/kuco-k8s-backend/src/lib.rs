use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::core::v1::{Namespace, Pod};

use kube::ResourceExt;
use kube::api::LogParams;
use kube::{
    Client,
    api::{Api, ListParams},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KucoBackendError {
    #[error("unable to initialize kubernetes client - please verify you can access the cluster")]
    KubeConnectionError(#[from] kube::Error),
    #[error("unknown data store error")]
    Unknown,
}

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

// Create a Kubernetes client. This will use your default kubeconfig.
async fn get_client() -> Result<Client, KucoBackendError> {
    let client = Client::try_default().await?;

    Ok(client)
}

#[derive(Clone, Debug)]
pub struct NamespaceData {
    pub names: Vec<String>,
}

impl Default for NamespaceData {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceData {
    pub fn new() -> Self {
        NamespaceData { names: Vec::new() }
    }

    pub async fn update(&mut self, client: Client) {
        let ns_api_data: Api<Namespace> = Api::all(client);

        // List all pods in the namespace.
        let lp = ListParams::default();
        let ns_list = ns_api_data.list(&lp).await.unwrap().items;

        // If a namespace was deleted, remove it as well.
        if ns_list.len() < self.names.len() {
            let mut replacement_vec: Vec<String> = Vec::new();

            // TODO: find a better way to do this ...
            for ns in &ns_list {
                let ns_name = ns.name_any();
                replacement_vec.push(ns_name);
            }
            self.names = replacement_vec;
        }

        for ns in ns_list {
            let ns_name = ns.name_any();

            // If not already in the array, add it.
            if !self.names.contains(&ns_name) {
                self.names.push(ns_name);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PodInfo {
    pub name: String,
    pub status: String,
    pub replicas: Option<i32>,
    pub desired_replicas: Option<i32>,
}

impl PodInfo {
    pub async fn update(
        &mut self,
        client: Client,
        namespace: &str,
        pod_name: &str,
    ) -> Result<(), kube::Error> {
        let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
        let pod = pods.get(pod_name).await?;

        // Update name
        self.name = pod.name_any();

        // Update status
        self.status = pod
            .status
            .and_then(|s| s.phase)
            .unwrap_or_else(|| "Unknown".into());

        if let Some(owners) = &pod.metadata.owner_references {
            for owner in owners {
                let kind = &owner.kind;
                let owner_name = &owner.name;
                match kind.as_str() {
                    "ReplicaSet" => {
                        let rs_api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
                        if let Ok(rs) = rs_api.get(owner_name).await {
                            self.desired_replicas = rs.spec.and_then(|s| s.replicas);
                            self.replicas = rs.status.map(|s| s.replicas);
                            break;
                        }
                    }
                    "Deployment" => {
                        let deploy_api: Api<Deployment> =
                            Api::namespaced(client.clone(), namespace);
                        if let Ok(deploy) = deploy_api.get(owner_name).await {
                            self.desired_replicas = deploy.spec.and_then(|s| s.replicas);
                            self.replicas = deploy.status.and_then(|s| s.replicas);
                            break;
                        }
                    }
                    "StatefulSet" => {
                        let sts_api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
                        if let Ok(sts) = sts_api.get(owner_name).await {
                            self.desired_replicas = sts.spec.and_then(|s| s.replicas);
                            self.replicas = sts.status.map(|s| s.replicas);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PodData {
    pub list: Vec<PodInfo>,
    pub names: Vec<String>,
}

impl PodData {
    pub async fn get_names(&mut self, client: Client, namespace: &str) -> Result<(), kube::Error> {
        let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

        // List all pods in the namespace.
        let lp = ListParams::default();
        let pod_list = pods.list(&lp).await?;

        let mut pod_name_list: Vec<String> = Vec::new();
        for pod in pod_list.items {
            let pod_name = pod.name_any();
            pod_name_list.push(pod_name);
        }

        self.names = pod_name_list;

        Ok(())
    }

    pub async fn update_all(&mut self, client: Client, namespace: &str) -> Result<(), kube::Error> {
        // Get a reference to the Pod API within the specified namespace.
        let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

        // List all pods in the namespace.
        let lp = ListParams::default();
        let pod_list = pods.list(&lp).await?;

        let mut pod_info_list: Vec<PodInfo> = Vec::new();

        for pod in pod_list.items {
            let pod_name = pod.name_any();
            let pod_status = pod
                .status
                .and_then(|s| s.phase)
                .unwrap_or_else(|| "Unknown".into());

            let mut replicas: Option<i32> = None;
            let mut desired_replicas: Option<i32> = None;

            if let Some(owners) = &pod.metadata.owner_references {
                for owner in owners {
                    let kind = &owner.kind;
                    let owner_name = &owner.name;
                    match kind.as_str() {
                        "ReplicaSet" => {
                            let rs_api: Api<k8s_openapi::api::apps::v1::ReplicaSet> =
                                Api::namespaced(client.clone(), namespace);
                            if let Ok(rs) = rs_api.get(owner_name).await {
                                desired_replicas = rs.spec.and_then(|s| s.replicas);
                                replicas = rs.status.map(|s| s.replicas);
                                break;
                            }
                        }
                        "Deployment" => {
                            let deploy_api: Api<k8s_openapi::api::apps::v1::Deployment> =
                                Api::namespaced(client.clone(), namespace);
                            if let Ok(deploy) = deploy_api.get(owner_name).await {
                                desired_replicas = deploy.spec.and_then(|s| s.replicas);
                                replicas = deploy.status.and_then(|s| s.replicas);
                                break;
                            }
                        }
                        "StatefulSet" => {
                            let sts_api: Api<k8s_openapi::api::apps::v1::StatefulSet> =
                                Api::namespaced(client.clone(), namespace);
                            if let Ok(sts) = sts_api.get(owner_name).await {
                                desired_replicas = sts.spec.and_then(|s| s.replicas);
                                replicas = sts.status.map(|s| s.replicas);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            pod_info_list.push(PodInfo {
                name: pod_name,
                status: pod_status,
                replicas,
                desired_replicas,
            });
        }

        self.list = pod_info_list;

        Ok(())
    }
}

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

#[derive(Clone, Debug)]
pub struct LogData {
    pub lines: Vec<String>,
}

impl Default for LogData {
    fn default() -> Self {
        Self::new()
    }
}

impl LogData {
    pub fn new() -> Self {
        LogData { lines: Vec::new() }
    }

    pub async fn update(
        &mut self,
        client: Client,
        namespace: &str,
        pod_name: &str,
        container_name: &str,
    ) -> Result<(), KucoBackendError> {
        let pods_api: Api<Pod> = Api::namespaced(client, namespace);

        // 2. Define LogParams to specify the container and other log options.
        //    - `container`: Specifies which container's logs to fetch.
        //    - `follow`: If true, streams logs. Default is false (get current logs).
        //    - `timestamps`: If true, adds timestamps to log lines.
        //    - `tail_lines`: Fetches only the last N lines.
        //    - `previous`: If true, fetches logs from a previous, terminated instance of the container.
        let log_params = LogParams {
            container: Some(container_name.to_string()),
            timestamps: true, // Example: include timestamps
            // tail_lines: Some(100), // Example: get last 100 lines
            ..Default::default() // Uses default for follow (false), previous (false), etc.
        };

        let log_string = pods_api.logs(pod_name, &log_params).await.unwrap_or({
            format!(
                "Failed to fetch logs for container '{}' in pod '{}', namespace '{}'",
                container_name, pod_name, namespace
            )
        });

        let logs_vector: Vec<String> = log_string.lines().map(String::from).collect();

        self.lines = logs_vector;

        Ok(())
    }
}
