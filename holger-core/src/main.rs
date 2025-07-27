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
    let tls_cfg = load_tls_config("cert.pem", "key.pem")?;
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





async fn handle_requestOLD(_req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    let body = Full::new(Bytes::from_static(b"Hello over HTTPS+HTTP2")).boxed();
    Ok(Response::new(body))
}
async fn handle_request(req: Request<Body>) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    let path = req.uri().path();
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    match parts.as_slice() {
        ["crates", crate_name, version, "download"] => {
            println!("Download request: crate={} version={}", crate_name, version);
            let body = Full::new(Bytes::from_static(b"OK")).boxed();
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .body(body)
                .unwrap());
        }

        ["config.json"] => {
            println!("config.json requested");
            let json = r#"{"dl":"https://127.0.0.1:8443/crates"}"#;
            let body = Full::new(Bytes::from_static(json.as_bytes())).boxed();
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap());
        }

        ["index", crate_name] => {
            println!("Index request: crate={}", crate_name);
            let body = Full::new(Bytes::from_static(b"dummy-index-content")).boxed();
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(body)
                .unwrap());
        }

        _ => {
            println!("Unhandled path: {}", path);
            let body = Full::new(Bytes::from_static(b"Not found")).boxed();
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body)
                .unwrap());
        }
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

