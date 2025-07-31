use tokio::signal;
use std::path::Path;
use holger_core::config::factory;
use holger_core::load_config_from_path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config_from_path("holger-core/tests/prod.toml")?;
    let holger = factory(config)?;
    holger.start()?;

    println!("Holger is running. Press Ctrl+C to stop.");
    signal::ctrl_c().await?;
    holger.stop()?;
    Ok(())
}