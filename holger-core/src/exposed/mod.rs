use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use derivative::Derivative;

pub mod http2;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ExposedEndpointInstance {
    pub name: String,
    #[derivative(Debug = "ignore")]
    pub backend: Option<Arc<dyn ExposedEndpointBackend>>,
}

impl ExposedEndpointInstance {
    pub fn new(name: impl Into<String>) -> Self {
        ExposedEndpointInstance {
            name: name.into(),
            backend: None,
        }
    }
}

/// Core trait for all exposed endpoint backends (like HTTP/2 servers)
pub trait ExposedEndpointBackend: Send + Sync {
    /// Return the endpoint name
    fn name(&self) -> &str;

    /// Handle an incoming request, delegating to repositories as needed
    fn handle_request(&self, path: &str, body: &[u8]) -> anyhow::Result<Vec<u8>>;

    /// Allows downcasting for testing / special cases
    fn as_any(&self) -> &dyn Any;
}
