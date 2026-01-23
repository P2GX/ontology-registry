# Example

```rust
use ontology_registry::blocking::bio_registry_metadata_provider::BioRegistryMetadataProvider;
use ontology_registry::blocking::file_system_ontology_registry::FileSystemOntologyRegistry;
use ontology_registry::blocking::obolib_ontology_provider::OboLibraryProvider;
use ontology_registry::enums::{FileType, Version};

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
