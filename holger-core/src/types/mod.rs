use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ArtifactId {
    pub namespace: Option<String>,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactFormat {
    Maven3,
    Pip,
    Rust,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Znippy,
    Rocksdb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEndpoint {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: StorageType,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Rust,
    Pip,
    Maven3,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposedEndpoint {
    pub name: String,
    pub url_prefix: String,
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InOut {
    pub storage_backend: String,
    pub exposed_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: RepositoryType,
    #[serde(default)]
    pub r#in: Option<InOut>,
    pub out: Option<InOut>,
    #[serde(default)]
    pub upstreams: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolgerConfig {
    pub exposed_endpoints: Vec<ExposedEndpoint>,
    pub storage_endpoints: Vec<StorageEndpoint>,
    pub repositories: Vec<Repository>,
}
