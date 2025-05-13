use kuco::app::KucoInterface;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = KucoInterface::new().await.run(terminal).await;

    ratatui::restore();

    result
}
