// znippy-cli/src/main.rs

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use holger_core::config::factory;
use holger_core::load_config_from_path;

#[derive(Parser)]
#[command(name = "holger")]
#[command(about = "Holger: Guards your artifacts at rest.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Holger
    Start {
        #[arg(short, long)]
        config: PathBuf,

    },

}

pub fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config} => {
            println!("Starting Holger {:?}",config);

            let config = load_config_from_path(config)?;
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
//            Ok(())
        }
    }

    Ok(())
}