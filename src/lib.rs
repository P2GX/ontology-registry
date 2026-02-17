//! # Ontology Registry
//!
//! A robust, thread-safe library for managing the lifecycle of biological ontologies.
//!
//! This crate provides a unified interface to resolve, download, cache, and load ontologies
//! (such as Mondo, GO, ChEBI) using standard formats (JSON, OBO, OWL). It is designed
//! with a modular architecture that separates metadata resolution, content provision,
//! and local storage management.
//!
//! ## Key Features
//!
//! * **Version Resolution:** Automatically resolves `Version::Latest` to specific semantic versions or dates using the BioRegistry API.
//! * **Local Caching:** Implements a file-system-based registry that caches ontologies to prevent redundant network requests.
//! * **Thread Safety:** The registry handles concurrent registration requests safely using atomic writes and mutex locks.
//! * **Standard Providers:** Includes built-in support for:
//!     * Metadata: [BioRegistry.io](https://bioregistry.io)
//!     * Content: [OBO Library](https://obolibrary.org)
//!
//! ## Usage
//!
//! The core entry point is the `FileSystemOntologyRegistry`. It requires a metadata provider
//! (to resolve versions) and an ontology provider (to download content).
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
//! use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
//! use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
//! use ontology_registry::enums::{FileType, Version};
//! use ontology_registry::traits::OntologyRegistration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Configure the storage location
//!     let cache_path = PathBuf::from("./local_ontology_cache");
//!
//!     // 2. Initialize the registry with standard providers
//!     let registry = FileSystemOntologyRegistry::new(
//!         cache_path,
//!         BioRegistryMetadataProvider::default(),
//!         OboLibraryProvider::default(),
//!     );
//!
//!     // 3. Register (download and cache) an ontology
//!     // This resolves the 'latest' version of Mondo and saves it as an OBO file.
//!     let _reader = registry.register(
//!         "mondo",
//!         Version::Latest,
//!         FileType::Obo
//!     )?;
//!
//!     // 4. Access the ontology later (offline)
//!     if let Some(_content) = registry.get("mondo", Version::Latest, FileType::Obo) {
//!         println!("Mondo ontology loaded successfully.");
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is built around three core traits defined in the [`traits`] module:
//!
//! 1.  **`OntologyMetadataProviding`**: Resolves high-level metadata (e.g., "What is the latest version of 'go'?").
//! 2.  **`OntologyProviding`**: Fetches the raw bytes of the ontology file from a remote source.
//! 3.  **`OntologyRegistration`**: High-level interface for registering (downloading/saving), unregistering, and retrieving ontologies.
//!
//! ## Modules
//!
//! * [`blocking`]: Contains concrete implementations of the providers and registry for synchronous (blocking) operations.
//! * [`ontology_metadata`]: Structs representing ontology metadata.
//! * [`enums`]: Enumerations for `Version` strategies and `FileType` formats.
//! * [`error`]: Crate-specific error types.
//! * [`traits`]: The core definitions ensuring modularity and extensibility.

pub mod blocking;
pub mod enums;
pub mod error;
pub mod ontology_metadata;
pub mod traits;
