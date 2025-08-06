// holger-ron/src/lib.rs
use serde::{Deserialize, Serialize};
//use hyper::{Request, Response, Body};
//use hyper::body::{BoxBody, Bytes};


use http_body_util::BodyExt;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Context;
use anyhow::Result;
use ron::de::from_reader;

// ========================= Wire Holger  =========================

use std::collections::HashMap;
use crate::exposed::ExposedEndpoint;
pub use crate::repository::Repository;
pub use crate::storage::StorageEndpoint;

pub mod exposed;
mod repository;
mod storage;

pub fn wire_holger(holger: &mut Holger) -> Result<()> {
    // ========================= PASS 1: Index by name =========================
    let mut repo_map = HashMap::new();
    let mut exposed_map = HashMap::new();
    let mut storage_map = HashMap::new();

    for repo in &*holger.repositories {
        repo_map.insert(repo.ron_name.clone(), repo as *const Repository);
    }
    for exp in &*holger.exposed_endpoints {
        exposed_map.insert(exp.ron_name.clone(), exp as *const ExposedEndpoint);
    }
    for st in &*holger.storage_endpoints {
        storage_map.insert(st.ron_name.clone(), st as *const StorageEndpoint);
    }

    // ========================= PASS 2: Wire forward references =========================
    for repo in &mut holger.repositories {
        // Wire upstreams
        for name in &repo.ron_upstreams {
            if let Some(ptr) = repo_map.get(name) {
                repo.wired_upstreams.push(*ptr);
            } else {
                return Err(anyhow::anyhow!("Missing upstream repo: {}", name));
            }
        }

        // Wire IN IO
        if let Some(io) = &mut repo.ron_in {
            io.wired_storage = *storage_map
                .get(&io.ron_storage_endpoint)
                .ok_or_else(|| anyhow::anyhow!("Missing storage endpoint: {}", io.ron_storage_endpoint))?;
            io.wired_exposed = *exposed_map
                .get(&io.ron_exposed_endpoint)
                .ok_or_else(|| anyhow::anyhow!("Missing exposed endpoint: {}", io.ron_exposed_endpoint))?;
        }

        // Wire OUT IO
        if let Some(io) = &mut repo.ron_out {
            io.wired_storage = *storage_map
                .get(&io.ron_storage_endpoint)
                .ok_or_else(|| anyhow::anyhow!("Missing storage endpoint: {}", io.ron_storage_endpoint))?;
            io.wired_exposed = *exposed_map
                .get(&io.ron_exposed_endpoint)
                .ok_or_else(|| anyhow::anyhow!("Missing exposed endpoint: {}", io.ron_exposed_endpoint))?;
        }
    }

    // ========================= PASS 2b: Wire reverse links =========================
    for exp in &mut holger.exposed_endpoints {
        for repo in &holger.repositories {
            if let Some(io) = &repo.ron_in {
                if io.ron_exposed_endpoint == exp.ron_name {
                    exp.wired_in_repositories.push(repo as *const Repository);
                }
            }
            if let Some(io) = &repo.ron_out {
                if io.ron_exposed_endpoint == exp.ron_name {
                    exp.wired_out_repositories.push(repo as *const Repository);
                }
            }
        }
    }

    for st in &mut holger.storage_endpoints {
        for repo in &holger.repositories {
            if let Some(io) = &repo.ron_in {
                if io.ron_storage_endpoint == st.ron_name {
                    st.wired_in_repositories.push(repo as *const Repository);
                }
            }
            if let Some(io) = &repo.ron_out {
                if io.ron_storage_endpoint == st.ron_name {
                    st.wired_out_repositories.push(repo as *const Repository);
                }
            }
        }
    }

    // ========================= PASS 3: Reverse wiring =========================
    for repo in &holger.repositories {
        let repo_ptr = repo as *const Repository;

        // Wire IN
        if let Some(io) = &repo.ron_in {
            // Storage reverse link
            let storage_ptr = io.wired_storage;
            let storage = unsafe { &mut *(storage_ptr as *mut StorageEndpoint) };
            storage.wired_in_repositories.push(repo_ptr);

            // Exposed reverse link
            let exposed_ptr = io.wired_exposed;
            let exposed = unsafe { &mut *(exposed_ptr as *mut ExposedEndpoint) };
            exposed.wired_in_repositories.push(repo_ptr);
        }

        // Wire OUT
        if let Some(io) = &repo.ron_out {
            // Storage reverse link
            let storage_ptr = io.wired_storage;
            let storage = unsafe { &mut *(storage_ptr as *mut StorageEndpoint) };
            storage.wired_out_repositories.push(repo_ptr);

            // Exposed reverse link
            let exposed_ptr = io.wired_exposed;
            let exposed = unsafe { &mut *(exposed_ptr as *mut ExposedEndpoint) };
            exposed.wired_out_repositories.push(repo_ptr);
        }
    }


    Ok(())
}

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


impl Holger {
    pub fn start(&self) -> anyhow::Result<()> {
        for ep in &self.exposed_endpoints {
            let backend = ep.backend_http2.clone();
            tokio::spawn(async move {
                if let Err(e) = backend.start().await {
                    eprintln!("Backend start failed: {e}");
                }
            });
        }
        Ok(())
    }


    pub fn stop(&self) -> anyhow::Result<()> {
        for ep in &self.exposed_endpoints {
            // Arc<Http2Backend> → just clone and call stop()
            ep.backend_http2.stop()?;
        }
        Ok(())
    }
    pub fn instantiate_backends(&mut self) -> anyhow::Result<()> {
        for ep in &mut self.exposed_endpoints {
            ep.backend_from_config()?;
        }
        for se in &mut self.storage_endpoints {
            se.backend_from_config()?;
        }
        for repo in &mut self.repositories {
            repo.backend_from_config()?;
        }
        Ok(())
    }
}
// ========================= RON STRUCTS =========================





