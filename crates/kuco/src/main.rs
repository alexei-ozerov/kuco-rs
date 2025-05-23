use kuco::app::Kuco;
use kuco::sync::periodic_kubernetes_to_cache_sync;
use kuco::tracing::init_tracing;

use kuco_k8s_backend::context::KubeContext;
use kuco_sqlite_backend::cache::SqliteCache;

use color_eyre::eyre::{Result, WrapErr};

#[tokio::main]
async fn main() -> Result<()> {
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

    // Init Sqlite in-memory cache
    let sqlite_cache = SqliteCache::new_in_memory()
        .await
        .wrap_err("SQLx cache init failed")?;
    tracing::info!("In-memory SQLx cache initialized.");

    // Clone contexts to send to secondary thread
    let kube_context_for_task = kube_context.clone();
    let sqlite_cache_for_task = sqlite_cache.clone();

    // Secondary thread for syncing kube data to cache
    tokio::spawn(periodic_kubernetes_to_cache_sync(
        kube_context_for_task,
        sqlite_cache_for_task,
    ));
    tracing::info!("Periodic K8s data sync task (using SQLx) spawned.");

    // Run TUI
    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
