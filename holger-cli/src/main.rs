

// znippy-cli/src/main.rs
use anyhow::Result;


#[tokio::main]
async fn main() -> Result<()> {
    holger_cli::run().await
}

