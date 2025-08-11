use tokio::main;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    holger_cli::run().await
}
