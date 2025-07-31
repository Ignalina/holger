use http_body_util::{Full, combinators::BoxBody};
use bytes::Bytes;
use std::convert::Infallible;
use std::io::Read;

use http_body_util::BodyExt;
use std::{
    fs::File,
    io::{BufReader, Result as IoResult},
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use hyper::{
    body::Incoming as Body,
    service::service_fn,
    Request, Response, StatusCode,
};
use hyper::server::conn::http2;
use hyper_util::rt::TokioIo;
//use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer};
use rustls_pemfile::{certs, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, TlsStream};
use tokio_rustls::rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer}};

#[tokio::main]
async fn main() -> Result<()> {

    let tls_cfg = load_tls_config("tests/certs/cert.pem", "tests/certs/key.pem")?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    println!("Listening on https://127.0.0.1:8443");

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let Ok(tls_stream) = acceptor.accept(stream).await else {
                eprintln!("TLS handshake failed");
                return;
            };

            let io = TokioIo::new(tls_stream);

            let builder = http2::Builder::new(hyper_util::rt::TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, service_fn(handle_request)).await {
                eprintln!("Connection error: {:?}", err);
            }
        });
    }
}






use hyper::body::Body as _;
use std::collections::HashMap;
use once_cell::sync::Lazy;



pub async fn handle_request(
    req: Request<Body>,
) -> Result<Response<BoxBody<Bytes, std::convert::Infallible>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let suburl = path.trim_start_matches('/');

    let body_bytes = req.into_body().collect().await?.to_bytes();
    let body_vec = body_bytes.to_vec();


    let mut repo_map: HashMap<String, Arc<dyn RepositoryBackend>> = HashMap::new();

    let dummy_storage: StorageEndpointInstance = unsafe { std::mem::zeroed() }; // placeholder

    repo_map.insert(
        "crates".to_string(),
        Arc::new(RustRepo {
            name: "dummy".to_string(),
            in_backend: None,
            out_backend: dummy_storage,
        }) as Arc<dyn RepositoryBackend>,
    );

    // Select repo based on first path segment
    let repo_key = suburl.split('/').next().unwrap_or("");
    if let Some(repo) = repo_map.get(repo_key) {
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
use holger_core::repository::rust::RustRepo;
use holger_core::{RepositoryBackend, StorageEndpointInstance};

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

