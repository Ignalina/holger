mod raf_backend;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::exposed::http2_backend::Http2Backend;
use crate::repository::Repository;
use crate::storage::raf_backend::RafBackend;

#[derive(Serialize, Deserialize)]
pub struct StorageEndpoint {
    pub ron_name: String,
    pub ron_storage_type: String, // "znippy" | "rocksdb"
    pub ron_path: String,

    #[serde(skip_deserializing, skip_serializing, default)]
    pub backend_raf: Arc<RafBackend>,

    #[serde(skip_serializing, skip_deserializing, default)]
    pub wired_in_repositories: Vec<*const Repository>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub wired_out_repositories: Vec<*const Repository>,

}

impl StorageEndpoint {
    pub fn backend_from_config(&mut self) -> anyhow::Result<()> {
        // instantiate self.backend_raf or similar
        Ok(())
    }
}


