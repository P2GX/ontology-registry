//! # Core Interface Definitions
//!
//! This module defines the three fundamental traits that power the ontology registry.
//! These traits facilitate a separation of concerns between:
//!
//! 1.  **Metadata Resolution**: Finding out *where* an ontology is and *what* version is current.
//! 2.  **Content Provision**: The actual mechanism of downloading or fetching bytes.
//! 3.  **Registration**: The lifecycle management (saving, loading, deleting) of the files.
//!
//! Implementing these traits allows users to create custom backends (e.g., an S3-backed registry
//! or a custom internal metadata server) while keeping the rest of the application logic unchanged.

use crate::enums::{FileType, Version};
use crate::error::OntologyRegistryError;
use crate::ontology_metadata::OntologyMetadata;
use std::io::Read;

/// Defines how to retrieve metadata about an ontology.
///
/// Implementors of this trait are responsible for taking a high-level ID (e.g., "mondo")
/// and resolving it to concrete details like the latest version string, download URLs,
/// and title.
///
/// # Example
///
/// A `BioRegistry` implementation would query the BioRegistry.io API to fill this struct.
pub trait OntologyMetadataProviding {
    /// Fetch metadata for a specific ontology ID.
    ///
    /// # Errors
    /// Returns an error if the ID is unknown or the metadata source is unreachable.
    fn provide_metadata(
        &self,
        ontology_id: &str,
    ) -> Result<OntologyMetadata, OntologyRegistryError>;
}

/// Defines how to fetch the raw content (bytes) of an ontology file.
///
/// This trait is agnostic to the *content* of the file; it simply retrieves a stream of bytes
/// given a specific location and version.
pub trait OntologyProviding {
    /// Returns a reader for the requested ontology file.
    ///
    /// # Arguments
    /// * `ontology_id` - The ID of the ontology (e.g., "go").
    /// * `file_name` - The specific file name requested (e.g., "go.owl").
    /// * `version` - The resolved version string (e.g., "2024-01-01").
    fn provide_ontology(
        &self,
        ontology_id: &str,
        file_name: &str,
        version: &str,
    ) -> Result<impl Read, OntologyRegistryError>;
}

/// The primary interface for managing the ontology lifecycle.
///
/// This trait acts as a facade, coordinating the `OntologyMetadataProviding` and
/// `OntologyProviding` traits to download, cache, and manage files.
pub trait OntologyRegistration {
    /// Downloads and registers an ontology.
    ///
    /// If `Version::Latest` is passed, the implementor should use a metadata provider
    /// to resolve it to a concrete version string before downloading.
    fn register(
        &self,
        ontology_id: &str,
        version: Version,
        file_type: FileType,
    ) -> Result<impl Read, OntologyRegistryError>;

    /// Removes an ontology from the registry.
    fn unregister(
        &self,
        ontology_id: &str,
        version: Version,
        file_type: FileType,
    ) -> Result<(), OntologyRegistryError>;

    /// Retrieves a previously registered ontology.
    ///
    /// Returns `None` if the ontology is not found in the registry.
    fn get(&self, ontology_id: &str, version: Version, file_type: FileType) -> Option<impl Read>;

    /// Lists all ontologies currently stored in the registry.
    fn list(&self) -> Vec<String>;
}
