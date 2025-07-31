use tokio::signal;
use std::path::Path;
use holger_core::config::factory;
use holger_core::load_config_from_path;

fn main() -> anyhow::Result<()> {
    let config = load_config_from_path("holger-core/tests/prod.toml")?;
    let holger = factory(&config)?;
    holger.start()?;

    println!("Holger is running. Press Ctrl+C to stop.");

    // ✅ Block until Ctrl+C
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })?;

    // Wait for signal
    rx.recv().expect("Failed to receive signal");

    holger.stop()?;
    println!("Holger stopped.");
    Ok(())
}