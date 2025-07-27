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





async fn handle_request(_req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    let body = Full::new(Bytes::from_static(b"Hello over HTTPS+HTTP2")).boxed();
    Ok(Response::new(body))
}
fn load_tls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    println!("{}", std::env::current_dir()?.display());
    let certs = load_certs(cert_path)?;
    let key = load_key(key_path)?;

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

fn load_key<P: AsRef<Path>>(path: P) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(&path).context("unable to open key.pem")?;
    let mut reader = BufReader::new(file);
    let items = read_all(&mut reader).context("unable to parse PEM in key.pem")?;

    for item in items {
        match item {
            Item::RSAKey(key) => {
                let pkcs1 = PrivatePkcs1KeyDer::from(key);
                return Ok(PrivateKeyDer::from(pkcs1));
            }
            Item::PKCS8Key(key) => {
                let pkcs8 = PrivatePkcs8KeyDer::from(key);
                return Ok(PrivateKeyDer::from(pkcs8));
            }
            Item::ECKey(key) => {
                let sec1 = PrivateSec1KeyDer::from(key);
                return Ok(PrivateKeyDer::from(sec1));
            }
            _ => continue, // Skip unknown
        }
    }

    Err(anyhow::anyhow!("no private key found in {:?}", path.as_ref()))
}
