use kube::ResourceExt;
use kube::{
    Client,
    api::{Api, ListParams},
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct PodInfo {
    name: String,
    status: String,
    replicas: Option<i32>,
    desired_replicas: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    // Determine the namespace from an environment variable or default to "default"
    let namespace = env::var("KUBE_NAMESPACE").unwrap_or_else(|_| "kube-system".into());

    // Create a Kubernetes client. This will use your default kubeconfig.
    let client = Client::try_default().await?;

    // Get a reference to the Pod API within the specified namespace.
    let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(client.clone(), &namespace);

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
                            Api::namespaced(client.clone(), &namespace);
                        if let Ok(rs) = rs_api.get(&owner_name).await {
                            desired_replicas = rs.spec.and_then(|s| s.replicas);
                            replicas = rs.status.and_then(|s| Some(s.replicas));
                            break; // Found the ReplicaSet, no need to check others
                        }
                    }
                    "Deployment" => {
                        let deploy_api: Api<k8s_openapi::api::apps::v1::Deployment> =
                            Api::namespaced(client.clone(), &namespace);
                        if let Ok(deploy) = deploy_api.get(&owner_name).await {
                            desired_replicas = deploy.spec.and_then(|s| s.replicas);
                            replicas = deploy.status.and_then(|s| s.replicas);
                            break; // Found the Deployment
                        }
                    }
                    "StatefulSet" => {
                        let sts_api: Api<k8s_openapi::api::apps::v1::StatefulSet> =
                            Api::namespaced(client.clone(), &namespace);
                        if let Ok(sts) = sts_api.get(&owner_name).await {
                            desired_replicas = sts.spec.and_then(|s| s.replicas);
                            replicas = sts.status.and_then(|s| Some(s.replicas));
                            break; // Found the StatefulSet
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

    println!("Pods in namespace '{}':", namespace);
    for info in pod_info_list {
        if let Some(replicas) = info.replicas {
            if replicas == -1 {
                println!(
                    "  Name: {}, Status: {}, Replicas: ERROR",
                    info.name, info.status,
                );
            } else {
                println!(
                    "  Name: {}, Status: {}, Replicas: {}/{}",
                    info.name,
                    info.status,
                    info.replicas.unwrap(),
                    info.desired_replicas.unwrap()
                );
            }
        } else {
            println!("  Name: {}, Status: {}", info.name, info.status);
        }
    }

    Ok(())
}
