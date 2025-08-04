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
    pub storage: Option<Arc<StorageEndpointInstance>>,
    pub exposed: Option<Arc<ExposedEndpointInstance>>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RepositoryInstance {
    pub name: String,
    pub format: ArtifactFormat,
    pub repo_type: RepositoryType,
    pub in_io: Option<IOInstance>,
    pub out_io: Option<IOInstance>,
    pub upstreams: Option<Vec<String>>, // store names first; link phase later
    #[derivative(Debug = "ignore")]
    pub backend: Option<Arc<dyn RepositoryBackend>>
}


/*
impl RepositoryInstance {
    /// Pass 1: build empty I/O (no storage or endpoints)
    pub fn from_config(cfg: &Repository) -> anyhow::Result<Self> {
        let format = match cfg.ty {
            RepositoryType::Maven3 => ArtifactFormat::Maven3,
            RepositoryType::Pip => ArtifactFormat::Pip,
            RepositoryType::Rust => ArtifactFormat::Rust,
            RepositoryType::Raw => ArtifactFormat::Raw,
        };

        Ok(RepositoryInstance {
            name: cfg.name.clone(),
            format,
            repo_type: cfg.ty.clone(),
            in_io: None,
            out_io: None,
            upstreams: cfg.upstreams.clone(),
            backend: None,
        })
    }

    /// Pass 2: wire storage + endpoints
    pub fn wire<F, G>(
        &mut self,
        cfg: &Repository,
        resolve_storage: &F,
        resolve_endpoint: &G,
    ) -> anyhow::Result<()>
    where
        F: Fn(&str) -> anyhow::Result<Arc<StorageEndpointInstance>>,
        G: Fn(&str) -> anyhow::Result<Arc<ExposedEndpointInstance>>,
    {
        // --- Wire IN
        self.in_io = if let Some(ref in_cfg) = cfg.r#in {
            Some(IOInstance {
                storage: resolve_storage(&in_cfg.storage_backend)?,
                exposed: resolve_endpoint(&in_cfg.exposed_endpoint)?,
            })
        } else {
            None
        };

        // --- Wire OUT
        self.out_io = if let Some(ref out_cfg) = cfg.out {
            Some(IOInstance {
                storage: resolve_storage(&out_cfg.storage_backend)?,
                exposed: resolve_endpoint(&out_cfg.exposed_endpoint)?,
            })
        } else {
            None
        };

        // --- Instantiate backend based on format
        self.backend = Some(match self.format {
            ArtifactFormat::Rust => Arc::new(crate::repository::rust::RustRepo::new(self.name.clone()))
                as Arc<dyn RepositoryBackend>,
            _ => {
                return Err(anyhow::anyhow!(
            "Backend not implemented for format: {:?}",
            self.format
        ))
            }
        });

        Ok(())
    }
}
*/

/*
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
                exposed: resolve_endpoint(&in_cfg.exposed_endpoint)?,
            })
        } else {
            None
        };

        // Handle optional out_io
        let out_io = if let Some(ref out_cfg) = cfg.out {
            Some(IOInstance {
                storage: resolve_storage(&out_cfg.storage_backend)?,
                exposed: resolve_endpoint(&out_cfg.exposed_endpoint)?,
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
*/

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





impl RepositoryInstance {
    pub fn exposed_endpoint_name(&self) -> &str {
        self.out_io
            .as_ref()
            .and_then(|io| io.exposed.as_ref()) // Option<&ExposedEndpointInstance>
            .map(|exposed| exposed.name.as_str())
            .unwrap_or("")
    }    pub fn from_config(cfg: &Repository) -> anyhow::Result<Self> {
        let format = match cfg.ty {
            RepositoryType::Maven3 => ArtifactFormat::Maven3,
            RepositoryType::Pip => ArtifactFormat::Pip,
            RepositoryType::Rust => ArtifactFormat::Rust,
            RepositoryType::Raw => ArtifactFormat::Raw,
        };

        Ok(RepositoryInstance {
            name: cfg.name.clone(),
            format,
            repo_type: cfg.ty.clone(),
            in_io: None,
            out_io: None,
            upstreams: if cfg.upstreams.is_empty() { None } else { Some(cfg.upstreams.clone()) },
            backend: None,
        })
    }

    /// Pass 2: wire storage + endpoints
    pub fn wire(
        &mut self,
        cfg: &Repository,
        storage_map: &HashMap<String, Arc<StorageEndpointInstance>>,
        endpoint_map: &HashMap<String, Arc<ExposedEndpointInstance>>,
    ) -> anyhow::Result<()> {
        // --- Wire IN
        self.in_io = cfg.r#in.as_ref().map(|in_cfg| {
            let storage = storage_map.get(&in_cfg.storage_backend)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Storage endpoint '{}' not found", in_cfg.storage_backend))?;
            let exposed = endpoint_map.get(&in_cfg.exposed_endpoint)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Exposed endpoint '{}' not found", in_cfg.exposed_endpoint))?;
            Ok::<IOInstance, anyhow::Error>(IOInstance { storage: Some(storage), exposed: Some(exposed) })
        }).transpose()?;

        // --- Wire OUT
        self.out_io = cfg.out.as_ref().map(|out_cfg| {
            let storage = storage_map.get(&out_cfg.storage_backend)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Storage endpoint '{}' not found", out_cfg.storage_backend))?;
            let exposed = endpoint_map.get(&out_cfg.exposed_endpoint)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Exposed endpoint '{}' not found", out_cfg.exposed_endpoint))?;
            Ok::<IOInstance, anyhow::Error>(IOInstance { storage: Some(storage), exposed: Some(exposed) })
        }).transpose()?;

        // --- Instantiate backend based on format
        self.backend = Some(match self.format {
            ArtifactFormat::Rust => Arc::new(crate::repository::rust::RustRepo::new(self.name.clone()))
                as Arc<dyn RepositoryBackend>,
            _ => return Err(anyhow::anyhow!("Backend not implemented for format: {:?}", self.format)),
        });

        Ok(())
    }
    pub fn backend(&self) -> Option<Arc<dyn RepositoryBackend>> {
        self.backend.clone()
    }
}
