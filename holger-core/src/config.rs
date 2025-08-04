use std::fs;
use std::path::Path;
use crate::exposed::{ExposedEndpointBackend, ExposedEndpointInstance};
use crate::{HolgerConfig, RepositoryBackend, RepositoryInstance, StorageEndpointInstance};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::exposed::fast_routes::FastRoutes;



pub fn factory(config: &HolgerConfig) -> Result<HolgerInstance> {
    // 1. Storage endpoints
    let storage_endpoints: Vec<StorageEndpointInstance> = config
        .storage_endpoints
        .iter()
        .map(|se| StorageEndpointInstance::from_config(se))
        .collect::<Result<_, _>>()?;

    // 2. Exposed endpoints
    let mut exposed_endpoints: Vec<ExposedEndpointInstance> = config
        .exposed_endpoints
        .iter()
        .map(|ee| ExposedEndpointInstance::from_config(ee))
        .collect::<Result<_, _>>()?;

    // 3. Repository instances
    let repositories: Vec<RepositoryInstance> = config
        .repositories
        .iter()
        .map(|repo_cfg| RepositoryInstance::from_config(repo_cfg))
        .collect::<Result<_, _>>()?;

    // 3b. WIRE: Link RepositoryInstances -> ExposedEndpointInstances
    let mut repositories = repositories; // make mutable
    for repo in &mut repositories {
        if let Some(io) = repo.out_io.as_mut() {
            if let Some(name) = &io.exposed_name {
                if let Some(endpoint) = exposed_endpoints.iter().find(|ep| ep.name == *name) {
                    io.exposed = Some(endpoint.clone());
                }
            }
        }
    }
    // 4. Build FastRoutes for each exposed endpoint
    for endpoint in &mut exposed_endpoints {
        let routes: Vec<(String, Arc<dyn RepositoryBackend>)> = repositories
            .iter()
            .filter_map(|repo| {
                let name= repo.exposed_endpoint_name();
                println!("LHS={:?}, RHS={:?}", name, endpoint.name);
                if name == endpoint.name {
                    repo.backend()
                        .map(|backend| (repo.name.clone(), backend))
                } else {
                    None
                }
            })
            .collect();

        endpoint.set_fast_routes(FastRoutes::new(routes));
    }

    Ok(HolgerInstance {
        exposed_endpoints,
        storage_endpoints,
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
    pub exposed_endpoints: Vec<ExposedEndpointInstance>,
    pub storage_endpoints: Vec<StorageEndpointInstance>,
    pub repositories: Vec<RepositoryInstance>,
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

pub(crate) fn parse_ip_port(url: &str) -> (String, u16) {
    let clean = url.trim_end_matches('/');
    let without_scheme = clean.split("://").nth(1).unwrap_or(clean);
    let mut parts = without_scheme.split(':');
    let ip = parts.next().unwrap_or("127.0.0.1").to_string();
    let port = parts.next().and_then(|p| p.parse().ok()).unwrap_or(443);
    (ip, port)
}

