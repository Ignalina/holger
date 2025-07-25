# Holger

**Holger guards your artifacts at rest.**

Immutable Rust-based artifact airgaper. Holger ingests language-specific package trees and serves them over standardized APIs, just like Artifactory or Nexus — but with an airgapped, append-only backend called **Artisan** based on Znippy archives.

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

The Holger service reads this `.znippy` archive and exposes one virtual API endpoint per language. Internally, the `.znippy` file is parsed into one Arrow-based table per language, collectively called an `.artisan` file. Holger uses this file to respond to requests from tools like Cargo, pip, Maven and Go.

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
    HOLGER --> ARTISAN_V1["📚 .artisan v1"]
    HOLGER --> ARTISAN_V2["📚 .artisan v2"]
    HOLGER --> ARTISAN_V3["📚 .artisan v3"]
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

ARTISAN_V2 --> CARGO_dev
ARTISAN_V2 --> PIP_dev
ARTISAN_V2 --> MAVEN_dev
ARTISAN_V2 --> GOPROXY_dev

ARTISAN_V3 --> CARGO_prod
ARTISAN_V3 --> PIP_prod
ARTISAN_V3 --> MAVEN_prod
ARTISAN_V3 --> GOPROXY_prod

```

## Status

- ✅ Znippy archive ingestion
- ✅ Arrow-based indexing
- ✅ Immutable .artisan output
- 🔧 API servers in progress
- 🛡 Blake3 verification in place

