use std::path::Path;
use holger_core::config::factory;
use holger_core::load_config_from_path;

fn main() -> anyhow::Result<()> {
    // 1. Load the config file
    println!("{}", std::env::current_dir().unwrap().display());
    let config = load_config_from_path("holger-core/tests/prod.toml")?;

    // 2. Build HolgerInstance
    let instance = factory(config)?;

    // 3. For debugging
    println!("Holger instance initialized with {} repositories", instance.repositories.len());

    Ok(())
}