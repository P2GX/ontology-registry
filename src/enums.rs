use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default, Serialize, Deserialize)]
pub enum Version {
    #[default]
    Latest,
    Declared(String),
}

impl From<&str> for Version {
    fn from(value: &str) -> Self {
        Version::Declared(value.to_string())
    }
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
