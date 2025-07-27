# Holger

**Holger guards your artifacts at rest.**

Immutable Rust-based artifact airgaper. Holger ingests language-specific package trees and serves them over standardized APIs, just like Artifactory or Nexus — but with an airgapped, append-only backend called **artifact**, based on Znippy archives.

## Overview

When airgapping environments, your company saves offline packages using native language tools. These packages are exported and organized under the following structure:

```text
/airgap/
  rust/     <- cargo vendor
  python/   <- pip download
  java/     <- mvn dependency:go-offline
  go/       <- TBD
```

These folders are archived into a `.znippy` file by the Znippy CLI. The resulting `.znippy` file is immutable and can be verified using Blake3 checksums.

The Holger service reads this `.znippy` archive and exposes one virtual API endpoint per language. Internally, the `.znippy` file is parsed into one Arrow-based table per language, collectively called an `.artifact` file. Holger uses this file to respond to requests from tools like Cargo, pip, Maven and Go.

## Holger Serving Modes

All `.artifact` files are immutable. However, Holger can optionally be configured to allow **live ingest** of artifacts not found in the current `.artifact`. This is primarily useful in DEV environments.

| History | Source                   | Update Capability         | Use Case                           |
| ---- | ------------------------ | ------------------------- | ---------------------------------- |
| V1   | Initial .artifact import  | Immutable                 | Bootstrap, base snapshot           |
| V2   | .artifact + live ingest   | Yes (in-memory + RocksDB) | DEV: allow dynamic additions       |
| V3   | Promoted `.artifact` only | Immutable                 | PROD: strict airgapped enforcement |

- V2 allows development-time fetches from upstream sources (e.g. crates.io, PyPI) and caches them.
- V3 is the result of promoting selected artifacts from V2 into a new `.artifact` file.
- Live proxy mode can be disabled completely in strict environments.

## Architecture

```mermaid
flowchart LR

%% Step 1: Airgap dump
subgraph Offline_Airgap_Dump
    RUST["rust/ (cargo vendor)"]
    PYTHON["python/ (pip download)"]
    JAVA["java/ (mvn go-offline)"]
    GO["golang/ (TBD)"]
end

RUST --> ZNIPPY["📦 Znippy Archive"]
PYTHON --> ZNIPPY
JAVA --> ZNIPPY
GO --> ZNIPPY

%% Step 2: Holger Ingestion
subgraph Holger_Processing
    INPUTAPI["Holger INPUT api/"] --> HOLGER["🛡 Holger Ingest & Promote Service"]
    ZNIPPY --> HOLGER
    HOLGER --> artifact_V1["📚 .artifact v1"]
    HOLGER --> artifact_V2["📚 .artifact v2"]
    HOLGER --> artifact_V3["📚 .artifact v3"]
end

%% Step 3: Holger Serving via APIs
subgraph Holger DEV _API_Endpoints
    CARGO_dev["Cargo Git+HTTP"]
    PIP_dev["PyPI Simple API"]
    MAVEN_dev["Maven Repo API"]
    GOPROXY_dev["Golang Proxy Mode"]
end

subgraph HolgerPROD _API_Endpoints
    CARGO_prod["Cargo Git+HTTP"]
    PIP_prod["PyPI Simple API"]
    MAVEN_prod["Maven Repo API"]
    GOPROXY_prod["Golang Proxy Mode"]
end

artifact_V2 --> CARGO_dev
artifact_V2 --> PIP_dev
artifact_V2 --> MAVEN_dev
artifact_V2 --> GOPROXY_dev

artifact_V3 --> CARGO_prod
artifact_V3 --> PIP_prod
artifact_V3 --> MAVEN_prod
artifact_V3 --> GOPROXY_prod
```

## Status

- ✅ Znippy archive ingestion
- ✅ Arrow-based indexing
- ✅ Immutable .artifact output
- ⛖ API servers in progress
- 🚽 Blake3 verification in place
- 

## Mindmapping ..
```mermaid

flowchart TD

%% Konfiguration
subgraph Config
    CFG[holger.yaml]
end

%% Repos
subgraph Repositories
    Hosted1[Hosted Repo\nname: java-test\nmode: .artifact + live\nformat: Java]
    Hosted2[Hosted Repo\nname: rust-prod\nmode: .artifact only\nformat: Rust]
    Proxy1[Proxy Repo\nname: maven-central\nURL: mirror]
    Agg1[Aggregated Repo\nname: all-java\nincludes: java-test and maven-central]
end

%% Komponenter
subgraph HolgerCore
    CLI[CLI and REST API]
    Promote[Ingest and Promote Service]
    Access[Artifact Access API]
    Merge[Znippy Merger\nmerging and dependency resolution]
end

%% Metadata
subgraph ArtifactLayer
    Artifact[Artifact\nUUIDv7 and checksums\nplus type metadata]
    DepGraph[Dependency Graph\nUUID-based DAG]
end

%% Flöden
CLI --> Promote
CLI --> Access
Promote -->|uses| Repositories
Access -->|uses| Repositories
Repositories --> Artifact
Repositories --> DepGraph
Merge --> Promote
Artifact --> Merge
DepGraph --> Promote
CFG --> Repositories

```

```json
{
  "exposed_endpoints": [
    {
      "name": "main",
      "url_prefix": "https://holger.example.com/"
    }
  ],
  "storage_endpoints": [
    {
      "name": "znippy-local",
      "type": {
        "znippy": {
          "path": "/var/lib/holger/znippy/",
          "supports_random_read": true
        }
      }
    }
  ],
  "repositories": [
    {
      "name": "rust-prod",
      "type": {
        "rust": {
          "accept_unpublished": false
        }
      },
      "out": {
        "storage_backend": "znippy-local",
        "endpoints": ["main"]
      }
    }
  ]
}

```


```json

{
  "exposed_endpoints": [
    {
      "name": "main",
      "url_prefix": "https://holger.example.com/"
    },
    {
      "name": "internal",
      "url_prefix": "http://localhost:8080/"
    }
  ],
  "storage_endpoints": [
    {
      "name": "znippy-local",
      "type": {
        "znippy": {
          "path": "/var/lib/holger/znippy/",
          "supports_random_read": true
        }
      }
    },
    {
      "name": "rocks-dev",
      "type": {
        "rocksdb": {
          "path": "/var/lib/holger/rocksdb/",
          "supports_random_read": false
        }
      }
    }
  ],
  "repositories": [
    {
      "name": "rust-dev",
      "type": {
        "rust": {
          "accept_unpublished": true
        }
      },
      "in": {
        "storage_backend": "rocks-dev",
        "endpoints": ["internal"]
      },
      "out": {
        "storage_backend": "znippy-local",
        "endpoints": ["main"]
      },
      "upstreams": []
    }
  ]
}
```
