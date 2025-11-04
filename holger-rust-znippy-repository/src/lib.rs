use anyhow::anyhow;
use holger_traits::{ArtifactFormat, ArtifactId, RepositoryBackendTrait};
use sha2::{Digest, Sha256};
use std::any::Any;

/// Minimal RustRepo example
pub struct RustRepoZnippy {
    pub name: String,
    //    pub format: ArtifactFormat,
    pub artifacts: Vec<ArtifactId>, // cached list of artifacts
}


impl RustRepoZnippy {
    pub fn new(name: String) -> Self {
        RustRepoZnippy {
            // initialize fields if any; if none, leave empty struct
            // Example: name
            // name,
            name,
            artifacts: vec![],
        }
    }
}

impl RepositoryBackendTrait for RustRepoZnippy {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle_http2_request(
        &self,
        suburl: &str,
        body: &[u8],
    ) -> anyhow::Result<(u16, Vec<(String, String)>, Vec<u8>)> {
        let _ = body;
        println!("Rust repo znippy handle_http2_request.suburl={}", suburl);

        let parts: Vec<&str> = suburl.trim_start_matches('/').split('/').collect();

        match parts.as_slice() {
            // Sparse root config.json → /rust-prod/index/config.json
            [repo, "index", "config.json"] if *repo == self.name() => {
                println!("Sparse config.json requested");
                let json = format!(
                    r#"{{
                        "dl": "https://127.0.0.1:8443/{}/crates",
                        "api": null
                    }}"#,
                    self.name()
                );
                return Ok((
                    200,
                    vec![("Content-Type".into(), "application/json".into())],
                    json.as_bytes().to_vec(),
                ));
            }

            // Sparse crate metadata → /rust-prod/index/se/rd/serde
            [repo, "index", p1, p2, crate_name] if *repo == self.name() => {
                    return Ok((404, Vec::new(), b"Not found".to_vec()));

            }

            // Crate download → /crates/<crate>/<version>/download
            ["crates", crate_name, version, "download"] => {
                println!("Download request: crate={} version={}", crate_name, version);
                return Ok((
                    200,
                    vec![("Content-Type".into(), "application/octet-stream".into())],
                    b"FAKE_CRATE_CONTENT".to_vec(),
                ));
            }

            _ => {
                println!("Unhandled path: {}", suburl);
                Ok((404, Vec::new(), b"Not found".to_vec()))
            }
        }
    }
    fn format(&self) -> ArtifactFormat {
        ArtifactFormat::Rust
    }

    fn is_writable(&self) -> bool {
        todo!()
    }

    fn fetch(&self, id: &ArtifactId) -> anyhow::Result<Option<Vec<u8>>> {
        todo!()
    }

    fn put(&self, id: &ArtifactId, data: &[u8]) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}
