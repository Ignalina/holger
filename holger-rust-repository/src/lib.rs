use std::any::Any;
use anyhow::anyhow;
use holger_traits::{ArtifactFormat, ArtifactId, RepositoryBackendTrait};

/// Minimal RustRepo example
pub struct RustRepo {
    pub name: String,
//    pub format: ArtifactFormat,
    pub artifacts: Vec<ArtifactId>, // cached list of artifacts
}

impl RustRepo {
    pub fn new(name: String) -> Self {
        RustRepo {

            // initialize fields if any; if none, leave empty struct
            // Example: name
            // name,
            name,
            artifacts: vec![],
        }
    }
}

impl RepositoryBackendTrait for RustRepo {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle_http2_request(
        &self,
        suburl: &str,
        body: &[u8],
    ) -> anyhow::Result<(u16, Vec<(String, String)>, Vec<u8>)> {
        let _ = body; // currently unused
        println!("Rust repo handle_http2_request.suburl={} ",suburl);

        let parts: Vec<&str> = suburl.trim_start_matches('/').split('/').collect();

        match parts.as_slice() {
            ["crates", crate_name, version, "download"] => {
                println!("Download request: crate={} version={}", crate_name, version);
                Ok((
                    200,
                    vec![("Content-Type".into(), "application/octet-stream".into())],
                    b"OK".to_vec(),
                ))
            }

            ["config.json"] => {
                println!("config.json requested");
                let json = r#"{"dl":"https://127.0.0.1:8443/crates"}"#;
                Ok((
                    200,
                    vec![("Content-Type".into(), "application/json".into())],
                    json.as_bytes().to_vec(),
                ))
            }

            ["index", crate_name] => {
                println!("Index request: crate={}", crate_name);
                Ok((
                    200,
                    vec![("Content-Type".into(), "text/plain".into())],
                    b"dummy-index-content".to_vec(),
                ))
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


