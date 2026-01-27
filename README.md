# Ontology-Registry

[![Crates.io](https://img.shields.io/crates/v/ontology-registry.svg)](https://crates.io/crates/ontology-registry)
[![Docs.rs](https://docs.rs/ontology-registry/badge.svg)](https://docs.rs/ontology-registry)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![RobisonGroup](https://img.shields.io/badge/Robinson%20Group-blue)](https://robinsongroup.github.io/)

**Ontology-Registry** is a Rust crate designed to seamlessly fetch, manage, and persist ontology files directly from
the [OBO Foundry](https://obofoundry.org/).

It acts as a centralized local cache, ensuring that ontologies are persisted on your machine and easily accessible by
any application using this crate, reducing network redundancy and improving load times.

## ✨ Features

* **OBO Foundry Integration:** Pull ontology files straight from the source "out of the box."
* **Local Persistence:** Automatically stores downloaded ontologies in a standard directory on your machine.
* **Shared Registry:** The storage location is designed to be accessible by multiple applications, preventing duplicate
  downloads across different projects.
* **Offline Access:** Once downloaded, ontologies are available without an active internet connection.

## 📦 Installation

Run the following command in your project directory:

```sh
cargo add ontology-registry
```

## Quick Start

```rust
use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
use ontology_registry::enums::{FileType, Version};

let version = Version::Declared("2026-01-16".to_string());
let tmp_dir = TempDir::new().unwrap();
let registry = FileSystemOntologyRegistry::new(
tmp_dir.keep(),
BioRegistryMetadataProvider::default (),
OboLibraryProvider::default (),
);

let res = registry
.register("uo", version.clone(), FileType::Json)
.unwrap();

// Load ontology
```
