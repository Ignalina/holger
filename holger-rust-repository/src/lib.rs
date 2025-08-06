use sha2::{Sha256, Digest};
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

    /// Convert crate name to Cargo sparse 5-part path (p1, p2, name)
    pub fn sparse_path(crate_name: &str) -> (String, String, String) {
        let mut chars = crate_name.chars();

        let p1: String = chars.by_ref().take(2).collect();
        let mut p2: String = chars.by_ref().take(2).collect();

        // Cargo uses "_" as filler if name is shorter than 4 chars
        if p2.is_empty() {
            p2.push('_');
        }

        (p1, p2, crate_name.to_string())
    }

    /// Reverse matcher: takes a sparse path slice ["xx","yy","name"] -> crate name
    #[inline]
    pub fn sparse_crate_from_parts<'a>(parts: &'a [&'a str]) -> Option<&'a str> {
        if parts.len() == 3 {
            Some(parts[2])
        } else {
            None
        }
    }
    #[inline]
    pub fn crate_sha256_hex(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        use hex::encode; // Add `hex = "0.4"` to Cargo.toml

        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize(); // GenericArray<u8, U32>
        encode(hash) // Convert to lowercase hex string
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
        let _ = body;
        println!("Rust repo handle_http2_request.suburl={}", suburl);

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
                if let Some(actual_name) = RustRepo::sparse_crate_from_parts(&[p1, p2, crate_name]) {
                    println!("Sparse crate metadata request: {}/{}/{}", p1, p2, actual_name);

                    let fake_crate_data = b"FAKE_CRATE_CONTENT";
                    let checksum_hex = RustRepo::crate_sha256_hex(fake_crate_data);

                    let json = format!(
                        r#"[{{"vers":"1.0.0","deps":[],"cksum":"{}"}}]"#,
                        checksum_hex
                    );

                    return Ok((
                        200,
                        vec![("Content-Type".into(), "application/json".into())],
                        json.into_bytes(),
                    ));
                } else {
                    return Ok((404, Vec::new(), b"Not found".to_vec()));
                }
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


