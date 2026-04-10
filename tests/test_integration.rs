use ontology_registry::{
    BioRegistryMetadataProvider, FileSystemOntologyRegistry, FileType, OboLibraryProvider,
    OntologyRegistration, RegistryKey, SupportedOntology, Version,
};
use std::io::Read;
use tempfile::TempDir;

#[test]
fn test_integration_declared_version() {
    let version = Version::Declared("2026-01-16".to_string());
    let tmp_dir = TempDir::new().unwrap();
    let registry = FileSystemOntologyRegistry::new(
        tmp_dir.keep(),
        BioRegistryMetadataProvider::default(),
        OboLibraryProvider::default(),
    );

    let reg_key = RegistryKey::new("uo", version.clone(), FileType::Json);
    registry.register(reg_key.clone()).unwrap();
    let list = registry.list().unwrap();
    assert_eq!(list.len(), 1);

    let mut file = registry.get(reg_key.clone()).unwrap();
    let mut loaded_content = String::new();
    file.read_to_string(&mut loaded_content).unwrap();

    assert!(!loaded_content.is_empty());

    registry.unregister(reg_key).unwrap();

    let list = registry.list().unwrap();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_integration_declared_latest() {
    let reg_key = RegistryKey::new(SupportedOntology::HP, Version::Latest, FileType::Obo);

    let tmp_dir = TempDir::new().unwrap();
    let registry = FileSystemOntologyRegistry::new(
        tmp_dir.keep(),
        BioRegistryMetadataProvider::default(),
        OboLibraryProvider::default(),
    );

    registry.register(reg_key.clone()).unwrap();
    let list = registry.list().unwrap();
    assert_eq!(list.len(), 1);

    let retrieved_reg_key = list.first().unwrap();
    assert_eq!(retrieved_reg_key.file_type(), reg_key.file_type());
    assert_eq!(retrieved_reg_key.ontology_id(), reg_key.ontology_id());

    let mut file = registry.get(reg_key.clone()).unwrap();

    let mut loaded_content = String::new();
    file.read_to_string(&mut loaded_content).unwrap();

    assert!(!loaded_content.is_empty());

    registry.unregister(reg_key).unwrap();

    let list = registry.list().unwrap();
    assert_eq!(list.len(), 0);
}
