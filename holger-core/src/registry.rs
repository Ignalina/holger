use crate::config::load_config_from_path;
use crate::repo::{RepositoryBackend, RustRepo};
use crate::storage::{StorageEndpointInstance};
use crate::types::{HolgerConfig, RepositoryType};

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Load and instantiate all repositories from a given config file
pub fn load_registry(path: &Path) -> Result<Vec<Arc<dyn RepositoryBackend>>> {
    let config: HolgerConfig = load_config_from_path(path)?;

    // Step 1: Build map of storage backends
    let mut storage_map = HashMap::<String, StorageEndpointInstance>::new();
    for s in &config.storage_endpoints {
        let resolved = StorageEndpointInstance::from_config(s)?;
        storage_map.insert(s.name.clone(), resolved);
    }

    // Step 2: Instantiate all repositories
    let mut repo_instances: Vec<Arc<dyn RepositoryBackend>> = Vec::new();

    for r in &config.repositories {
        let in_backend = r.r#in.as_ref().map(|in_cfg| {
            storage_map
                .get(&in_cfg.storage_backend)
                .cloned()
                .ok_or_else(|| anyhow!("Unknown IN storage backend: {}", in_cfg.storage_backend))
        }).transpose()?;

        let out_backend = storage_map
            .get(&r.out.storage_backend)
            .cloned()
            .ok_or_else(|| anyhow!("Unknown OUT storage backend: {}", r.out.storage_backend))?;

        // Match repository type
        let repo: Arc<dyn RepositoryBackend> = match r.ty {
            RepositoryType::Rust => Arc::new(RustRepo {
                name: r.name.clone(),
                in_backend,
                out_backend,
            }),

            _ => return Err(anyhow!("Unsupported repository type: {:?}", r.ty)),
        };

        repo_instances.push(repo);
    }

    Ok(repo_instances)
}
