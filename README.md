# Example

```rust
let version = Version::Declared("2026-01-16".to_string());
let tmp_dir = TempDir::new().unwrap();
let registry = FileSystemOntologyRegistry::new(
tmp_dir.keep(),
BioRegistryMetadataProvider::default (),
OboLibraryProvider::default (),
);

let res = registry
.register("uo", version.clone(), FileType::Json)
.unwrap();

// Load ontology
```
