use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use rustls::ServerConfig;
use crate::exposed::fast_routes::FastRoutes;

pub struct RafBackend {
    pub name: String,
    pub listener_addr: String,
    pub tls_config: Arc<ServerConfig>,
    pub running: Arc<AtomicBool>,
    pub fast_routes: Option<FastRoutes>,
}

impl RafBackend {
    fn new() -> RafBackend {
        todo!()
    }
}

impl Default for RafBackend {
    fn default() -> Self {
        Self::new()
    }
}