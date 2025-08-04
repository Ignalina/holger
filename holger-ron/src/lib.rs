// holger-ron/src/lib.rs
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
//use hyper::{Request, Response, Body};
//use hyper::body::{BoxBody, Bytes};


use http_body_util::BodyExt;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Context;
use hyper::{
    body::Incoming as Body,
    service::service_fn,
    StatusCode,
    Request,
    Response
};
use http_body_util::combinators::BoxBody;
use http_body_util::Full;

use rustls_pemfile::certs;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::{pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer}, ServerConfig};

use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use ron::de::from_reader;

// ========================= public use  =========================

//pub use {read_ron_config};

// ========================= ROOT HOLGER =========================

#[derive(Serialize, Deserialize)]
pub struct Holger {
    pub repositories: Vec<Repository>,
    pub exposed_endpoints: Vec<ExposedEndpoint>,
    pub storage_endpoints: Vec<StorageEndpoint>,
}

// ========================= LOAD RON CONFIG =========================

pub fn read_ron_config<P: AsRef<Path>>(path: P) -> Result<Holger> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let holger: Holger = from_reader(reader)?;
    Ok(holger)
}

// ========================= RON STRUCTS =========================

#[derive(Serialize, Deserialize)]
pub struct Repository {
    // Parsed from RON
    pub ron_name: String,
    pub ron_repo_type: String,        // rust/java/python/raw
    pub ron_upstreams: Vec<String>,   // empty means no upstreams
    pub ron_in: Option<RepositoryIO>,
    pub ron_out: Option<RepositoryIO>,

    // Wired in second pass
    #[serde(skip_serializing, skip_deserializing)]
    pub wired_backend: Option<Box<dyn RepositoryBackendTrait>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub wired_upstreams: Vec<*const Repository>, // or &Repository pinned after build
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryIO {
    pub ron_storage_endpoint: String,
    pub ron_exposed_endpoint: String,


    #[serde(skip_serializing, skip_deserializing, default = "std::ptr::null")]
    pub wired_storage: *const StorageEndpoint,
    #[serde(skip_serializing, skip_deserializing, default = "std::ptr::null")]
    pub wired_exposed: *const ExposedEndpoint,
}

#[derive(Serialize, Deserialize)]
pub struct ExposedEndpoint {
    pub ron_name: String,
    pub ron_url: String, // Parsed internally to ip/port
    #[serde(skip_serializing, skip_deserializing)]
    pub backend_http2: Option<Box<dyn ExposedEndpoint_http2_Trait>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub wired_in_repositories: Vec<*const Repository>,
    #[serde(skip_serializing, skip_deserializing)]
    pub wired_out_repositories: Vec<*const Repository>,
}

#[derive(Serialize, Deserialize)]
pub struct StorageEndpoint {
    pub ron_name: String,
    pub ron_storage_type: String, // "znippy" | "rocksdb"
    pub ron_path: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub backend_raf: Option<Box<dyn StorageEndpoint_raf_Trait>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub wired_in_repositories: Vec<*const Repository>,
    #[serde(skip_serializing, skip_deserializing)]
    pub wired_out_repositories: Vec<*const Repository>,

}

// ========================= TRAITS =========================

#[async_trait]
pub trait RepositoryBackendTrait: Send + Sync {
    fn name(&self) -> &str;

    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    fn from_config(repo: &Repository) -> Result<Self>
    where
        Self: Sized;

    fn handle_http2_request(
        &self,
        suburl: &str,
        body: &[u8],
    ) -> anyhow::Result<(u16, Vec<(String, String)>, Vec<u8>)>;
}

#[async_trait]
pub trait ExposedEndpoint_http2_Trait: Send + Sync {
    fn name(&self) -> &str;

    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    fn from_config(endpoint: &ExposedEndpoint) -> Result<Self>
    where
        Self: Sized;

    async fn handle_request(
        &self,
        req: Request<Body>,
    ) -> Result<Response<BoxBody<Bytes, std::convert::Infallible>>, hyper::Error>;

    fn set_fast_routes(&mut self, routes: FastRoutes);
}

pub trait StorageEndpoint_raf_Trait: Send + Sync {
    fn name(&self) -> &str;

    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    fn from_config(endpoint: &StorageEndpoint) -> Result<Self>
    where
        Self: Sized;
}

// ========================= FAST ROUTES TRAIT =========================

#[derive(Clone)]
pub struct FastRoute {
    pub name: String,
    pub backend: Arc<dyn RepositoryBackendTrait>,
}

#[derive(Clone)]
pub struct FastRoutes {
    pub routes: Vec<FastRoute>,
    pub first_byte_index: [usize; 256],
    pub first_byte_len: [usize; 256],
}

pub trait FastRoutesTrait {
    fn new(routes: Vec<(String, Arc<dyn RepositoryBackendTrait>)>) -> Self
    where
        Self: Sized;

    fn lookup(&self, name: &str) -> Option<&Arc<dyn RepositoryBackendTrait>>;
}
