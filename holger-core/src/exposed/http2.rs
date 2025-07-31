use http_body_util::BodyExt;
use std::{
    fs::File,
    io::{BufReader, Result as IoResult},
    path::Path,
};

use anyhow::{Context};
use hyper::{
    body::Incoming as Body,
    service::service_fn,
};
use rustls_pemfile::{certs, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, TlsStream};
use tokio_rustls::rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer}};

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
use crate::{RepositoryBackend, StorageEndpointInstance};
use crate::repository::rust::RustRepo;
use super::ExposedEndpointBackend;

/// HTTP2 backend holding routing to repository backends
pub struct Http2Backend {
    name: String,
    /// Maps sub-URL to the backend repository handling it
    pub routes: HashMap<String, Arc<dyn RepositoryBackend>>,
}

impl Http2Backend {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            routes: HashMap::new(),
        }
    }

    /// Register a repository backend to a sub-URL
    pub fn register_route(&mut self, sub_url: &str, backend: Arc<dyn RepositoryBackend>) {
        self.routes.insert(sub_url.to_string(), backend);
    }
}

#[async_trait::async_trait]
impl ExposedEndpointBackend for Http2Backend {
    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_request(
        &self,
        req: Request<Body>,
    ) -> Result<Response<BoxBody<Bytes, std::convert::Infallible>>, hyper::Error> {
        let path = req.uri().path().to_string();
        let suburl = path.trim_start_matches('/');

        let body_bytes = req.into_body().collect().await?.to_bytes();
        let body_vec = body_bytes.to_vec();

        // Use the real routes HashMap
        let repo_key = suburl.split('/').next().unwrap_or("");
        if let Some(repo) = self.routes.get(repo_key) {
            let suburl_owned = suburl.to_string();
            let result = tokio::task::spawn_blocking({
                let repo = Arc::clone(repo);
                move || repo.handle_http2_request(&suburl_owned, &body_vec)
            })
                .await
                .unwrap();

            match result {
                Ok((status, headers, data)) => {
                    let mut response = Response::builder()
                        .status(StatusCode::from_u16(status).unwrap());
                    for (k, v) in headers {
                        response = response.header(k, v);
                    }
                    Ok(response.body(Full::new(Bytes::from(data)).boxed()).unwrap())
                }
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(Bytes::from_static(b"Internal Error")).boxed())
                    .unwrap()),
            }
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from_static(b"Repo Not Found")).boxed())
                .unwrap())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}




fn load_tls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    println!("{}", std::env::current_dir()?.display());
    let certs = load_certs(cert_path)?;
    let key = load_key(Path::new(key_path))?;

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    // Enable HTTP/2 ALPN
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(config)
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut file = BufReader::new(File::open(path)?);
    let certs = certs(&mut file)
        .context("failed to read certificates")?
        .into_iter()
        .map(CertificateDer::from)
        .collect();
    Ok(certs)
}


use rustls_pemfile::{Item, read_all};
use tokio_rustls::rustls::pki_types::{PrivatePkcs8KeyDer, PrivateSec1KeyDer};

fn load_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);
    let items = read_all(&mut reader)?;

    for item in items {
        match item {
            Item::PKCS8Key(bytes) => {
                println!("Found PKCS8 key");
                return Ok(PrivateKeyDer::from(PrivatePkcs8KeyDer::from(bytes)));
            }
            Item::RSAKey(bytes) => {
                println!("Found RSA key");
                return Ok(PrivateKeyDer::from(PrivatePkcs1KeyDer::from(bytes)));
            }
            Item::ECKey(bytes) => {
                println!("Found EC key");
                return Ok(PrivateKeyDer::from(PrivateSec1KeyDer::from(bytes)));
            }
            _ => println!("Skipping non-key item"),
        }
    }

    Err(anyhow::anyhow!("no private key found"))
}

