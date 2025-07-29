use crate::types::{ArtifactFormat, ArtifactId, Repository, RepositoryType};
use crate::storage::ResolvedStorage;
use anyhow::{anyhow, Result};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct RepositoryInstance {
    pub name: String,
    pub format: ArtifactFormat,
    pub repo_type: RepositoryType,
    pub in_backend: Option<ResolvedStorage>,
    pub out_backend: ResolvedStorage,
    pub upstreams: Vec<String>,
}

impl RepositoryInstance {
    pub fn from_config<F>(cfg: &Repository, resolve_storage: &F) -> Result<Self>
    where
        F: Fn(&str) -> ResolvedStorage,
    {
        let format = match cfg.ty {
            RepositoryType::Maven3 => ArtifactFormat::Maven3,
            RepositoryType::Pip => ArtifactFormat::Pip,
            RepositoryType::Rust => ArtifactFormat::Rust,
            RepositoryType::Raw => ArtifactFormat::Raw,
        };

        let in_backend = cfg.in_.as_ref().map(|in_cfg| resolve_storage(&in_cfg.storage_backend));
        let out_backend = resolve_storage(&cfg.out.storage_backend);
        let upstreams = cfg.upstreams.clone();

        Ok(RepositoryInstance {
            name: cfg.name.clone(),
            format,
            repo_type: cfg.ty.clone(),
            in_backend,
            out_backend,
            upstreams,
        })
    }

    pub fn is_writable(&self) -> bool {
        self.in_backend.is_some()
    }
}

/// Core trait for all repository types
pub trait RepositoryBackend: Send + Sync {
    fn name(&self) -> &str;
    fn format(&self) -> ArtifactFormat;
    fn is_writable(&self) -> bool;

    fn fetch(&self, id: &ArtifactId) -> Result<Option<Vec<u8>>>;
    fn put(&self, id: &ArtifactId, data: &[u8]) -> Result<()>;

    fn as_any(&self) -> &dyn Any;

    fn fetch_many_with_upstreams(
        &self,
        upstreams: &[Arc<dyn RepositoryBackend>],
        ids: &[ArtifactId],
    ) -> Result<HashMap<ArtifactId, Vec<u8>>> {
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
}

/// A sample implementation for RustRepo
pub struct RustRepo {
    pub name: String,
    pub accept_unpublished: bool,
    pub in_backend: Option<ResolvedStorage>,
    pub out_backend: ResolvedStorage,
}

impl RepositoryBackend for RustRepo {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> ArtifactFormat {
        ArtifactFormat::Rust
    }

    fn is_writable(&self) -> bool {
        self.in_backend.is_some()
    }

    fn fetch(&self, _id: &ArtifactId) -> Result<Option<Vec<u8>>> {
        // placeholder for now
        Ok(None)
    }

    fn put(&self, _id: &ArtifactId, _data: &[u8]) -> Result<()> {
        if !self.accept_unpublished || self.in_backend.is_none() {
            return Err(anyhow!("Repository '{}' is read-only", self.name));
        }

        // placeholder for now
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
