use crate::types::{StorageEndpoint, StorageLocation, StorageType};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Represents a fully resolved storage backend.
#[derive(Debug, Clone)]
pub enum ResolvedStorage {
    ZnippyLocal { path: PathBuf },
    ZnippyS3 { bucket: String, prefix: String },
    RocksdbLocal { path: PathBuf },
}

impl ResolvedStorage {
    pub fn from_config(config: &StorageEndpoint) -> Result<Self> {
        match config.ty {
            StorageType::Znippy => match config.location {
                StorageLocation::Local => Ok(ResolvedStorage::ZnippyLocal {
                    path: PathBuf::from(&config.path),
                }),
                StorageLocation::S3 => Ok(ResolvedStorage::ZnippyS3 {
                    bucket: extract_s3_bucket(&config.path)?,
                    prefix: extract_s3_prefix(&config.path)?,
                }),
            },

            StorageType::Rocksdb => match config.location {
                StorageLocation::Local => Ok(ResolvedStorage::RocksdbLocal {
                    path: PathBuf::from(&config.path),
                }),
                StorageLocation::S3 => Err(anyhow!(
                    "RocksDB cannot use remote (S3) location: {}",
                    config.path
                )),
            },
        }
    }
}

/// Naive S3 parser: splits s3://bucket/prefix...
fn extract_s3_bucket(uri: &str) -> Result<String> {
    let without_prefix = uri.strip_prefix("s3://").ok_or_else(|| {
        anyhow!("Invalid S3 path '{}', must start with s3://", uri)
    })?;

    let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid S3 path '{}', missing '/' after bucket", uri));
    }
    Ok(parts[0].to_string())
}

fn extract_s3_prefix(uri: &str) -> Result<String> {
    let without_prefix = uri.strip_prefix("s3://").ok_or_else(|| {
        anyhow!("Invalid S3 path '{}', must start with s3://", uri)
    })?;

    let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid S3 path '{}', missing '/' after bucket", uri));
    }
    Ok(parts[1].to_string())
}
