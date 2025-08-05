use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Rust,
    Pip,
    Maven3,
    Raw,
}
impl RepositoryType {
    pub fn endpoint_name(&self) -> &'static str {
        match self {
            RepositoryType::Rust => "rust",
            RepositoryType::Pip => "pip",
            RepositoryType::Maven3 => "maven3",
            RepositoryType::Raw => "raw",
        }
    }
}


#[async_trait]
pub trait RepositoryBackendTrait: Send + Sync {
    fn name(&self) -> &str;
    fn format(&self) -> ArtifactFormat;
    fn is_writable(&self) -> bool;

    fn fetch(&self, id: &ArtifactId) -> anyhow::Result<Option<Vec<u8>>>;
    fn put(&self, id: &ArtifactId, data: &[u8]) -> anyhow::Result<()>;


    fn fetch_many_with_upstreams(
        &self,
        upstreams: &[Arc<dyn RepositoryBackendTrait>],
        ids: &[ArtifactId],
    ) -> anyhow::Result<HashMap<ArtifactId, Vec<u8>>> {
        let mut result = HashMap::new();

        for id in ids {
            if let Some(data) = self.fetch(id)? {
                result.insert(id.clone(), data);
                continue;
            }
            for up in upstreams {
                if let Some(data) = up.fetch(id)? {
                    result.insert(id.clone(), data);
                    break;
                }
            }
        }
        Ok(result)
    }
    fn handle_http2_request(
        &self,
        suburl: &str,
        body: &[u8],
    ) -> anyhow::Result<(u16, Vec<(String, String)>, Vec<u8>)>;

}

