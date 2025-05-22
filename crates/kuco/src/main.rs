use kuco::app::Kuco;
use kuco::sync::kube_to_valkey_cache_sync;
use kuco::tracing::init_tracing;

use kuco_cache::CacheStore;
use kuco_k8s_backend::context::KubeContext;

// TODO: Implement a Redis Cache for storing data from K8s
// TODO: Use Sqlite for storing persistent data

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Init Tracing
    color_eyre::install()?;
    let _guard = init_tracing()?;

    // Create KubeContext
    let mut kube_context = KubeContext::default();
    kube_context.init_context().await.map_err(|e| {
        tracing::error!("Failed to initialize Kubernetes context: {}", e);
        color_eyre::eyre::eyre!("K8s context init failed: {}", e)
    })?;
    tracing::info!("Kubernetes context initialized.");

    // Init Valkey Client & Connection
    let mut cache_store = CacheStore::new();
    cache_store.create_client().map_err(|e| {
        tracing::error!("Failed to create Valkey client: {}", e);
        e
    })?;
    cache_store.open_connection().await.map_err(|e| {
        tracing::error!("Failed to open Valkey connection: {}", e);
        e
    })?;
    tracing::info!("CacheStore initialized and connection opened.");

    // Clone Arc (for running sync tasks on another thread)
    let kube_context_for_task = kube_context.clone();
    let cache_store_for_task = cache_store.clone();

    // Spawn second thread
    tokio::spawn(kube_to_valkey_cache_sync(
        kube_context_for_task,
        cache_store_for_task,
    ));
    tracing::info!("K8s data sync task spawned.");

    // Run TUI
    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
