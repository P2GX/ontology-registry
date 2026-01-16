use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
use ontology_registry::enums::{FileType, Version};
use ontology_registry::traits::OntologyRegistry;
use std::fs;
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

    registry.register("uo", &version, &FileType::Json).unwrap();
    let list = registry.list();
    assert_eq!(list.len(), 1);

    let path = registry.get("uo", &version, &FileType::Json).unwrap();
    let file = fs::read(path).unwrap();
    assert!(!file.is_empty());

    registry.unregister("uo", &version, &FileType::Json);

    let list = registry.list();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_integration_declared_latest() {
    let version = Version::Latest;
    let ontology_id = "hp".to_string();
    let tmp_dir = TempDir::new().unwrap();
    let registry = FileSystemOntologyRegistry::new(
        tmp_dir.keep(),
        BioRegistryMetadataProvider::default(),
        OboLibraryProvider::default(),
    );

    registry
        .register(&ontology_id, &version, &FileType::Json)
        .unwrap();
    let list = registry.list();
    assert_eq!(list.len(), 1);

    let path = registry
        .get(&ontology_id, &version, &FileType::Json)
        .unwrap();
    let file = fs::read(path).unwrap();
    assert!(!file.is_empty());

    registry.unregister(&ontology_id, &version, &FileType::Json);

    let list = registry.list();
    assert_eq!(list.len(), 0);
}
