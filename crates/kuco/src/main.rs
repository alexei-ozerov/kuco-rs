use kuco::app::Kuco;
use kuco::tracing::init_tracing;

use kuco_sqlite_backend::SqliteStore;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _guard = init_tracing()?;

    // Sqlite Store Init
    let _db_pool = SqliteStore::new().await;

    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
