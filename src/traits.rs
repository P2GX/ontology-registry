use crate::dataclasses::OntologyMetadata;
use crate::enums::{FileType, Version};
use crate::error::OntologyRegistryError;
use std::io::Read;

pub trait OntologyMetadataProvider {
    fn provide_metadata(
        &self,
        ontology_id: &str,
    ) -> Result<OntologyMetadata, OntologyRegistryError>;
}

pub trait OntologyProvider {
    fn provide_ontology(
        &self,
        ontology_id: &str,
        file_name: &str,
        version: &str,
    ) -> Result<String, OntologyRegistryError>;
}

pub trait OntologyRegistry {
    fn register(
        &self,
        ontology_id: impl Into<String>,
        version: Version,
        file_type: FileType,
    ) -> Result<impl Read, OntologyRegistryError>;
    fn unregister(
        &self,
        ontology_id: impl Into<String>,
        version: Version,
        file_type: FileType,
    ) -> Result<(), OntologyRegistryError>;
    fn get(
        &self,
        ontology_id: impl Into<String>,
        version: Version,
        file_type: FileType,
    ) -> Option<impl Read>;
    fn list(&self) -> Vec<String>;
}
