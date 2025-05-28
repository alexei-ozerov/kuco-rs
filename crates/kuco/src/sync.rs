use chrono::Utc;
use color_eyre::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use kuco_k8s_backend::{context::KubeContext, namespaces::NamespaceData, pods::PodData};
use kuco_sqlite_backend::KucoSqliteStore;
use std::time::Duration;
use tokio::time::interval;

use crate::constants::{
    CONT_NAMES_CACHE_KEY, KUCO_CACHE_TABLE, NS_NAMES_CACHE_KEY, POD_NAMES_CACHE_KEY,
};

async fn run_stage1_sync<S: KucoSqliteStore + Clone + 'static>(
    client: &Client,
    cache_store: &S,
) -> Result<Vec<String>> {
    tracing::info!("Running Stage 1 Sync: Namespaces and Pod Names");
    let mut ns_data_fetcher = NamespaceData::new();
    ns_data_fetcher.update(client.clone()).await; // Fetches namespace names

    cache_store
        .set_json(
            KUCO_CACHE_TABLE.to_owned(),
            NS_NAMES_CACHE_KEY.to_string(),
            &ns_data_fetcher.names,
        )
        .await?;

    for ns_name in &ns_data_fetcher.names {
        let mut pod_data_fetcher = PodData::default();
        match pod_data_fetcher.get_names(client.clone(), ns_name).await {
            Ok(_) => {
                let pod_names_key = format!("{}{}", POD_NAMES_CACHE_KEY, ns_name);
                cache_store
                    .set_json(
                        KUCO_CACHE_TABLE.to_owned(),
                        pod_names_key,
                        &pod_data_fetcher.names,
                    )
                    .await?;
            }
            Err(e) => tracing::error!("Stage 1: Failed to get pod names for ns {}: {}", ns_name, e),
        }
    }

    let current_timestamp_seconds: i64 = Utc::now().timestamp();
    cache_store
        .set_json(
            KUCO_CACHE_TABLE.to_owned(),
            "last_refreshed_at".to_string(),
            &current_timestamp_seconds,
        )
        .await?;
    tracing::info!("Finished Stage 1 Sync.");
    Ok(ns_data_fetcher.names)
}

async fn run_stage2_sync_for_namespace<S: KucoSqliteStore + Clone + 'static>(
    client: &Client,
    cache_store: &S,
    ns_name: &str,
) {
    let pod_names_key = format!("{}{}", POD_NAMES_CACHE_KEY, ns_name);
    let pod_names_for_ns: Option<Vec<String>> = cache_store
        .get_json(KUCO_CACHE_TABLE.to_owned(), pod_names_key)
        .await
        .unwrap_or_default();

    if let Some(pod_names) = pod_names_for_ns {
        for pod_name in pod_names {
            let cont_cache_key = format!("{}{}_{}", CONT_NAMES_CACHE_KEY, ns_name, pod_name);

            // Check if already cached to avoid refetching too aggressively in this slow loop
            if cache_store
                .get_bytes(KUCO_CACHE_TABLE.to_owned(), cont_cache_key.clone())
                .await
                .ok()
                .flatten()
                .is_none()
            {
                tracing::debug!(
                    "Stage 2: Fetching container details for pod {}/{}",
                    ns_name,
                    pod_name
                );
                let pods_api_for_detail: Api<Pod> = Api::namespaced(client.clone(), ns_name);
                match pods_api_for_detail.get(&pod_name).await {
                    Ok(pod_detail) => {
                        let container_names: Vec<String> = pod_detail
                            .spec
                            .map(|spec| spec.containers.into_iter().map(|c| c.name).collect())
                            .unwrap_or_default();
                        if let Err(e) = cache_store
                            .set_json(
                                KUCO_CACHE_TABLE.to_owned(),
                                cont_cache_key,
                                &container_names,
                            )
                            .await
                        {
                            tracing::error!(
                                "Stage 2: Failed to cache containers for {}/{}: {}",
                                ns_name,
                                pod_name,
                                e
                            );
                        }
                    }
                    Err(e) => tracing::error!(
                        "Stage 2: Failed to get pod details for {}/{}: {}",
                        ns_name,
                        pod_name,
                        e
                    ),
                }
                tokio::time::sleep(Duration::from_millis(100)).await; // Rate limit
            }
        }
    }
}

pub async fn periodic_multistage_cache_sync<S: KucoSqliteStore + Clone + 'static>(
    kube_ctx_clone: KubeContext,
    cache_store: S,
) {
    let kube_client = kube_ctx_clone
        .client
        .as_ref()
        .expect("Kube client not initialized")
        .clone();
    let mut stage1_ticker = tokio::time::interval(Duration::from_secs(10)); // Namespace/Pod names
    let mut stage2_ticker = tokio::time::interval(Duration::from_secs(20)); // Slower full detail scan
    let mut current_namespaces_for_stage2: Vec<String> = Vec::new();
    let mut stage2_ns_index = 0;

    tracing::info!("Periodic K8s sync task started (Staged).");

    loop {
        tokio::select! {
            _ = stage1_ticker.tick() => {
                match run_stage1_sync(&kube_client, &cache_store).await {
                    Ok(ns_names) => current_namespaces_for_stage2 = ns_names, // Update list for Stage 2
                    Err(e) => tracing::error!("Stage 1 Sync failed: {:?}", e),
                }
            }
            _ = stage2_ticker.tick() => {
                if !current_namespaces_for_stage2.is_empty() {
                    // Process one namespace per Stage 2 tick to spread the load
                    let ns_to_process = current_namespaces_for_stage2[stage2_ns_index].clone();
                    tracing::info!("Running Stage 2 Sync: Container Details for namespace '{}'", ns_to_process);
                    run_stage2_sync_for_namespace(&kube_client, &cache_store, &ns_to_process).await;
                    stage2_ns_index = (stage2_ns_index + 1) % current_namespaces_for_stage2.len(); // Cycle through namespaces
                     tracing::info!("Finished Stage 2 Sync for namespace '{}'", ns_to_process);
                } else {
                    tracing::info!("Stage 2: No namespaces found from Stage 1 to process yet.");
                }
            }
        }
    }
}

