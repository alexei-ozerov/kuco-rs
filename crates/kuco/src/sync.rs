use kuco_k8s_backend::{context::KubeContext, namespaces::NamespaceData, pods::PodData};
use kuco_sqlite_backend::KucoSqliteStore;

use chrono::Utc;
use std::time::Duration;
use tokio::time::interval;

pub async fn periodic_kubernetes_to_cache_sync<S: KucoSqliteStore + Clone + 'static>(
    kube_ctx_clone: KubeContext,
    cache_store: S,
) {
    let cache_table = "kv_cache".to_string();
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
                cache_table.clone(),
                "all_namespaces".to_string(),
                &ns_data_fetcher.names,
            )
            .await
        {
            tracing::error!("Failed to cache namespace names (sqlx): {}", e);
        }

        // Write Pod Info
        for ns_name in &ns_data_fetcher.names {
            let mut pod_data_fetcher = PodData::default();
            if let Err(e) = pod_data_fetcher
                .update_all(kube_client.clone(), ns_name)
                .await
            {
                tracing::error!(
                    "Failed to fetch pods for namespace {} (sqlx): {}",
                    ns_name,
                    e
                );
                continue;
            }
            for pod_info in &pod_data_fetcher.list {
                let pod_key = format!("{}-{}", ns_name, pod_info.name);
                if let Err(e) = cache_store
                    .set_json(cache_table.clone(), pod_key, pod_info)
                    .await
                {
                    tracing::error!(
                        "Failed to cache pod info for {} (sqlx): {}",
                        pod_info.name,
                        e
                    );
                }
            }
        }

        let current_timestamp_seconds: i64 = Utc::now().timestamp();
        let timestamp_key = "last_refreshed_at".to_string();
        if let Err(e) = cache_store
            .set_json(
                cache_table.clone(),
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
