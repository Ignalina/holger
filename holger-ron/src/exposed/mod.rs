use async_trait::async_trait;
use bytes::Bytes;
use hyper::{body::Incoming as Body, Request, Response};
use http_body_util::combinators::BoxBody;

use std::any::Any;
use std::sync::Arc;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use fast_routes::FastRoutes;
use crate::exposed::http2_backend::Http2Backend;
use crate::repository::Repository;

pub mod http2_backend;
pub mod fast_routes;


#[derive(Serialize, Deserialize)]
pub struct ExposedEndpoint {
    pub ron_name: String,
    pub ron_url: String, // Parsed internally to ip/port
    pub ron_cert: String,
    pub ron_key: String,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub backend_http2: Arc<Http2Backend>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub aggregated_routes: Option<FastRoutes>,

    #[serde(skip_serializing, skip_deserializing, default)]
    pub wired_in_repositories: Vec<*const Repository>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub wired_out_repositories: Vec<*const Repository>,
}

impl ExposedEndpoint {

    pub fn backend_from_config(&mut self) -> anyhow::Result<()> {
        let mut backend = Http2Backend::backend_from_config(self)?;

        self.backend_http2 = Arc::new(backend);
        Ok(())
    }
}
