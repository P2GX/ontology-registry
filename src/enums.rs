use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default, Serialize, Deserialize)]
pub enum Version {
    #[default]
    Latest,
    Declared(String),
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Latest => f.write_str("latest"),
            Version::Declared(v) => f.write_str(v),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Json,
    Obo,
    Owl,
}

impl FileType {
    pub fn as_file_ending(&self) -> &'static str {
        match self {
            FileType::Json => ".json",
            FileType::Obo => ".obo",
            FileType::Owl => ".owl",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
/// This enum contains ontologies that have been validated to work with ontology registry. Others might also work.
pub enum SupportedOntology {
    HP,
    MONDO,
    MAXO,
    UO,
    UBERON,
    PATO,
    /// Only supports OWL and OBO
    NCIT,
    RO,
    GENO,
    GO,
}

impl Display for SupportedOntology {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedOntology::HP => f.write_str("hp"),
            SupportedOntology::MONDO => f.write_str("mondo"),
            SupportedOntology::MAXO => f.write_str("maxo"),
            SupportedOntology::UO => f.write_str("uo"),
            SupportedOntology::UBERON => f.write_str("uberon"),
            SupportedOntology::PATO => f.write_str("pato"),
            SupportedOntology::NCIT => f.write_str("ncit"),
            SupportedOntology::RO => f.write_str("ro"),
            SupportedOntology::GENO => f.write_str("geno"),
            SupportedOntology::GO => f.write_str("go"),
        }
    }
}

impl From<SupportedOntology> for String {
    fn from(value: SupportedOntology) -> Self {
        value.to_string()
    }
}
