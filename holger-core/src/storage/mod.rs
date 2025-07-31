use crate::types::{StorageEndpoint, StorageType};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Represents a fully resolved storage backend.
#[derive(Debug, Clone)]
pub enum StorageEndpointInstance {
    Znippy { path: PathBuf },
    Rocksdb { path: PathBuf },
}

impl StorageEndpointInstance {
    pub fn from_config(config: &StorageEndpoint) -> Result<Self> {
        match config.ty {
            StorageType::Znippy => Ok(StorageEndpointInstance::Znippy {
                path: PathBuf::from(&config.path),
            }),
            StorageType::Rocksdb => Ok(StorageEndpointInstance::Rocksdb {
                path: PathBuf::from(&config.path),
            }),
        }
    }
}
