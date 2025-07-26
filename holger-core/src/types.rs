use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactFormat {
    Java,
    Python,
    Rust,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum RepositoryType {
    #[serde(rename = "java")]
    Java(JavaRepoConfig),
    #[serde(rename = "python")]
    Python(PythonRepoConfig),
    #[serde(rename = "rust")]
    Rust(RustRepoConfig),
    #[serde(rename = "raw")]
    Raw(RawRepoConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct JavaRepoConfig {
    pub allow_snapshots: bool,
    pub group_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct PythonRepoConfig {
    pub allow_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct RustRepoConfig {
    pub accept_unpublished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct RawRepoConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum StorageBackendType {
    #[serde(rename = "znippy")]
    Znippy {
        path: String,
        #[serde(default = "default_true")]
        supports_random_read: bool,
    },
    #[serde(rename = "rocksdb")]
    RocksDb {
        path: String,
        #[serde(default = "default_false")]
        supports_random_read: bool,
    },
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        prefix: Option<String>,
        #[serde(default = "default_true")]
        supports_random_read: bool,
    },
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
