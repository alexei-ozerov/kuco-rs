use kuco_cache::CacheStore;
use kuco_k8s_backend::{context::KubeContext, namespaces::NamespaceData, pods::PodData};

use chrono::Utc;
use std::time::Duration;
use tokio::time::interval;

// Run a sync of kube data that KuCo tracks to Valkey
pub async fn kube_to_valkey_cache_sync(
    mut kube_ctx_clone: KubeContext,
    mut cache_store_clone: CacheStore,
) {
    let kube_client = match kube_ctx_clone.client.as_ref() {
        Some(c) => c.clone(),
        None => {
            if kube_ctx_clone.init_context().await.is_err() {
                tracing::error!(
                    "Periodic task: Failed to initialize Kubernetes client. Task cannot proceed."
                );
                return;
            }
            kube_ctx_clone.client.as_ref().unwrap().clone()
        }
    };

    let needs_cache_connection = {
        let guard = cache_store_clone.connection.arc.lock().unwrap();
        guard.is_none()
    };
    if needs_cache_connection {
        if let Err(e) = cache_store_clone.open_connection().await {
            tracing::error!(
                "Periodic task: Failed to open Valkey connection: {}. Task cannot proceed.",
                e
            );
            return;
        }
    }

    let mut ticker = interval(Duration::from_secs(5));
    tracing::info!("Periodic task: K8s to Valkey sync task started.");

    loop {
        ticker.tick().await;
        tracing::info!("Periodic task: Starting data fetch and cache cycle.");

        let mut ns_data_fetcher = NamespaceData::new();
        ns_data_fetcher.update(kube_client.clone()).await;
        tracing::debug!("Fetched {} namespaces.", ns_data_fetcher.names.len());

        match serde_json::to_string(&ns_data_fetcher.names) {
            Ok(ns_json) => {
                if let Err(e) = cache_store_clone.set("k8s:all_namespaces", ns_json).await {
                    tracing::error!("Failed to cache namespace names: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize namespace names: {}", e);
            }
        }

        for ns_name in &ns_data_fetcher.names {
            tracing::debug!("Fetching pods for namespace: {}", ns_name);
            let mut pod_data_fetcher = PodData::default();
            // Ensure this method exists, is async, and is part of your PodData struct.
            if let Err(e) = pod_data_fetcher
                .update_all(kube_client.clone(), ns_name)
                .await
            {
                tracing::error!("Failed to fetch pods for namespace {}: {}", ns_name, e);
                continue;
            }
            tracing::debug!(
                "Fetched {} pods in namespace '{}'.",
                pod_data_fetcher.list.len(),
                ns_name
            );

            for pod_info in &pod_data_fetcher.list {
                let pod_key = format!("k8s:ns:{}:pods:{}", ns_name, pod_info.name);
                match serde_json::to_string(pod_info) {
                    Ok(pod_json) => {
                        if let Err(e) = cache_store_clone.set(&pod_key, &pod_json).await {
                            tracing::error!("Failed to cache pod info for {}: {}", pod_key, e);
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to serialize pod info for {}: {}",
                            pod_info.name,
                            e
                        );
                    }
                }
            }
        }

        let current_timestamp_seconds: i64 = Utc::now().timestamp();
        let timestamp_key = "k8s:sync:last_refreshed_at";

        match cache_store_clone
            .set(timestamp_key, current_timestamp_seconds)
            .await
        {
            Ok(_) => {
                tracing::debug!(
                    "Successfully set last_refreshed_at timestamp: {}",
                    current_timestamp_seconds
                );
            }
            Err(e) => {
                tracing::error!("Failed to set last_refreshed_at timestamp in cache: {}", e);
            }
        }
        tracing::info!("Periodic task: Data fetch and cache cycle complete.");
    }
}
