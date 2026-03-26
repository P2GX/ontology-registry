use crate::{FileType, OntologyRegistryError, Version};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RegistryKey {
    ontology_id: String,
    version: Version,
    file_type: FileType,
}

impl RegistryKey {
    pub fn new(
        ontology_id: impl Into<String>,
        version: impl Into<Version>,
        file_type: impl Into<FileType>,
    ) -> Self {
        Self {
            ontology_id: ontology_id.into(),
            version: version.into(),
            file_type: file_type.into(),
        }
    }
    pub fn ontology_id(&self) -> &str {
        &self.ontology_id
    }
    pub fn version(&self) -> &Version {
        &self.version
    }
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
    pub fn into_parts(self) -> (String, Version, FileType) {
        (self.ontology_id, self.version, self.file_type)
    }
    pub fn as_file_name(&self) -> String {
        format!(
            "{}@{}{}",
            self.ontology_id,
            self.version,
            self.file_type.as_file_ending()
        )
    }

    pub fn from_file_name(file_name: &str) -> Result<RegistryKey, OntologyRegistryError> {
        let parse_err = || OntologyRegistryError::CantParseRegistryKey {
            raw_key: file_name.to_string(),
        };

        let (ontology_id, rest) = file_name
            .splitn(2, '@')
            .collect_tuple()
            .ok_or_else(parse_err)?;

        let file_type = FileType::all()
            .into_iter()
            .find(|ft| rest.ends_with(ft.as_file_ending()))
            .ok_or_else(parse_err)?;

        let version_str = rest
            .strip_suffix(file_type.as_file_ending())
            .ok_or_else(parse_err)?;

        let version = match version_str {
            "latest" => Version::Latest,
            v => Version::Declared(v.to_string()),
        };

        Ok(Self::new(ontology_id.to_string(), version, file_type))
    }
}
impl<S, V, F> From<(S, V, F)> for RegistryKey
where
    S: Into<String>,
    V: Into<Version>,
    F: Into<FileType>,
{
    fn from((ontology_id, version, file_type): (S, V, F)) -> Self {
        Self::new(ontology_id, version, file_type)
    }
}

impl std::fmt::Display for RegistryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.ontology_id,
            self.version,
            self.file_type.as_file_ending()
        )
    }
}

impl FromStr for RegistryKey {
    type Err = OntologyRegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 3 {
            return Err(OntologyRegistryError::CantParseRegistryKey {
                raw_key: s.to_string(),
            });
        }

        let ontology_id = parts[0].to_string();

        let version = match parts[1] {
            "latest" => Version::Latest,
            v => Version::Declared(v.to_string()),
        };

        let file_type = match parts[2] {
            "json" => FileType::Json,
            "owl" => FileType::Owl,
            "obo" => FileType::Obo,
            _ => {
                return Err(OntologyRegistryError::CantParseFileFormat {
                    raw_format: parts[2].to_string(),
                });
            }
        };

        Ok(Self::new(ontology_id, version, file_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file_name() {
        let file_name = "uo@4v.json";

        let key = RegistryKey::from_file_name(file_name).unwrap();

        assert_eq!(key.version(), &Version::Declared("4v".to_string()));
        assert_eq!(key.ontology_id, "uo");
        assert_eq!(key.file_type, FileType::Json);
    }
    #[test]
    fn test_from_file_name_semantic_version() {
        let file_name = "uo@4.0.3.json";

        let key = RegistryKey::from_file_name(file_name).unwrap();

        assert_eq!(key.version(), &Version::Declared("4.0.3".to_string()));
        assert_eq!(key.ontology_id, "uo");
        assert_eq!(key.file_type, FileType::Json);
    }

    #[test]
    fn test_from_file_name_owl() {
        let key = RegistryKey::from_file_name("uo@4v.owl").unwrap();
        assert_eq!(key.file_type, FileType::Owl);
    }

    #[test]
    fn test_from_file_name_obo() {
        let key = RegistryKey::from_file_name("go@2024.obo").unwrap();
        assert_eq!(key.file_type, FileType::Obo);
    }

    #[test]
    fn test_from_file_name_long_ontology_id() {
        let key = RegistryKey::from_file_name("my_long_ontology@1.0.json").unwrap();
        assert_eq!(key.ontology_id, "my_long_ontology");
    }

    #[test]
    fn test_from_file_name_uppercase_ontology_id() {
        let key = RegistryKey::from_file_name("GO@1.0.json").unwrap();
        assert_eq!(key.ontology_id, "GO");
    }

    #[test]
    fn test_from_file_name_version_with_only_major() {
        let key = RegistryKey::from_file_name("uo@1.json").unwrap();
        assert_eq!(key.version(), &Version::Declared("1".to_string()));
    }

    #[test]
    fn test_from_file_name_version_with_date() {
        let key = RegistryKey::from_file_name("go@2024-01-15.json").unwrap();
        assert_eq!(key.version(), &Version::Declared("2024-01-15".to_string()));
    }

    #[test]
    fn test_as_file_name() {
        let reg_key = RegistryKey::new("uo", Version::Declared("4v".to_string()), FileType::Json);
        let reg_key_str = reg_key.as_file_name();
        assert_eq!(RegistryKey::from_file_name(&reg_key_str).unwrap(), reg_key);
    }
}
