# Example

```rust
let version = Version::Latest;
let ontology_id = "hp".to_string();
let tmp_dir = TempDir::new().unwrap();
let registry = FileSystemOntologyRegistry::new(tmp_dir.keep(), BioRegistryMetadataProvider::default (), ObolibraryProvider::default ());

let file_dir = registry.register( & ontology_id, & version, & FileType::Json).unwrap();

// Load ontology
```
