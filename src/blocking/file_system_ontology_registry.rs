use crate::RegistryKey;
use crate::enums::Version;
use crate::error::OntologyRegistryError;
use crate::traits::{OntologyMetadataProviding, OntologyProviding, OntologyRegistration};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};

#[derive(Debug)]
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
pub struct FileSystemOntologyRegistry<MDP, OP> {
    /// The root directory where ontology files will be stored.
    registry_path: PathBuf,
    /// The provider used to fetch ontology data during registration.
    ontology_provider: OP,
    /// The provider used to resolve ontology metadata (versions).
    metadata_provider: MDP,
    /// A lock used to ensure thread-safe file writing operations.
    write_lock: Mutex<()>,
}

impl<MDP, OP> FileSystemOntologyRegistry<MDP, OP> {
    fn create_temp_dir(&self) -> Result<PathBuf, OntologyRegistryError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let pid = process::id();
        let dir_name = format!("tmp_{}_{}", timestamp, pid);
        let tmp_dir = self.registry_path.join(dir_name);
        fs::create_dir_all(&tmp_dir).map_err(|_| OntologyRegistryError::NoRegistry)?;

        Ok(tmp_dir)
    }
}

impl<MDP: OntologyMetadataProviding, OP: OntologyProviding> FileSystemOntologyRegistry<MDP, OP> {
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
}

impl<MDP: OntologyMetadataProviding, OP: OntologyProviding> OntologyRegistration
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
    fn register(&self, registry_key: RegistryKey) -> Result<File, OntologyRegistryError> {
        if !self.registry_path.exists() {
            fs::create_dir_all(&self.registry_path)
                .map_err(|_| OntologyRegistryError::NoRegistry)?;
        }

        let mut out_path = self.registry_path.clone();

        let resolved_version = Version::Declared(
            self.resolve_version(registry_key.ontology_id(), registry_key.version())?,
        );

        let resolved_registry_key = RegistryKey::new(
            registry_key.ontology_id(),
            resolved_version,
            registry_key.file_type(),
        );

        let registry_file_name = resolved_registry_key.as_file_name();
        out_path.push(registry_file_name.clone());

        if out_path.exists() {
            return File::open(&out_path).map_err(|e| OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to open existing file '{}': {}",
                    out_path.display(),
                    e
                ),
            });
        }

        let provider_file_name = format!(
            "{}{}",
            resolved_registry_key.ontology_id(),
            resolved_registry_key.file_type().as_file_ending()
        );

        let mut ontology_reader = self.ontology_provider.provide_ontology(
            resolved_registry_key.ontology_id(),
            &provider_file_name,
            resolved_registry_key.version(),
        )?;

        let _guard =
            self.write_lock
                .lock()
                .map_err(|_| OntologyRegistryError::UnableToRegister {
                    reason: provider_file_name.clone(),
                })?;

        if out_path.exists() {
            return File::open(&out_path).map_err(|e| OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to open existing file '{}': {}",
                    out_path.display(),
                    e
                ),
            });
        }

        let temp_file_name = format!("{}.tmp", registry_file_name);
        let temp_dir = self.create_temp_dir()?;
        let temp_file_dir = temp_dir.join(temp_file_name);

        let mut temp_file =
            File::create(&temp_file_dir).map_err(|_| OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to create temporary file '{}'",
                    temp_file_dir.display()
                ),
            })?;

        let mut buffer: Vec<u8> = vec![];
        let _ = ontology_reader.read_to_end(&mut buffer).map_err(|err| {
            OntologyRegistryError::UnableToRegister {
                reason: err.to_string(),
            }
        })?;

        if temp_file.write_all(buffer.as_slice()).is_err() {
            let _ = fs::remove_file(&temp_file_dir);
            return Err(OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to write to temporary file '{}'",
                    temp_file_dir.display()
                ),
            });
        }

        drop(temp_file);

        fs::rename(&temp_file_dir, &out_path).map_err(|err| {
            let _ = fs::remove_file(&temp_file_dir);
            OntologyRegistryError::UnableToRegister {
                reason: format!(
                    "Unable to rename temporary file '{}'. Error: {}",
                    temp_file_dir.display(),
                    err
                ),
            }
        })?;

        fs::remove_dir_all(&temp_dir).map_err(|err| OntologyRegistryError::UnableToRegister {
            reason: format!(
                "Unable to delete temp directory '{}': {}",
                temp_dir.display(),
                err
            ),
        })?;

        File::open(&out_path).map_err(|err| OntologyRegistryError::UnableToRegister {
            reason: format!(
                "Unable to open final file '{}': {}",
                out_path.display(),
                err
            ),
        })
    }

    /// Removes an ontology from the local filesystem registry.
    ///
    /// Logs a warning if the version cannot be resolved or if deletion fails.
    /// This operation is thread-safe regarding the `write_lock`.
    fn unregister(&self, registry_key: RegistryKey) -> Result<(), OntologyRegistryError> {
        let resolved_version =
            self.resolve_version(registry_key.ontology_id(), registry_key.version())?;

        let resolved_registry_key = RegistryKey::new(
            registry_key.ontology_id(),
            Version::Declared(resolved_version),
            registry_key.file_type(),
        );

        let file_path = self
            .registry_path
            .clone()
            .join(resolved_registry_key.as_file_name());

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
    fn get(&self, registry_key: RegistryKey) -> Option<File> {
        let resolved_version = self
            .resolve_version(registry_key.ontology_id(), registry_key.version())
            .ok()?;

        let resolved_registry_key = RegistryKey::new(
            registry_key.ontology_id(),
            Version::Declared(resolved_version),
            registry_key.file_type(),
        );

        let file_path = self
            .registry_path
            .join(resolved_registry_key.as_file_name());

        File::open(file_path).ok()
    }

    /// Lists all files currently stored in the registry directory.
    ///
    /// Returns a vector of strings representing the absolute paths of the files.
    fn list(&self) -> Result<Vec<RegistryKey>, OntologyRegistryError> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(self.registry_path.clone()) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file()
                    && let Some(file_name) = path.file_name()
                    && let Some(file_name_str) = file_name.to_str()
                {
                    files.push(RegistryKey::from_file_name(file_name_str)?);
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileType;
    use crate::ontology_metadata::OntologyMetadata;
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[derive(Clone, Debug)]
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

    impl OntologyMetadataProviding for MockMetadataProvider {
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

    #[derive(Clone, Debug)]
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

    impl OntologyProviding for MockOntologyProvider {
        fn provide_ontology(
            &self,
            ontology_id: &str,
            _file_name: &str,
            _version: &Version,
        ) -> Result<impl Read, OntologyRegistryError> {
            Ok(Cursor::new(
                self.content
                    .get(ontology_id)
                    .cloned()
                    .ok_or(OntologyRegistryError::UnableToRegister {
                        reason: "Content not found".into(),
                    })?
                    .into_bytes(),
            ))
        }
    }

    #[test]
    fn test_register_declared_version_success() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        let content = "<rdf>content</rdf>";
        let metadata_mock = MockMetadataProvider::new();
        let ontology_mock = MockOntologyProvider::new().with_content("my_ontology", content);

        let registry =
            FileSystemOntologyRegistry::new(registry_path.clone(), metadata_mock, ontology_mock);
        let reg_key = RegistryKey::new(
            "my_ontology",
            Version::Declared("1.0".to_string()),
            FileType::Json,
        );
        let result = registry.register(reg_key);

        assert!(result.is_ok());
        let mut file = result.unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        assert_eq!(content, "<rdf>content</rdf>");
    }

    #[test]
    fn test_register_latest_version_resolution() {
        let temp_dir = tempdir().unwrap();
        let content = "latest_context";
        let metadata_mock = MockMetadataProvider::new().with_version("my_ontology", "2024-05-05");
        let ontology_mock = MockOntologyProvider::new().with_content("my_ontology", content);

        let registry = FileSystemOntologyRegistry::new(
            temp_dir.path().to_path_buf(),
            metadata_mock,
            ontology_mock,
        );

        let reg_key = RegistryKey::new("my_ontology", Version::Latest, FileType::Json);
        let result = registry.register(reg_key);

        assert!(result.is_ok());
        let mut file = result.unwrap();
        let mut loaded_content = String::new();
        file.read_to_string(&mut loaded_content).unwrap();

        assert_eq!(loaded_content, content);
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

        let reg_key = RegistryKey::new(
            "test_ont",
            Version::Declared("1.0".to_string()),
            FileType::Json,
        );
        let result = registry.register(reg_key);

        assert!(result.is_ok());

        let content = fs::read_to_string(existing_file_path).unwrap();
        assert_eq!(content, "old_content");
    }

    #[test]
    fn test_get_existing_ontology() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();
        let target_path = registry_path.join("findme@2024-05-05.json");

        let content = "data";
        fs::write(&target_path, content).unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        let reg_key = RegistryKey::new(
            "findme",
            Version::Declared("2024-05-05".to_string()),
            FileType::Json,
        );

        let result = registry.get(reg_key);

        let mut file = result.unwrap();
        let mut loaded_content = String::new();
        file.read_to_string(&mut loaded_content).unwrap();

        assert_eq!(loaded_content, content);
    }

    #[test]
    fn test_get_non_existent_ontology() {
        let temp_dir = tempdir().unwrap();
        let registry = FileSystemOntologyRegistry::new(
            temp_dir.path().to_path_buf(),
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );
        let reg_key = RegistryKey::new(
            "missing",
            Version::Declared("9.9".to_string()),
            FileType::Obo,
        );
        let result = registry.get(reg_key);

        assert!(result.is_none());
    }

    #[test]
    fn test_unregister_removes_file() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();
        let target_path = registry_path.join("todelete@2024-05-05.json");
        fs::write(&target_path, "delete_me").unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        assert!(target_path.exists());

        let reg_key = RegistryKey::new(
            "todelete",
            Version::Declared("2024-05-05".to_string()),
            FileType::Json,
        );

        registry.unregister(reg_key).unwrap();

        assert!(!target_path.exists());
    }

    #[test]
    fn test_list_files() {
        let temp_dir = tempdir().unwrap();
        let registry_path = temp_dir.path().to_path_buf();

        fs::write(registry_path.join("A@1.0.json"), "").unwrap();
        fs::write(registry_path.join("B@2.0.obo"), "").unwrap();
        fs::create_dir(registry_path.join("subdir")).unwrap();

        let registry = FileSystemOntologyRegistry::new(
            registry_path,
            MockMetadataProvider::new(),
            MockOntologyProvider::new(),
        );

        let files = registry.list().unwrap();

        assert_eq!(files.len(), 2);
        assert!(
            files.iter().any(|f| *f
                == RegistryKey::new("A", Version::Declared("1.0".to_string()), FileType::Json))
        );
        assert!(
            files.iter().any(|f| *f
                == RegistryKey::new("B", Version::Declared("2.0".to_string()), FileType::Obo))
        );
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
                let reg_key = RegistryKey::new(
                    "shared",
                    Version::Declared("1.0".to_string()),
                    FileType::Json,
                );
                reg_clone.register(reg_key).map(|_| ())
            }));
        }

        for handle in handles {
            let res = handle.join().unwrap();
            assert!(res.is_ok());
        }
    }
}
