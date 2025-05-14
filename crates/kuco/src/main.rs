use kuco::app::Kuco;
use kuco::tracing::init_tracing;

use tracing::info;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _guard = init_tracing()?;

    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
