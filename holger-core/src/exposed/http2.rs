use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;

use crate::RepositoryBackend;
use super::ExposedEndpointBackend;

/// HTTP2 backend holding routing to repository backends
pub struct Http2Backend {
    name: String,
    /// Maps sub-URL to the backend repository handling it
    pub routes: HashMap<String, Arc<dyn RepositoryBackend>>,
}

impl Http2Backend {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            routes: HashMap::new(),
        }
    }

    /// Register a repository backend to a sub-URL
    pub fn register_route(&mut self, sub_url: &str, backend: Arc<dyn RepositoryBackend>) {
        self.routes.insert(sub_url.to_string(), backend);
    }
}

impl ExposedEndpointBackend for Http2Backend {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle_request(&self, path: &str, _body: &[u8]) -> Result<Vec<u8>> {
        // Match against the first segment after '/'
        let trimmed = path.trim_start_matches('/');
        let first_segment = trimmed.split('/').next().unwrap_or("");

        if let Some(repo_backend) = self.routes.get(first_segment) {
            // This is where you would eventually call
            // repo_backend.handle_http2_backend_req(...)
            Ok(format!("Routed to repository '{}'", repo_backend.name()).into_bytes())
        } else {
            Err(anyhow::anyhow!("No route for path {}", path))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
