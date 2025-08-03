use crate::repository::rust::RustRepo;
use crate::repository::types::IOInstance;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Context;
use crate::exposed::{ExposedEndpointBackend, ExposedEndpointInstance};
use url::Url;
use crate::{
    ArtifactFormat, HolgerConfig, RepositoryInstance, RepositoryBackend, StorageEndpointInstance,
    RepositoryType, StorageType,
};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use rustls::ServerConfig;
use crate::exposed::http2::{FastRoutes, Http2Backend};


use std::sync::atomic::AtomicBool;
use crate::{ExposedEndpoint, StorageEndpoint};


pub fn factory(config: &HolgerConfig) -> Result<HolgerInstance> {
    use std::collections::HashMap;
    use std::sync::Arc;

    // 1. Build StorageEndpointInstances
    let mut storage_map: HashMap<String, Arc<StorageEndpointInstance>> = HashMap::new();
    let mut storage_endpoints = Vec::new();

    for se in &config.storage_endpoints {
        let instance = Arc::new(StorageEndpointInstance::from_config(se)?);
        storage_map.insert(se.name.clone(), instance.clone());
        storage_endpoints.push(instance);
    }

    // 2. Build ExposedEndpointInstances
    let mut endpoint_map: HashMap<String, Arc<ExposedEndpointInstance>> = HashMap::new();
    let mut exposed_endpoints = Vec::new();

    for ee in &config.exposed_endpoints {
        let instance = Arc::new(ExposedEndpointInstance::from_config(ee)?);
        endpoint_map.insert(ee.name.clone(), instance.clone());
        exposed_endpoints.push(instance);
    }

    // 3. Pass 1: Instantiate repositories
    let mut repositories: Vec<Arc<RepositoryInstance>> = Vec::new();
    for repo_cfg in &config.repositories {
        let repo = Arc::new(RepositoryInstance::from_config(repo_cfg)?);
        repositories.push(repo);
    }

    // 4. Pass 2: Wire repositories to storage/endpoints
    for (i, repo_cfg) in config.repositories.iter().enumerate() {
        let mut_repo = Arc::get_mut(&mut repositories[i])
            .expect("Factory: repo Arc should be unique at this point");
        mut_repo.wire(repo_cfg, &storage_map, &endpoint_map)?;
    }

    Ok(HolgerInstance {
        exposed_endpoints,
        storage_endpoints,
        repositories,
    })
}


/// Inline helper to parse IP + port from URL
pub(crate) fn parse_ip_port(url: &str) -> (String, u16) {
    let clean = url.trim_end_matches('/');
    let without_scheme = clean.split("://").nth(1).unwrap_or(clean);
    let mut parts = without_scheme.split(':');
    let ip = parts.next().unwrap_or("127.0.0.1").to_string();
    let port = parts.next().and_then(|p| p.parse().ok()).unwrap_or(443);
    (ip, port)
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
            .filter_map(|ep| ep.backend.as_ref()) // take only Some
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

