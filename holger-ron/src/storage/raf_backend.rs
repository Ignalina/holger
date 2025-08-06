use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use rustls::server::{ClientHello, NoClientAuth, ResolvesServerCert};
use rustls::ServerConfig;
use rustls::sign::CertifiedKey;
use crate::exposed::fast_routes::FastRoutes;

pub struct RafBackend {
    pub name: String,
    pub listener_addr: String,
    pub tls_config: Arc<ServerConfig>,
    pub running: Arc<AtomicBool>,
    pub fast_routes: Option<FastRoutes>,
}


struct DummyResolver;

impl Debug for DummyResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ResolvesServerCert for DummyResolver {
    fn resolve(&self, _client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        None
    }
}

/// Minimal placeholder ServerConfig for `#[serde(default)]`
fn dummy_server_config() -> Arc<ServerConfig> {
    let builder = ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(NoClientAuth));

    let server_config: ServerConfig = builder
        .with_cert_resolver(Arc::new(DummyResolver));

    Arc::new(server_config)
}


impl Default for RafBackend {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            listener_addr: "".to_string(),
            tls_config: dummy_server_config(),
            // … other dummy fields
            running: Arc::new(Default::default()),
            fast_routes: None,
        }
    }
}