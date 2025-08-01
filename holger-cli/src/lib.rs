// znippy-cli/src/main.rs

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;


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

        #[arg(short, long)]
        cert: PathBuf,

        #[arg(short, long)]
        key: PathBuf,

    },

}

pub fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, cert, key } => {
            !todo!()
        }
    }

    Ok(())
}