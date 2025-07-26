use crate::types::{ArtifactFormat, RepositoryType};
use crate::config::{RepositoryConfig, RepositoryIO};
use crate::storage::{ResolvedStorage};

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
    pub fn from_config<F>(cfg: &RepositoryConfig, resolve_storage: &F) -> Self
    where
        F: Fn(&str) -> ResolvedStorage,
    {
        let format = match &cfg.r#type {
            RepositoryType::Java(_) => ArtifactFormat::Java,
            RepositoryType::Python(_) => ArtifactFormat::Python,
            RepositoryType::Rust(_) => ArtifactFormat::Rust,
            RepositoryType::Raw(_) => ArtifactFormat::Raw,
        };

        let in_backend = cfg.in_.as_ref().map(|io| resolve_storage(&io.storage_backend));
        let out_backend = resolve_storage(&cfg.out.storage_backend);

        RepositoryInstance {
            name: cfg.name.clone(),
            format,
            repo_type: cfg.r#type.clone(),
            in_backend,
            out_backend,
            upstreams: cfg.upstreams.clone().unwrap_or_default(),
        }
    }

    pub fn is_writable(&self) -> bool {
        self.in_backend.is_some()
    }
}

pub trait Repository {
    fn name(&self) -> &str;
    fn format(&self) -> ArtifactFormat;
    fn is_writable(&self) -> bool;
}
