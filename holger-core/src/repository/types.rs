use derivative::Derivative;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::{ArtifactFormat, ArtifactId, Repository, RepositoryType, StorageEndpointInstance};
use crate::exposed::ExposedEndpointInstance;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct IOInstance {
    pub storage: Arc<StorageEndpointInstance>,
    pub endpoint: Arc<ExposedEndpointInstance>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RepositoryInstance {
    pub name: String,
    pub format: ArtifactFormat,
    pub repo_type: RepositoryType,
    pub in_io: Option<IOInstance>,
    pub out_io: Option<IOInstance>,
    pub upstreams: Vec<String>, // store names first; link phase later
    #[derivative(Debug = "ignore")]
    pub backend: Option<Arc<dyn RepositoryBackend>>
}

impl RepositoryInstance {
    pub fn from_config<F, G>(
        cfg: &Repository,
        resolve_storage: &F,
        resolve_endpoint: &G,
    ) -> anyhow::Result<Self>
    where
        F: Fn(&str) -> anyhow::Result<Arc<StorageEndpointInstance>>,
        G: Fn(&str) -> anyhow::Result<Arc<ExposedEndpointInstance>>,
    {
        let format = match cfg.ty {
            RepositoryType::Maven3 => ArtifactFormat::Maven3,
            RepositoryType::Pip => ArtifactFormat::Pip,
            RepositoryType::Rust => ArtifactFormat::Rust,
            RepositoryType::Raw => ArtifactFormat::Raw,
        };

        // Handle optional in_io
        let in_io = if let Some(ref in_cfg) = cfg.r#in {
            Some(IOInstance {
                storage: resolve_storage(&in_cfg.storage_backend)?,
                endpoint: resolve_endpoint(&in_cfg.exposed_endpoint)?,
            })
        } else {
            None
        };

        // Handle optional out_io
        let out_io = if let Some(ref out_cfg) = cfg.out {
            Some(IOInstance {
                storage: resolve_storage(&out_cfg.storage_backend)?,
                endpoint: resolve_endpoint(&out_cfg.exposed_endpoint)?,
            })
        } else {
            None
        };

        Ok(RepositoryInstance {
            name: cfg.name.clone(),
            format,
            repo_type: cfg.ty.clone(),
            in_io,
            out_io,
            upstreams: cfg.upstreams.clone(),
            backend: None,
        })
    }
}

/// Core trait for all repository types
#[async_trait]
pub trait RepositoryBackend: Send + Sync {
    fn name(&self) -> &str;
    fn format(&self) -> ArtifactFormat;
    fn is_writable(&self) -> bool;

    fn fetch(&self, id: &ArtifactId) -> anyhow::Result<Option<Vec<u8>>>;
    fn put(&self, id: &ArtifactId, data: &[u8]) -> anyhow::Result<()>;

    fn as_any(&self) -> &dyn Any;

    fn fetch_many_with_upstreams(
        &self,
        upstreams: &[Arc<dyn RepositoryBackend>],
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


