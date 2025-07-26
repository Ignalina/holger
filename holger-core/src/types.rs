use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub struct ArtifactId {
    pub namespace: Option<String>,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactFormat {
    Maven3,
    Pip,
    Rust,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Znippy,
    Rocksdb,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StorageLocation {
    Local,
    S3,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StorageConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: StorageType,
    pub location: StorageLocation,
    pub path: String,
    pub supports_random_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Rust,
    Pip,
    Maven3,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Endpoint {
    pub name: String,
    pub url_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InOutConfig {
    pub storage_backend: String,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: RepositoryType,
    pub accept_unpublished: bool,
    #[serde(default)]
    pub in_: Option<InOutConfig>,
    pub out: InOutConfig,
    #[serde(default)]
    pub upstreams: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HolgerConfig {
    pub exposed_endpoints: Vec<Endpoint>,
    pub storage_endpoints: Vec<StorageConfig>,
    pub repositories: Vec<RepositoryConfig>,
}
