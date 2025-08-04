use std::sync::Arc;
use crate::RepositoryBackend;

#[derive(Clone)]
pub struct FastRoute {
    pub name: String,
    // todo make Option
    pub backend: Arc<dyn RepositoryBackend>,
}

#[derive(Clone)]
pub struct FastRoutes {
    pub routes: Vec<FastRoute>,
    pub first_byte_index: [usize; 256],
    pub first_byte_len: [usize; 256],
}

impl FastRoutes {
    pub fn new(routes: Vec<(String, Arc<dyn RepositoryBackend>)>) -> Self {
        let mut routes_vec: Vec<FastRoute> = routes
            .into_iter()
            .map(|(name, backend)| FastRoute { name, backend })
            .collect();

        // Sort by first byte then length
        routes_vec.sort_by(|a, b| {
            let fa = a.name.as_bytes().first().copied().unwrap_or(0);
            let fb = b.name.as_bytes().first().copied().unwrap_or(0);
            fa.cmp(&fb).then(a.name.len().cmp(&b.name.len()))
        });

        let mut first_byte_index = [0usize; 256];
        let mut first_byte_len = [0usize; 256];

        // Build lookup table
        let mut i = 0usize;
        while i < routes_vec.len() {
            let byte = routes_vec[i].name.as_bytes().first().copied().unwrap_or(0);
            let start = i;
            while i < routes_vec.len()
                && routes_vec[i].name.as_bytes().first().copied().unwrap_or(0) == byte
            {
                i += 1;
            }
            first_byte_index[byte as usize] = start;
            first_byte_len[byte as usize] = i - start;
        }

        Self {
            routes: routes_vec,
            first_byte_index,
            first_byte_len,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Arc<dyn RepositoryBackend>> {
        let bytes = name.as_bytes();
        let first = bytes.first().copied().unwrap_or(0) as usize;

        let start = self.first_byte_index[first];
        let len = self.first_byte_len[first];
        if len == 0 {
            return None;
        }

        let bucket = &self.routes[start..start + len];
        for r in bucket {
            if r.name == name {
                return Some(&r.backend);
            }
        }
        None
    }
}