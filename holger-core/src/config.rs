use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::types::HolgerConfig;

use serde::Deserialize;
use crate::repository::ExposedEndpointInstance;
use crate::repository::types::RepositoryInstance;
use crate::{RepositoryBackend, StorageEndpointInstance};

pub fn factory(config:HolgerConfig) -> Result<HolgerInstance>  {
    Ok(HolgerInstance {
        exposed_endpoints: Vec::new(),
        storage_endpoints: Vec::new(),
        repositories: Vec::new(),
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
    pub repositories: Vec<Arc<RepositoryInstance>>
}



