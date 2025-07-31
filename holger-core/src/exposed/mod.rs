use async_trait::async_trait;
use bytes::Bytes;
use hyper::{Request, Response, body::Incoming as Body};
use http_body_util::combinators::BoxBody;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use derivative::Derivative;
use crate::RepositoryInstance;

pub mod http2;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ExposedEndpointInstance {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub routes: HashMap<String, Arc<RepositoryInstance>>,
    #[derivative(Debug = "ignore")]
    pub backend: Arc<dyn ExposedEndpointBackend>,
}

impl ExposedEndpointInstance {
    pub fn new(
        name: impl Into<String>,
        ip: impl Into<String>,
        port: u16,
        backend: Arc<dyn ExposedEndpointBackend>,
        routes: HashMap<String, Arc<RepositoryInstance>>,
    ) -> Self {
        ExposedEndpointInstance {
            name: name.into(),
            ip: ip.into(),
            port,
            routes,
            backend,
        }
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

}
