use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
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

#[derive(Debug, PartialEq)]
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
