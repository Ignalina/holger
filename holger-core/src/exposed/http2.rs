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
use std::sync::{Arc, Mutex};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
use crate::{RepositoryBackend, StorageEndpointInstance};
use crate::repository::rust::RustRepo;
use super::ExposedEndpointBackend;
use std::any::Any;
/// HTTP2 backend holding routing to repository backends
pub struct Http2Backend {
    pub  name: String,
    pub listener_addr: String,
    pub port: u16,
    pub tls_config: Arc<ServerConfig>,
    pub running: Arc<AtomicBool>,
    pub routes: HashMap<String, Arc<dyn RepositoryBackend>>,


}

impl Http2Backend {

    /// Register a repository backend to a sub-URL
    pub fn new(name: String, listener_addr: String, port: u16,tls_config: Arc<ServerConfig>) -> Self {
        Self {
            name,
            listener_addr,
            port,
            tls_config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            routes:  HashMap::new(),
        }
    }

    /// ✅ Now takes &self, safe to call from factory
    /// Start serving requests asynchronously (spawns a background task)
    /// 2️⃣ Start serving HTTPS + HTTP/2 using the internal routing map
    pub async fn start(self: Arc<Self>) -> anyhow::Result<JoinHandle<()>> {
        let listener = TcpListener::bind(&self.listener_addr).await?;
        let tls_acceptor = TlsAcceptor::from(self.tls_config.clone());
        self.running.store(true, Ordering::SeqCst);

        println!("Listening on https://{}", self.listener_addr);

        let running = self.running.clone();
        let handle = tokio::spawn(async move {
            loop {
                if !running.load(Ordering::SeqCst) {
                    println!("Http2Backend stopped");
                    break;
                }

                let Ok((stream, _)) = listener.accept().await else {
                    eprintln!("TCP accept failed");
                    continue;
                };

                let acceptor = tls_acceptor.clone();
                let this = Arc::clone(&self);

                tokio::spawn(async move {
                    let Ok(tls_stream) = acceptor.accept(stream).await else {
                        eprintln!("TLS handshake failed");
                        return;
                    };

                    let io = TokioIo::new(tls_stream);
                    let builder = http2::Builder::new(hyper_util::rt::TokioExecutor::new());

                    if let Err(err) = builder
                        .serve_connection(io, service_fn(move |req| {
                            let this = Arc::clone(&this);
                            async move { this.handle_request(req).await }
                        }))
                        .await
                    {
                        eprintln!("Connection error: {:?}", err);
                    }
                });
            }
        });

        Ok(handle)
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

    fn start(&self) -> anyhow::Result<()> {
        println!("Starting HTTP2 backend on {}:{}", self.listener_addr, self.port);
        // Here you would normally bind the TCP listener and spawn the server thread.
        // For now, just mark as running.
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        // Signal the server loop to exit
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        println!("Stopping HTTP2 backend on {}:{}", self.listener_addr, self.port);
        Ok(())
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


/*******





 */
use std::sync::{ atomic::{AtomicBool, Ordering}};
use tokio::task::JoinHandle;
use hyper::server::conn::http2;
use hyper_util::rt::TokioIo;



/*


 */


