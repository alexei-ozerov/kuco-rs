use kuco::app::Kuco;
use kuco::sync::periodic_kubernetes_to_cache_sync;
use kuco::tracing::init_tracing;

use kuco_k8s_backend::context::KubeContext;
use kuco_sqlite_backend::{SqliteCache, SqliteDb};

use color_eyre::eyre::{Result, WrapErr, eyre};
use std::{path::PathBuf, sync::Arc};

fn get_user_home() -> Option<PathBuf> {
    dirs_next::home_dir()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init Tracing
    color_eyre::install()?;
    let _guard = init_tracing()?;

    // Setup Data Persistence Arguments
    let db_connection_timeout: f64 = 30.0;
    let db_path: PathBuf = match get_user_home() {
        Some(home_path) => {
            let path = home_path.join(".kuco").join("user_kube_data.db");
            tracing::info!("Database path will be: {}", home_path.display());

            path
        }
        None => {
            let error_message = "Critical Error: Could not determine the home directory. Application cannot continue.";
            tracing::error!("{}", error_message);

            return Err(eyre!(error_message));
        }
    };

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
        .wrap_err("Sqlite cache init failed")?;
    tracing::info!("In-memory Sqlite cache initialized.");

    // Init Sqlite persistent storage
    let sqlite_db = SqliteDb::new(&db_path, db_connection_timeout)
        .await
        .wrap_err("Sqlite cache init failed")?;
    tracing::info!("Persistent Sqlite DB initialized.");

    // Clone contexts to send to secondary thread
    let kube_context_for_task = kube_context.clone();
    let sqlite_cache_for_task = sqlite_cache.clone();
    let _sqlite_db_for_task = sqlite_db.clone();

    // Secondary thread for syncing kube data to cache
    tokio::spawn(periodic_kubernetes_to_cache_sync(
        kube_context_for_task,
        sqlite_cache_for_task,
        // sqlite_db_for_task,
    ));
    tracing::info!("Periodic K8s data sync task (using SQLx) spawned.");

    let arc_sqlite_cache = Arc::new(sqlite_cache);
    let arc_sqlite_db = Arc::new(sqlite_db);

    // Run TUI
    let terminal = ratatui::init();
    let result = Kuco::new(arc_sqlite_cache, arc_sqlite_db)
        .await
        .run(terminal)
        .await;

    ratatui::restore();

    result
}
