use kuco::app::Kuco;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = Kuco::new().await.run(terminal).await;

    ratatui::restore();

    result
}
