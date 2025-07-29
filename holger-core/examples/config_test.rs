use std::path::Path;
use holger_core::registry::load_registry;

fn main() {
    let config_path = Path::new("holger-core/tests/prod.toml");

    match load_registry(config_path) {
        Ok(repos) => {
            println!("✅ Registry laddad.");
            println!("Repositories: {}", repos.len());

            for repo in repos {
                println!("  - {}", repo.name()); // kräver att traiten har `.name()`
            }
        }
        Err(e) => {
            eprintln!("❌ Fel vid registry-laddning: {e}");
            std::process::exit(1);
        }
    }
}
