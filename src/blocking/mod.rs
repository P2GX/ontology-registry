//! # Synchronous Implementations
//!
//! This module provides the standard, blocking implementations of the registry traits.
//! These components are designed to be used together to create a fully functional
//! file-system-based ontology registry.
//!
//! ## Components
//!
//! * **[`bio_registry_metadata_provider`]:**
//!   Connects to the [BioRegistry.io](https://bioregistry.io) API to resolve ontology
//!   versions and metadata. It maps `Version::Latest` to the most recent release
//!   date available in the registry.
//!
//! * **[`obolib_ontology_provider`]:**
//!   Downloads ontology files directly from the [OBO Library](https://obolibrary.org).
//!   It constructs URLs based on the OBO library's standard release structure
//!   (e.g., `.../obo/mondo/releases/2024-01-01/mondo.owl`).
//!
//! * **[`file_system_ontology_registry`]:**
//!   The main coordinator. It persists downloaded ontologies to a local directory.
//!   It includes robust handling for:
//!     * **Concurrency:** Uses `Mutex` locks to prevent race conditions when multiple threads try to download the same ontology.
//!     * **Atomic Writes:** Downloads to temporary files (`.tmp`) and renames them only upon successful completion to ensure data integrity.
//!
//! ## Example Configuration
//!
//! ```rust,no_run
//! use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
//! use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
//! use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
//! use std::path::PathBuf;
//!
//! let registry = FileSystemOntologyRegistry::new(
//!     PathBuf::from("/tmp/ontologies"),
//!     BioRegistryMetadataProvider::default(),
//!     OboLibraryProvider::default(),
//! );
//! ```

pub mod bio_registry_metadata_provider;
pub mod file_system_ontology_registry;
pub mod obolib_ontology_provider;
