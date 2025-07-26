use std::collections::HashMap;

use crate::config::{HolgerConfig, StorageBackendConfig};
use crate::storage::{ResolvedStorage, StorageKind};
use crate::repo::{RepositoryInstance};
use crate::types::{StorageBackendType};

pub struct RepositoryRegistry {
    pub repositories: HashMap<String, RepositoryInstance>,
    pub storage_backends: HashMap<String, ResolvedStorage>,
}

impl RepositoryRegistry {
    pub fn from_config(config: HolgerConfig) -> Self {
        // Förbered storage-backends
        let storage_backends: HashMap<String, ResolvedStorage> = config
            .storage_endpoints
            .into_iter()
            .map(|s| {
                let name = s.name.clone();
                let resolved = match s.r#type {
                    StorageBackendType::Znippy { path, supports_random_read } => ResolvedStorage {
                        name: name.clone(),
                        kind: StorageKind::Znippy { path },
                        supports_random_read,
                    },
                    StorageBackendType::RocksDb { path, supports_random_read } => ResolvedStorage {
                        name: name.clone(),
                        kind: StorageKind::RocksDb { path },
                        supports_random_read,
                    },
                    StorageBackendType::S3 { bucket, prefix, supports_random_read } => ResolvedStorage {
                        name: name.clone(),
                        kind: StorageKind::S3 { bucket, prefix },
                        supports_random_read,
                    },
                };
                (name, resolved)
            })
            .collect();

        let resolve = |name: &str| -> ResolvedStorage {
            storage_backends
                .get(name)
                .expect(&format!("Storage backend '{}' not found", name))
                .clone()
        };

        let repositories: HashMap<String, RepositoryInstance> = config
            .repositories
            .iter()
            .map(|cfg| {
                let instance = RepositoryInstance::from_config(cfg, &resolve);
                (cfg.name.clone(), instance)
            })
            .collect();

        RepositoryRegistry {
            repositories,
            storage_backends,
        }
    }

    pub fn get_repo(&self, name: &str) -> Option<&RepositoryInstance> {
        self.repositories.get(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.repositories.keys().cloned().collect()
    }
}
