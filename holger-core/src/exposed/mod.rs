use async_trait::async_trait;
use bytes::Bytes;
use hyper::{Request, Response, body::Incoming as Body};
use http_body_util::combinators::BoxBody;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use derivative::Derivative;
use crate::exposed::http2::{FastRoutes, Http2Backend};
use crate::{ExposedEndpoint, RepositoryInstance};

pub mod http2;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ExposedEndpointInstance {
    pub name: String,
    pub ip: String,
    pub port: u16,
    #[derivative(Debug = "ignore")]
    pub backend: Option<Arc<dyn ExposedEndpointBackend>>,
    #[derivative(Debug = "ignore")]
    pub fast_routes: Option<FastRoutes>
}


impl ExposedEndpointInstance {
    pub fn new(
        name: impl Into<String>,
        ip: impl Into<String>,
        port: u16,
        backend: Option<Arc<dyn ExposedEndpointBackend>>,
//        routes: HashMap<String, Arc<RepositoryInstance>>,
    ) -> Self {
        ExposedEndpointInstance {
            name: name.into(),
            ip: ip.into(),
            port,
            backend,
            fast_routes: None
        }
    }
    pub fn from_config(cfg: &ExposedEndpoint) -> anyhow::Result<Self> {
        let (ip, port) = Http2Backend::parse_ip_port(&cfg.url_prefix);
        Ok(Self::new(cfg.name.clone(), ip.clone(), port,None))
    }
    pub fn set_fast_routes(&mut self, routes: FastRoutes) {
        self.fast_routes = Some(routes);
    }

    /// Optionally set backend in the second pass
    pub fn set_backend(&mut self, backend: Arc<dyn ExposedEndpointBackend>) {
        self.backend = Some(backend);
    }
}



#[async_trait]
pub trait ExposedEndpointBackend: Send + Sync {
    /// Return the endpoint name
    fn name(&self) -> &str;

    /// Handle an incoming HTTP/2 request asynchronously
    async fn handle_request(
        &self,
        req: Request<Body>,
    ) -> Result<Response<BoxBody<Bytes, std::convert::Infallible>>, hyper::Error>;

    /// Allows downcasting for testing / special cases
    fn as_any(&self) -> &dyn Any;
    fn start(&self) -> anyhow::Result<()>;

    /// Stop the backend and block until stopped
    fn stop(&self) -> anyhow::Result<()>;
    fn set_fast_routes(&mut self, routes: FastRoutes);
}
