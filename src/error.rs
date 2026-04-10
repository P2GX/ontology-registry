use thiserror::Error;

#[derive(Debug, Error)]
pub enum OntologyRegistryError {
    #[error("Unable to provide Metadata: {reason}")]
    ProvidingMetadata { reason: String },
    #[error("Unable to provide Metadata: {reason}")]
    ProvidingOntology { reason: String },
    #[error("Unable to create registry")]
    NoRegistry,
    #[error("Unable to register ontology: {reason}")]
    UnableToRegister { reason: String },
    #[error("Unable to unregister ontology: {reason}")]
    UnableToUnregister { reason: String },
    #[error("Expected format: ontology_id@version.file_type. Found: {raw_key}")]
    CantParseRegistryKey { raw_key: String },
    #[error("Expected format: .json, .owl. obo. Found: {raw_format}")]
    CantParseFileFormat { raw_format: String },
}
