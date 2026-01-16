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
}
