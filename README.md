# Ontology-Registry

[![Crates.io](https://img.shields.io/crates/v/ontology-registry.svg)](https://crates.io/crates/ontology-registry)
[![Docs.rs](https://docs.rs/ontology-registry/badge.svg)](https://docs.rs/ontology-registry)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![RobisonGroup](https://img.shields.io/badge/Robinson%20Group-blue)](https://robinsongroup.github.io/)

**A robust, thread-safe Rust library for managing the lifecycle of biological ontologies.**

`ontology-registry` automates the process of resolving, downloading, and caching ontology files (JSON, OBO, OWL). It
acts as a centralized local registry, ensuring that your applications always have access to the data they need without
redundant network requests or race conditions.

---

## ✨ Features

* **🚀 Smart Caching:** Automatically downloads ontologies from the [OBO Foundry](https://obofoundry.org/) and persists
  them locally. Subsequent requests load instantly from the disk.
* **🔄 Version Resolution:** resolving `Version::Latest` automatically queries [BioRegistry.io](https://bioregistry.io)
  to find the most recent semantic version.
* **🛡️ Thread-Safe & Atomic:** Built for concurrency. It uses Mutex locks and atomic file writes (downloading to `.tmp`
  first) to ensure you never read a corrupted or partially downloaded file.
* **🔌 Modular Architecture:** The logic is split into `MetadataProviding`, `OntologyProviding`, and `Registration`
  traits, allowing you to swap out backends if needed.
* **📂 Multiple Formats:** First-class support for `.json`, `.obo`, and `.owl` formats.

## 📦 Installation

Add this to your `Cargo.toml`:

```sh
cargo add ontology-registry
```

## Quick Start

```rust
use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
use ontology_registry::enums::{FileType, Version, SupportedOntology};
use ontology_registry::traits::OntologyRegistration;
use std::path::PathBuf;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup the registry with standard providers
    // In a real app, use a persistent path like "~/.cache/ontologies"
    let cache_dir = PathBuf::from("./local_ontology_cache");

    let registry = FileSystemOntologyRegistry::new(
        cache_dir,
        BioRegistryMetadataProvider::default(), // Resolves versions via BioRegistry.io
        OboLibraryProvider::default(),          // Downloads content from OBO Library
    );

    // 2. Register (Download & Cache)
    // This resolves 'Latest' to a specific date (e.g., "2024-01-01")
    let mut reader = registry.register(
        SupportedOntology::MONDO, // This can also just be a string "mondo"
        Version::Latest,
        FileType::Obo
    )?;

    // 3. Read the content
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    println!("Successfully loaded Mondo Ontology ({} bytes)", content.len());
    Ok(())
}
```

### Pinning a Specific Version

If you need reproducibility, you can request a specific version string.

```rust
use ontology_registry::enums::{FileType, Version};
use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
use ontology_registry::traits::OntologyRegistration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = FileSystemOntologyRegistry::default();

    let version = Version::Declared("2023-01-01".to_string());

    let reader = registry.register(
        "go",
        version,
        FileType::Owl
    )?;
}
```

## 📂 Supported Formats

| Enum Variant     | Extension | Description                  |
|:-----------------|:----------|:-----------------------------|
| `FileType::Json` | `.json`   | OBO Graph JSON format        |
| `FileType::Obo`  | `.obo`    | Standard OBO flat file       |
| `FileType::Owl`  | `.owl`    | Web Ontology Language format |
