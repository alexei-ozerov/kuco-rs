use kuco::app::Kuco;
use kuco::tracing::init_tracing;

use kuco_cache::CacheStore;

// TODO: Implement a Redis Cache for storing data from K8s
// TODO: Use Sqlite for storing persistent data

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _guard = init_tracing()?;

    // Init Valkey Cache
    let mut cache_store = CacheStore::new();
    cache_store.create_client()?;
    cache_store.open_connection().await?;

    // Test
    cache_store.set("KEY", "VALUE").await?;

    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
