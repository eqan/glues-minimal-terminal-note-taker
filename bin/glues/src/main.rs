use {color_eyre::Result, glues_tui::cli};

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}
