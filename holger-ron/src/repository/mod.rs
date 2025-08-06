mod types;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use holger_rust_repository::RustRepo;
use holger_traits::RepositoryBackendTrait;
use crate::{ExposedEndpoint, StorageEndpoint};

#[derive(Serialize, Deserialize)]
pub struct Repository {
    // Parsed from RON
    pub ron_name: String,
    pub ron_repo_type: String,        // rust/java/python/raw
    pub ron_upstreams: Vec<String>,   // empty means no upstreams
    pub ron_in: Option<RepositoryIO>,
    pub ron_out: Option<RepositoryIO>,

    // Wired in second pass
    #[serde(skip_serializing, skip_deserializing, default)]
    pub backend_repository: Option<Arc<dyn RepositoryBackendTrait>>,


    #[serde(skip_serializing, skip_deserializing, default)]
    pub wired_upstreams: Vec<*const Repository>, // or &Repository pinned after build
}
impl Repository {
    pub fn backend_from_config(&mut self) -> anyhow::Result<()> {
        match self.ron_repo_type.as_str() {
            "rust" => {
                // Create the RustRepo and wrap it in Arc<dyn RepositoryBackendTrait>
                let backend: Arc<dyn RepositoryBackendTrait> = Arc::new(RustRepo {
                    name: self.ron_name.clone(),
                    artifacts: vec![],
                });

                self.backend_repository = Some(backend);
                Ok(())
            }
            other => anyhow::bail!("Unsupported repository type: {}", other),
        }
    }}

#[derive(Serialize, Deserialize)]
pub struct RepositoryIO {
    pub ron_storage_endpoint: String,
    pub ron_exposed_endpoint: String,


    #[serde(skip_serializing, skip_deserializing, default = "std::ptr::null")]
    pub wired_storage: *const StorageEndpoint,
    #[serde(skip_serializing, skip_deserializing, default = "std::ptr::null")]
    pub wired_exposed: *const ExposedEndpoint,
}
