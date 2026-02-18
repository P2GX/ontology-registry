use ontology_registry::{
    BioRegistryMetadataProvider, FileSystemOntologyRegistry, FileType, OboLibraryProvider,
    OntologyRegistration, SupportedOntology, Version,
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

    registry
        .register("uo", version.clone(), FileType::Json)
        .unwrap();
    let list = registry.list();
    assert_eq!(list.len(), 1);

    let mut file = registry.get("uo", version.clone(), FileType::Json).unwrap();
    let mut loaded_content = String::new();
    file.read_to_string(&mut loaded_content).unwrap();

    assert!(!loaded_content.is_empty());

    registry.unregister("uo", version, FileType::Json).unwrap();

    let list = registry.list();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_integration_declared_latest() {
    let version = Version::Latest;
    let file_format = FileType::Obo;
    let ontology_id = SupportedOntology::HP;
    let tmp_dir = TempDir::new().unwrap();
    let registry = FileSystemOntologyRegistry::new(
        tmp_dir.keep(),
        BioRegistryMetadataProvider::default(),
        OboLibraryProvider::default(),
    );

    registry
        .register(ontology_id, version.clone(), file_format)
        .unwrap();
    let list = registry.list();
    assert_eq!(list.len(), 1);
    assert!(list.first().unwrap().contains(&ontology_id.to_string()));
    assert!(list.first().unwrap().contains(file_format.as_file_ending()));

    let mut file = registry
        .get(ontology_id, version.clone(), file_format)
        .unwrap();

    let mut loaded_content = String::new();
    file.read_to_string(&mut loaded_content).unwrap();

    assert!(!loaded_content.is_empty());

    registry
        .unregister(ontology_id, version, file_format)
        .unwrap();

    let list = registry.list();
    assert_eq!(list.len(), 0);
}
