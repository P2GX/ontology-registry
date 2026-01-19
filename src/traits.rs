use crate::dataclasses::OntologyMetadata;
use crate::enums::{FileType, Version};
use crate::error::OntologyRegistryError;

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

pub trait OntologyRegistry<RegistryEntry> {
    fn register(
        &self,
        ontology_id: &str,
        version: &Version,
        file_type: &FileType,
    ) -> Result<RegistryEntry, OntologyRegistryError>;
    fn unregister(
        &self,
        ontology_id: &str,
        version: &Version,
        file_type: &FileType,
    ) -> Result<(), OntologyRegistryError>;
    fn get(
        &self,
        ontology_id: &str,
        version: &Version,
        file_type: &FileType,
    ) -> Option<RegistryEntry>;
    fn list(&self) -> Vec<String>;
}
