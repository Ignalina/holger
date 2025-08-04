use async_trait::async_trait;
use bytes::Bytes;
use hyper::{body::Incoming as Body, Request, Response};
use http_body_util::combinators::BoxBody;

use std::any::Any;
use std::sync::Arc;
use derivative::Derivative;
use fast_routes::FastRoutes;
use crate::config::parse_ip_port;
use crate::exposed::http2::Http2Backend;
use crate::ExposedEndpoint;

pub mod http2;
pub mod fast_routes;

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

        let backend: Arc<Http2Backend> = Arc::new( Http2Backend::from_config(
            cfg.name.clone(),
            &cfg.url_prefix,
            &cfg.cert,
            &cfg.key,
        )?);

        // Cast to Arc<dyn ExposedEndpointBackend>
        let backend_arc: Arc<dyn ExposedEndpointBackend> = backend.clone();
        Ok(Self::new(cfg.name.clone(), ip, port,Some(backend_arc)))
    }
    pub fn set_fast_routes(&mut self, routes: FastRoutes) {
        // Keep a copy internally
        self.fast_routes = Some(routes.clone());

        // Cascade to backend if unique Arc
        if let Some(backend_arc) = &mut self.backend {
            // If this Arc is uniquely owned, we can mutate the backend directly
            if let Some(backend_mut) = Arc::get_mut(backend_arc) {
                backend_mut.set_fast_routes(routes);
            } else {
                log::warn!(
                    "ExposedEndpointInstance '{}' backend Arc is shared; \
                     cannot directly set fast routes on backend",
                    self.name
                );
            }
        }
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
