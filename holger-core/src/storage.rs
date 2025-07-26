use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ResolvedStorage {
    pub name: String,
    pub kind: StorageKind,
    pub supports_random_read: bool,
}

#[derive(Debug, Clone)]
pub enum StorageKind {
    Znippy { path: String },
    RocksDb { path: String },
    S3 { bucket: String, prefix: Option<String> },
}
