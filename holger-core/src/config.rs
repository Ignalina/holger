use crate::repository::rust::RustRepo;
use crate::repository::types::IOInstance;
use std::fs;
use std::path::Path;

use crate::exposed::{ExposedEndpointBackend, ExposedEndpointInstance};
use url::Url;
use crate::{
    ArtifactFormat, HolgerConfig, RepositoryInstance, RepositoryBackend, StorageEndpointInstance,
    RepositoryType, StorageType,
};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::exposed::http2::Http2Backend;

pub fn factory(config: &HolgerConfig) -> Result<HolgerInstance> {
    // 1. Build StorageEndpointInstances
    let mut storage_map: HashMap<String, Arc<StorageEndpointInstance>> = HashMap::new();
    let mut storage_endpoints = Vec::new();

    for se in &config.storage_endpoints {
        let instance = Arc::new(StorageEndpointInstance::from_config(se)?);
        storage_map.insert(se.name.clone(), instance.clone());
        storage_endpoints.push(instance);
    }

    // 2. Prepare resolver closures for storage
    let resolve_storage = |name: &str| {
        storage_map
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Storage endpoint '{}' not found", name))
    };

    // 3. Aggregate repositories and routes (endpoint resolver is a dummy for now)
    let dummy_endpoint_resolver = |_name: &str| -> anyhow::Result<Arc<ExposedEndpointInstance>> {
        Err(anyhow::anyhow!(
            "Endpoint lookup used before endpoints were constructed"
        ))
    };

    let (repositories, mut endpoint_routes) =
        aggregate_routes(config, &resolve_storage, &dummy_endpoint_resolver)?;

    // 4. Build ExposedEndpointInstances with their aggregated routes
    let mut endpoint_map: HashMap<String, Arc<ExposedEndpointInstance>> = HashMap::new();
    let mut exposed_endpoints = Vec::new();

    for ep in &config.exposed_endpoints {
        let backend: Arc<Http2Backend> = Http2Backend::from_config(
            ep.name.clone(),
            &ep.url_prefix,
            &ep.cert,
            &ep.key,
        )?;

        let backend_arc: Arc<dyn ExposedEndpointBackend> = backend.clone();
        let (ip, port) = parse_ip_port(&ep.url_prefix);

        let routes = endpoint_routes
            .remove(&ep.name)
            .unwrap_or_default()
            .into_iter()
            .map(|repo| (repo.name.clone(), repo))
            .collect();

        let instance = Arc::new(ExposedEndpointInstance {
            name: ep.name.clone(),
            ip,
            port,
            routes,
            backend: backend_arc,
        });

        endpoint_map.insert(ep.name.clone(), instance.clone());
        exposed_endpoints.push(instance);
    }

    Ok(HolgerInstance {
        exposed_endpoints,
        storage_endpoints,
        repositories,
    })
}

pub fn factory2(config: &HolgerConfig) -> Result<HolgerInstance> {
    // 1. Build StorageEndpointInstances
    let mut storage_map: HashMap<String, Arc<StorageEndpointInstance>> = HashMap::new();
    let mut storage_endpoints = Vec::new();

    for se in &config.storage_endpoints {
        let instance = Arc::new(StorageEndpointInstance::from_config(se)?);
        storage_map.insert(se.name.clone(), instance.clone());
        storage_endpoints.push(instance);
    }



    // 3. Prepare resolver closures
    let resolve_storage = |name: &str| {
        storage_map
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Storage endpoint '{}' not found", name))
    };

    let resolve_endpoint = |name: &str| {
        endpoint_map
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Exposed endpoint '{}' not found", name))
    };

    // 4. Build RepositoryInstances
    // 4. Build repositories and aggregate routes
    let (repositories, mut endpoint_routes) =
        aggregate_routes(config, &resolve_storage, &resolve_endpoint)?;

    // 5. Wire up endpoints with routes
    for ep in &config.exposed_endpoints {
        let backend: Arc<Http2Backend> = Http2Backend::from_config(
            ep.name.clone(),
            &ep.url_prefix,
            &ep.cert,
            &ep.key,
        )?;

        let backend_arc: Arc<dyn ExposedEndpointBackend> = backend.clone();
        let (ip, port) = parse_ip_port(&ep.url_prefix);

        // Pull routes for this endpoint
        let routes = endpoint_routes
            .remove(&ep.name)
            .unwrap_or_default()
            .into_iter()
            .map(|repo| (repo.name.clone(), repo))
            .collect();

        let instance = Arc::new(ExposedEndpointInstance {
            name: ep.name.clone(),
            ip,
            port,
            routes,
            backend: backend_arc,
        });

        endpoint_map.insert(ep.name.clone(), instance.clone());
        exposed_endpoints.push(instance);
    }

    Ok(HolgerInstance {
        exposed_endpoints,
        storage_endpoints,
        repositories,
    })
}
/// Inline helper to parse IP + port from URL
fn parse_ip_port(url: &str) -> (String, u16) {
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
            .map(|ep| ep.backend.as_ref()) // just map, no filter_map
            .try_for_each(|backend| backend.start())?;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.exposed_endpoints
            .iter()
            .map(|ep| ep.backend.as_ref())
            .try_for_each(|backend| backend.stop())?;
        Ok(())
    }
}

/// Aggregate repository routes per endpoint name
fn aggregate_routes(
    config: &HolgerConfig,
    resolve_storage: &impl Fn(&str) -> anyhow::Result<Arc<StorageEndpointInstance>>,
    resolve_endpoint: &impl Fn(&str) -> anyhow::Result<Arc<ExposedEndpointInstance>>,
) -> anyhow::Result<(Vec<Arc<RepositoryInstance>>, HashMap<String, Vec<Arc<RepositoryInstance>>>)> {
    let mut repositories = Vec::new();
    let mut endpoint_routes: HashMap<String, Vec<Arc<RepositoryInstance>>> = HashMap::new();

    for r in &config.repositories {
        let repo = Arc::new(RepositoryInstance::from_config(
            r,
            resolve_storage,
            resolve_endpoint,
        )?);

        if let Some(out_io) = &repo.out_io {
            endpoint_routes
                .entry(out_io.endpoint.name.clone())
                .or_default()
                .push(repo.clone());
        }

        repositories.push(repo);
    }

    Ok((repositories, endpoint_routes))
}

