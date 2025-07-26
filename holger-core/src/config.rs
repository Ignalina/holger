use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::{RepositoryType, StorageBackendType};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolgerConfig {
    pub exposed_endpoints: Vec<EndpointConfig>,
    pub storage_endpoints: Vec<StorageBackendConfig>,
    pub repositories: Vec<RepositoryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointConfig {
    pub name: String,
    pub url_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StorageBackendConfig {
    pub name: String,
    pub r#type: StorageBackendType,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryConfig {
    pub name: String,
    pub r#type: RepositoryType,
    pub in_: Option<RepositoryIO>,
    pub out: RepositoryIO,
    pub upstreams: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryIO {
    pub storage_backend: String,
    pub endpoints: Vec<String>,
}
