use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::core::v1::Pod;

use kube::ResourceExt;
use kube::{
    Client,
    api::{Api, ListParams},
};

use serde::{Deserialize, Serialize};

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
