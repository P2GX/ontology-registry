use crate::enums::{FileType, Version};
use crate::error::OntologyRegistryError;
use crate::traits::{OntologyMetadataProvider, OntologyProvider, OntologyRegistry};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// A registry implementation that manages ontologies as files on the local filesystem.
///
/// This registry acts as a local cache/storage layer. When an ontology is registered,
/// it is fetched from the configured `OntologyProvider` and saved to the `registry_path`.
///
/// # Features
///
/// * **Thread Safety:** Registration is guarded by a mutex to prevent race conditions when
///   multiple threads attempt to download/write the same ontology simultaneously.
/// * **Atomic Writes:** Files are written to a temporary location first and then renamed.
///   This ensures that the registry never contains partially written or corrupted ontology files.
/// * **Version Resolution:** Supports resolving `Version::Latest` dynamically via the
///   `OntologyMetadataProvider`.
///
/// # Type Parameters
///
/// * `MDP`: **OntologyMetadataProvider** - Used to resolve version information (e.g., determining what "Latest" maps to).
/// * `OP`: **OntologyProvider** - Used to fetch the actual ontology content (bytes) from a remote or external source.
pub struct FileSystemOntologyRegistry<MDP: OntologyMetadataProvider, OP: OntologyProvider> {
    /// The root directory where ontology files will be stored.
    registry_path: PathBuf,
    /// The provider used to fetch ontology data during registration.
    ontology_provider: OP,
    /// The provider used to resolve ontology metadata (versions).
    metadata_provider: MDP,
    /// A lock used to ensure thread-safe file writing operations.
    write_lock: Mutex<()>,
}

impl<MDP: OntologyMetadataProvider, OP: OntologyProvider> FileSystemOntologyRegistry<MDP, OP> {
    /// Creates a new `FileSystemOntologyRegistry`.
    ///
    /// # Arguments
    ///
    /// * `registry_path` - The directory path where ontologies will be saved.
    /// * `metadata_provider` - The service to query for ontology version metadata.
    /// * `ontology_provider` - The service to download ontology content from.
    pub fn new(registry_path: PathBuf, metadata_provider: MDP, ontology_provider: OP) -> Self {
        FileSystemOntologyRegistry {
            registry_path,
            metadata_provider,
            ontology_provider,
            write_lock: Mutex::new(()),
        }
    }

    fn resolve_version(
        &self,
        ontology_id: &str,
        version: &Version,
    ) -> Result<String, OntologyRegistryError> {
        match version {
            Version::Latest => {
                let meta_data = self.metadata_provider.provide_metadata(ontology_id)?;
                Ok(meta_data.version)
            }
            Version::Declared(v) => Ok(v.to_string()),
        }
    }

    fn construct_registry_file_name(
        &self,
        ontology_id: &str,
        version: &str,
        file_type: &FileType,
    ) -> String {
        format!("{}_{}{}", ontology_id, version, file_type.as_file_ending())
    }
}

impl<MDP: OntologyMetadataProvider, OP: OntologyProvider> OntologyRegistry<PathBuf>
    for FileSystemOntologyRegistry<MDP, OP>
{
    /// Registers an ontology by downloading it and saving it to the local filesystem.
    ///
    /// # Behavior
    ///
    /// 1.  Resolves the version (fetching metadata if `Latest` is requested).
    /// 2.  Checks if the file already exists locally. If so, returns the path immediately.
    /// 3.  Fetches the content from the `OntologyProvider`.
    /// 4.  **Atomic Write:** Writes content to a `.tmp` file, then renames it to the final destination.
    ///     This prevents other threads/processes from reading incomplete files.
    ///
    /// # Errors
    ///
    /// Returns `OntologyRegistryError` if:
    /// * The metadata cannot be resolved.
    /// * The registry directory cannot be created.
    /// * The ontology provider fails to return data.
    /// * File I/O operations (creation, writing, renaming) fail.
    fn register(
        &self,
        ontology_id: &str,
        version: &Version,
        file_type: &FileType,
    ) -> Result<PathBuf, OntologyRegistryError> {
        let mut out_path = self.registry_path.clone();
        let resolved_version = self.resolve_version(ontology_id, version)?;

        let registry_file_name =
            self.construct_registry_file_name(ontology_id, &resolved_version, file_type);
        out_path.push(registry_file_name.clone());

        if out_path.exists() {
            return Ok(out_path);
        }

        if !self.registry_path.exists() {
            fs::create_dir_all(&self.registry_path)
                .map_err(|_| OntologyRegistryError::NoRegistry)?;
        }

        let provider_file_name = format!("{}{}", ontology_id, file_type.as_file_ending());

        let resp = self.ontology_provider.provide_ontology(
            ontology_id,
            &provider_file_name,
            &resolved_version,
        )?;

        let _guard =
            self.write_lock
                .lock()
                .map_err(|_| OntologyRegistryError::UnableToRegister {
                    reason: provider_file_name.clone(),
                })?;

        if out_path.exists() {
            return Ok(out_path);
        }

        let temp_file_name = format!("{}.tmp", registry_file_name);
        let mut temp_path = self.registry_path.clone();
        temp_path.push(&temp_file_name);

        let mut temp_file =
            fs::File::create(&temp_path).map_err(|_| OntologyRegistryError::UnableToRegister {
                reason: format!("Unable to create temporary file '{}'", temp_path.display()),
            })?;

        if temp_file.write_all(resp.as_bytes()).is_err() {
            let _ = fs::remove_file(&temp_path);
            return Err(OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to write to temporary file '{}'",
                    temp_path.display()
                ),
            });
        }

        fs::rename(&temp_path, &out_path).map_err(|_| {
            let _ = fs::remove_file(&temp_path);
            OntologyRegistryError::UnableToRegister {
                reason: format!("Unable to rename temporary file '{}'", temp_path.display()),
            }
        })?;

        Ok(out_path)
    }

    /// Removes an ontology from the local filesystem registry.
    ///
    /// Logs a warning if the version cannot be resolved or if deletion fails.
    /// This operation is thread-safe regarding the `write_lock`.
    fn unregister(
        &self,
        ontology_id: &str,
        version: &Version,
        file_type: &FileType,
    ) -> Result<(), OntologyRegistryError> {
        let resolved_version = self.resolve_version(ontology_id, version)?;

        let file_path = self
            .registry_path
            .clone()
            .join(self.construct_registry_file_name(ontology_id, &resolved_version, file_type));

        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|err| OntologyRegistryError::UnableToRegister {
                reason: format!("Unable to rename temporary file '{}'", err),
            })?;
        }

        Ok(())
    }

    /// Retrieves the local filesystem path for a specific ontology.
    ///
    /// Returns `None` if the ontology is not currently found in the local registry
    /// or if the version could not be resolved.
    fn get(&self, ontology_id: &str, version: &Version, file_type: &FileType) -> Option<PathBuf> {
        let resolved_version = self.resolve_version(ontology_id, version);

        if resolved_version.is_err() {
            return None;
        }

        let file_path = self
            .registry_path
            .clone()
            .join(self.construct_registry_file_name(
                ontology_id,
                &resolved_version.unwrap(),
                file_type,
            ));

        if !file_path.exists() {
            return None;
        }

        Some(file_path)
    }

    /// Lists all files currently stored in the registry directory.
    ///
    /// Returns a vector of strings representing the absolute paths of the files.
    fn list(&self) -> Vec<String> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(self.registry_path.clone()) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file()
                    && let Some(path_str) = path.to_str()
                {
                    files.push(path_str.to_string());
                }
            }
        }

        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataclasses::OntologyMetadata;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[derive(Clone)]
    struct MockMetadataProvider {
        data: HashMap<String, String>,
    }

    impl MockMetadataProvider {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
        fn with_version(mut self, id: &str, version: &str) -> Self {
            self.data.insert(id.to_string(), version.to_string());
            self
        }
    }

    impl OntologyMetadataProvider for MockMetadataProvider {
        fn provide_metadata(
            &self,
            ontology_id: &str,
        ) -> Result<OntologyMetadata, OntologyRegistryError> {
            match self.data.get(ontology_id) {
                Some(v) => Ok(OntologyMetadata {
                    ontology_id: "".to_string(),
                    version: v.clone(),
                    json_file_location: None,
                    owl_file_location: None,
                    obo_file_location: None,
                    title: None,
                }),
                None => Err(OntologyRegistryError::UnableToRegister {
                    reason: "Metadata not found".into(),
                }),
            }
        }
    }

    struct MockOntologyProvider {
        content: HashMap<String, String>,
    }

    impl MockOntologyProvider {
        fn new() -> Self {
            Self {
                content: HashMap::new(),
            }
        }
        fn with_content(mut self, id: &str, content: &str) -> Self {
            self.content.insert(id.to_string(), content.to_string());
            self
        }
    }

    impl OntologyProvider for MockOntologyProvider {
        fn provide_ontology(
            &self,
            ontology_id: &str,
            _file_name: &str,
            _version: &str,
        ) -> Result<String, OntologyRegistryError> {
            self.content
                .get(ontology_id)
                .cloned()
                .ok_or(OntologyRegistryError::UnableToRegister {
                    reason: "Content not found".into(),
                })
        }
    }

    #[test]
    fn test_register_declared_version_success() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        let metadata_mock = MockMetadataProvider::new();
        let ontology_mock =
            MockOntologyProvider::new().with_content("my_ontology", "<rdf>content</rdf>");

        let registry =
            FileSystemOntologyRegistry::new(registry_path.clone(), metadata_mock, ontology_mock);

        let result = registry.register(
            "my_ontology",
            &Version::Declared("1.0".to_string()),
            &FileType::Json,
        );

        // Assert
        assert!(result.is_ok());
        let path = result.unwrap();

        assert!(path.to_string_lossy().ends_with("my_ontology_1.0.json"));
        assert!(path.exists());

        // Check content
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "<rdf>content</rdf>");
    }

    #[test]
    fn test_register_latest_version_resolution() {
        // Setup
        let temp_dir = tempdir().unwrap();

        let metadata_mock = MockMetadataProvider::new().with_version("my_ontology", "2024-05-05");
        let ontology_mock =
            MockOntologyProvider::new().with_content("my_ontology", "latest_content");

        let registry = FileSystemOntologyRegistry::new(
            temp_dir.path().to_path_buf(),
            metadata_mock,
            ontology_mock,
        );

        let result = registry.register("my_ontology", &Version::Latest, &FileType::Json);

        // Assert
        assert!(result.is_ok());
        let path = result.unwrap();

        // Should resolve to 2.5 based on metadata provider
        assert!(
            path.to_string_lossy()
                .ends_with("my_ontology_2024-05-05.json")
        );
        assert_eq!(fs::read_to_string(path).unwrap(), "latest_content");
    }

    #[test]
    fn test_register_skips_existing_file() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        let existing_file_path = registry_path.join("test_ont_1.0.rdf");
        fs::write(&existing_file_path, "old_content").unwrap();

        let metadata_mock = MockMetadataProvider::new();
        let ontology_mock = MockOntologyProvider::new().with_content("test_ont", "new_content");

        let registry = FileSystemOntologyRegistry::new(registry_path, metadata_mock, ontology_mock);

        let result = registry.register(
            "test_ont",
            &Version::Declared("1.0".to_string()),
            &FileType::Json,
        );

        assert!(result.is_ok());

        // Verify content was NOT overwritten
        let content = fs::read_to_string(existing_file_path).unwrap();
        assert_eq!(content, "old_content");
    }

    #[test]
    fn test_get_existing_ontology() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();
        let target_path = registry_path.join("findme_2024-05-05.json");

        fs::write(&target_path, "data").unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        let result = registry.get(
            "findme",
            &Version::Declared("2024-05-05".to_string()),
            &FileType::Json,
        );

        assert!(result.is_some());
        assert_eq!(result.unwrap(), target_path);
    }

    #[test]
    fn test_get_non_existent_ontology() {
        let temp_dir = tempdir().unwrap();
        let registry = FileSystemOntologyRegistry::new(
            temp_dir.path().to_path_buf(),
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        let result = registry.get(
            "missing",
            &Version::Declared("9.9".to_string()),
            &FileType::Obo,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_unregister_removes_file() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();
        let target_path = registry_path.join("todelete_2024-05-05.json");
        fs::write(&target_path, "delete_me").unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        assert!(target_path.exists());

        registry
            .unregister(
                "todelete",
                &Version::Declared("2024-05-05".to_string()),
                &FileType::Json,
            )
            .unwrap();

        assert!(!target_path.exists());
    }

    #[test]
    fn test_list_files() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        fs::write(registry_path.join("A_1.0.json"), "").unwrap();
        fs::write(registry_path.join("B_2.0.obo"), "").unwrap();
        fs::create_dir(registry_path.join("subdir")).unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        let files = registry.list();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.contains("A_1.0.json")));
        assert!(files.iter().any(|f| f.contains("B_2.0.obo")));
    }

    #[test]
    fn test_concurrency_locks() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        let metadata = MockMetadataProvider::new();
        let ontology = MockOntologyProvider::new().with_content("shared", "content");

        let registry = Arc::new(FileSystemOntologyRegistry::new(
            registry_path,
            metadata,
            ontology,
        ));

        let mut handles = vec![];

        for _ in 0..5 {
            let reg_clone = registry.clone();
            handles.push(std::thread::spawn(move || {
                reg_clone.register(
                    "shared",
                    &Version::Declared("1.0".to_string()),
                    &FileType::Json,
                )
            }));
        }

        for handle in handles {
            let res = handle.join().unwrap();
            assert!(res.is_ok());
        }
    }
}
