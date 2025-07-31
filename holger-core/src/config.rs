use crate::repository::rust::RustRepo;
use crate::repository::types::IOInstance;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::exposed::http2::Http2Backend;

use crate::exposed::ExposedEndpointInstance;
use std::collections::HashMap;

use crate::{
    ArtifactFormat, HolgerConfig, RepositoryInstance, RepositoryBackend, StorageEndpointInstance,
    RepositoryType, StorageType,
};

pub fn factory(config: HolgerConfig) -> Result<HolgerInstance> {
    // 1. Build storage endpoints
    let storage_map: HashMap<String, Arc<StorageEndpointInstance>> = config
        .storage_endpoints
        .iter()
        .map(|s| {
            let inst = match s.ty {
                StorageType::Znippy => StorageEndpointInstance::Znippy { path: s.path.clone().into() },
                StorageType::Rocksdb => StorageEndpointInstance::Rocksdb { path: s.path.clone().into() },
            };
            (s.name.clone(), Arc::new(inst))
        })
        .collect();

    // 2. Build exposed endpoints
    let exposed_map: HashMap<String, Arc<ExposedEndpointInstance>> = config
        .exposed_endpoints
        .iter()
        .map(|e| {
            (
                e.name.clone(),
                Arc::new(ExposedEndpointInstance::new(
                    e.name.clone(),
                    e.ip.clone(),
                    e.port,
                )),
            )
        })
        .collect();

    // 3. Build repository instances
    let mut repositories: Vec<Arc<RepositoryInstance>> = Vec::new();
    for r in config.repositories {
        // Create IOInstance for `in` if present
        let in_io = r.r#in.as_ref().and_then(|in_cfg| {
            let storage = storage_map.get(&in_cfg.storage_backend)?;
            let endpoint = exposed_map.get(&in_cfg.exposed_endpoint)?;
            Some(IOInstance {
                storage: storage.clone(),
                endpoint: endpoint.clone(),
            })
        });

        // Create IOInstance for `out` if present
        let out_io = r.out.as_ref().and_then(|out_cfg| {
            let storage = storage_map.get(&out_cfg.storage_backend)?;
            let endpoint = exposed_map.get(&out_cfg.exposed_endpoint)?;
            Some(IOInstance {
                storage: storage.clone(),
                endpoint: endpoint.clone(),
            })
        });

        // Optional backend depending on type
        let backend: Option<Arc<dyn RepositoryBackend>> = match r.ty {
            RepositoryType::Rust => {
                if let Some(out_io_ref) = &out_io {
                    Some(Arc::new(RustRepo {
                        name: r.name.clone(),
                        in_backend: in_io.as_ref().map(|io| (*io.storage).clone()),
                        out_backend: (*out_io_ref.storage).clone(),
                    }))
                } else {
                    None
                }
            }
            _ => None,
        };

        repositories.push(Arc::new(RepositoryInstance {
            name: r.name.clone(),
            format: match r.ty {
                RepositoryType::Maven3 => ArtifactFormat::Maven3,
                RepositoryType::Pip => ArtifactFormat::Pip,
                RepositoryType::Rust => ArtifactFormat::Rust,
                RepositoryType::Raw => ArtifactFormat::Raw,
            },
            repo_type: r.ty,
            in_io,
            out_io,
            upstreams: r.upstreams.clone(),
            backend,
        }));
    }

    Ok(HolgerInstance {
        exposed_endpoints: exposed_map.values().cloned().collect(),
        storage_endpoints: storage_map.values().cloned().collect(),
        repositories,
    })
}


pub fn load_config_from_path<P: AsRef<Path>>(path: P) -> Result<HolgerConfig> {
    let data = fs::read_to_string(path)?;
    let config: HolgerConfig = toml::from_str(&data)?;
    Ok(config)
}

#[derive(Debug)]
pub struct HolgerInstance {
    pub exposed_endpoints: Vec<Arc<ExposedEndpointInstance>>,
    pub storage_endpoints: Vec<Arc<StorageEndpointInstance>>,
    pub repositories: Vec<Arc<RepositoryInstance>>,
}

impl HolgerInstance {
    pub fn start(&self) -> anyhow::Result<()> {
        self.exposed_endpoints
            .iter()
            .filter_map(|ep| ep.backend.as_ref())
            .try_for_each(|backend| backend.start())?;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.exposed_endpoints
            .iter()
            .filter_map(|ep| ep.backend.as_ref())
            .try_for_each(|backend| backend.stop())?;
        Ok(())
    }
}
