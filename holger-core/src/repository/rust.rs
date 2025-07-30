use crate::RepositoryBackend;
use std::any::Any;
use anyhow::anyhow;
use crate::{ArtifactFormat, ArtifactId, StorageEndpointInstance};

/// Minimal RustRepo example
pub struct RustRepo {
    pub name: String,
    pub in_backend: Option<StorageEndpointInstance>,
    pub out_backend: StorageEndpointInstance,
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

    fn fetch(&self, _id: &ArtifactId) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn put(&self, _id: &ArtifactId, _data: &[u8]) -> anyhow::Result<()> {
        if self.in_backend.is_none() {
            return Err(anyhow!("Repository '{}' is read-only", self.name));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
