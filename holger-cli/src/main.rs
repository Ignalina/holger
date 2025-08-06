

// znippy-cli/src/main.rs
use ron::ser::{to_string_pretty, PrettyConfig};

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use holger_ron::{wire_holger, Holger};
//use holger_core::config::factory;
//use holger_core::load_config_from_path;

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
#[tokio::main]
pub async fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config} => {
            println!("Starting Holger {:?}",config);

            let mut holger = holger_ron::read_ron_config(config)?;
            let backend=holger.instantiate_backends();

            let r=wire_holger(&mut holger);
            print_wiring_summary(&holger);
            let cfg = PrettyConfig::new()
                .depth_limit(4)
                .separate_tuple_members(true)
                .enumerate_arrays(true);

            println!("{}", to_string_pretty(&holger, cfg)?);


            holger.start()?;

            println!("Holger is running. Press Ctrl+C to stop.");

            // ✅ Block until Ctrl+C
            let (tx, rx) = std::sync::mpsc::channel();
            ctrlc::set_handler(move || {
                let _ = tx.send(());
            })?;

            // Wait for signal
            rx.recv().expect("Failed to receive signal");

            //            holger.stop()?;
            println!("Holger stopped.");
            //            Ok(())
        }
    }

    Ok(())
}
fn main() {
    run().unwrap();
}

pub fn print_wiring_summary(holger: &Holger) {
    println!("--- Wiring Summary ---");

    for (i, repo) in holger.repositories.iter().enumerate() {
        let upstreams = repo.ron_upstreams.len();
        let in_exists = repo.ron_in.is_some();
        let out_exists = repo.ron_out.is_some();

        println!(
            "[Repo #{i}] {} | upstreams:{} in:{} out:{}",
            repo.ron_name, upstreams, in_exists, out_exists
        );
    }

    println!("--- Wiring Check ---");
    let mut ok = true;
    for repo in &holger.repositories {
        if repo.ron_out.is_some() && holger.exposed_endpoints.is_empty() {
            println!("!! Repo {} has `ron_out` but no exposed endpoints wired", repo.ron_name);
            ok = false;
        }
        if repo.ron_out.is_some() && holger.storage_endpoints.is_empty() {
            println!("!! Repo {} has `ron_out` but no storage endpoints wired", repo.ron_name);
            ok = false;
        }
    }

    if ok {
        println!("Wiring looks OK.");
    } else {
        println!("Wiring has issues.");
    }


    println!("--- Reverse Wiring ---");

    for (i, ep) in holger.exposed_endpoints.iter().enumerate() {
        println!(
            "[Exposed #{i}] {} | in:{} out:{}",
            ep.ron_name,
            ep.wired_in_repositories.len(),
            ep.wired_out_repositories.len()
        );
    }

    for (i, st) in holger.storage_endpoints.iter().enumerate() {
        println!(
            "[Storage #{i}] {} | in:{} out:{}",
            st.ron_name,
            st.wired_in_repositories.len(),
            st.wired_out_repositories.len()
        );
    }
    println!("--- Aggregated Routes Debug ---");

    for (i, ep) in holger.exposed_endpoints.iter().enumerate() {
        println!(
            "[Exposed #{i}] {} | aggregated_routes: {}",
            ep.ron_name,
            if ep.aggregated_routes.is_some() { "yes" } else { "no" }
        );

        if !ep.wired_out_repositories.is_empty() {
            for &repo_ptr in &ep.wired_out_repositories {
                if repo_ptr.is_null() {
                    println!("    -> null pointer (skipped)");
                    continue;
                }
                let repo = unsafe { &*repo_ptr };
                println!(
                    "    -> wired repo: {} | backend: {}",
                    repo.ron_name,
                    if repo.backend_repository.is_some() { "ready" } else { "none" }
                );
            }
        } else {
            println!("    -> no wired out repositories");
        }
    }

    println!("--- Aggregated Routes Deep Check ---");

    for (i, ep) in holger.exposed_endpoints.iter().enumerate() {
        let mut possible_routes = 0;
        let mut ready_routes = 0;

        println!(
            "[Exposed #{i}] {} | aggregated_routes: {}",
            ep.ron_name,
            if ep.aggregated_routes.is_some() { "yes" } else { "no" }
        );

        if ep.wired_out_repositories.is_empty() {
            println!("    -> no wired out repositories");
        } else {
            for &repo_ptr in &ep.wired_out_repositories {
                if repo_ptr.is_null() {
                    println!("    -> null pointer (skipped)");
                    continue;
                }
                let repo = unsafe { &*repo_ptr };
                possible_routes += 1;

                let expected = &ep.ron_name;
                let actual = if let Some(io) = &repo.ron_out {
                    &io.ron_exposed_endpoint
                } else {
                    "<none>"
                };

                let backend_ready = repo.backend_repository.is_some();
                if backend_ready {
                    ready_routes += 1;
                }

                println!(
                    "    -> wired repo: {} | backend:{} | expected:{} vs actual:{}",
                    repo.ron_name,
                    if backend_ready { "ready" } else { "none" },
                    expected,
                    actual
                );
            }
        }

        println!(
            "    -> route candidates: {} | ready for FastRoutes: {}",
            possible_routes, ready_routes
        );
    }
}
