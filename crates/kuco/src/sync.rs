use kuco_k8s_backend::{
    context::KubeContext, namespaces::NamespaceData, pods::PodData,
};
use kuco_sqlite_backend::KucoSqliteStore;

use chrono::Utc;
use std::time::Duration;
use tokio::time::interval;

use crate::constants::KUCO_CACHE_TABLE;

pub async fn periodic_kubernetes_to_cache_sync<S: KucoSqliteStore + Clone + 'static>(
    kube_ctx_clone: KubeContext,
    cache_store: S,
) {
    let kube_client = kube_ctx_clone
        .client
        .as_ref()
        .expect("Kube client not initialized")
        .clone();

    let mut ticker = interval(Duration::from_secs(5));
    tracing::info!("Periodic K8s to SQLite Cache (sqlx) sync task started.");

    loop {
        ticker.tick().await;
        let mut ns_data_fetcher = NamespaceData::new();
        ns_data_fetcher.update(kube_client.clone()).await;

        // Write Namespace List
        if let Err(e) = cache_store
            .set_json(
                KUCO_CACHE_TABLE.to_owned().clone(),
                "all_namespaces".to_string(),
                &ns_data_fetcher.names,
            )
            .await
        {
            tracing::error!("Failed to cache namespace names (sqlx): {}", e);
        }

        // TODO: Optimize this :3
        // Write Pod Names
        for ns_name in &ns_data_fetcher.names {
            let mut pod_data_fetcher = PodData::default();
            match pod_data_fetcher
                .get_names(kube_client.clone(), ns_name)
                .await
            {
                Ok(_) => {
                    tracing::debug!("Successfully fetched pod names for {}.", ns_name.clone())
                }
                Err(_) => {
                    tracing::error!("Failed to cache pod names for {}.", ns_name.clone())
                }
            };
            let pods_in_ns_vec = pod_data_fetcher.names;

            let pod_table_name = format!("pods_{}", ns_name.clone());
            if let Err(e) = cache_store
                .set_json(
                    KUCO_CACHE_TABLE.to_owned().clone(),
                    pod_table_name,
                    &serde_json::json!(pods_in_ns_vec),
                )
                .await
            {
                tracing::error!(
                    "Failed to cache pod names for {} (sqlx): {}",
                    ns_name.clone(),
                    e
                );
            }

            // TODO: Find a way to implement this that is not absurdly slow.
            // Get Containers Per Pod
            // let mut container_data_fetcher = ContainerData::default();
            // for po in pods_in_ns_vec {
            //     match container_data_fetcher
            //         .update(kube_client.clone(), &ns_name, &po)
            //         .await
            //     {
            //         Ok(_) => {
            //             tracing::debug!("Successfully fetched pod names for {}.", ns_name.clone())
            //         }
            //         Err(_) => {
            //             tracing::error!("Failed to cache pod names for {}.", ns_name.clone())
            //         }
            //     };
            //
            //     let container_names_vec = container_data_fetcher.clone().names;
            //     let cont_table_name = format!("co_{}_{}", ns_name.clone(), po);
            //     if let Err(e) = cache_store
            //         .set_json(
            //             KUCO_CACHE_TABLE.to_owned().clone(),
            //             cont_table_name,
            //             &serde_json::json!(container_names_vec),
            //         )
            //             .await
            //     {
            //         tracing::error!(
            //             "Failed to cache pod names for {} (sqlx): {}",
            //             ns_name.clone(),
            //             e
            //         );
            //     }
            // }
        }

        // Write last update timestamp
        let current_timestamp_seconds: i64 = Utc::now().timestamp();
        let timestamp_key = "last_refreshed_at".to_string();
        if let Err(e) = cache_store
            .set_json(
                KUCO_CACHE_TABLE.to_owned().clone(),
                timestamp_key,
                &current_timestamp_seconds,
            )
            .await
        {
            tracing::error!(
                "Failed to set last_refreshed_at timestamp in cache (sqlx): {}",
                e
            );
        }
        tracing::info!("Periodic task (sqlx): Data fetch and cache cycle complete.");
    }
}
