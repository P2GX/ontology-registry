use crate::dataclasses::OntologyMetadata;
use crate::error::OntologyRegistryError;
use crate::traits::OntologyMetadataProvider;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BioRegistryResource {
    pub prefix: String,
    pub name: Option<String>,
    pub uri_format: Option<String>,
    pub homepage: Option<String>,
    pub version: Option<String>,
    pub download_owl: Option<String>,
    pub download_obo: Option<String>,
    pub download_json: Option<String>,
    pub download_rdf: Option<String>,
    pub preferred_prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BioRegistryMetadataProvider {
    api_url: String,
    client: Client,
}

impl BioRegistryMetadataProvider {
    pub fn new(api_url: &str) -> Self {
        let mut url = api_url.to_string();
        if !url.ends_with("/") {
            url += "/";
        }
        BioRegistryMetadataProvider {
            api_url: url,
            client: Client::new(),
        }
    }
}

impl Default for BioRegistryMetadataProvider {
    fn default() -> Self {
        BioRegistryMetadataProvider::new("https://bioregistry.io/api/")
    }
}
impl OntologyMetadataProvider for BioRegistryMetadataProvider {
    fn provide_metadata(
        &self,
        ontology_id: &str,
    ) -> Result<OntologyMetadata, OntologyRegistryError> {
        let url = self.api_url.clone() + "registry/" + ontology_id;

        let response = self
            .client
            .get(url.clone())
            .header("User-Agent", "phenoxtractor")
            .send()
            .map_err(|err| OntologyRegistryError::ProvidingMetadata {
                reason: err.to_string(),
            })?;

        let bio_bank_metadata: BioRegistryResource =
            response
                .json()
                .map_err(|_| OntologyRegistryError::ProvidingMetadata {
                    reason: format!("Cant convert to json for {ontology_id}"),
                })?;

        if let Some(version) = bio_bank_metadata.version {
            Ok(OntologyMetadata {
                ontology_id: bio_bank_metadata.prefix,
                version,
                json_file_location: bio_bank_metadata.download_json,
                owl_file_location: bio_bank_metadata.download_owl,
                obo_file_location: bio_bank_metadata.download_obo,
                title: bio_bank_metadata.name,
            })
        } else {
            Err(OntologyRegistryError::ProvidingMetadata {
                reason: format!("Version not found for {}", ontology_id),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn valid_response_json() -> String {
        r#"{
            "prefix": "mondo",
            "name": "Mondo Disease Ontology",
            "version": "2024-01-04",
            "download_owl": "http://purl.obolibrary.org/obo/mondo.owl",
            "download_json": "http://purl.obolibrary.org/obo/mondo.json",
            "download_obo": null
        }"#
        .to_string()
    }

    #[test]
    fn test_new_adds_trailing_slash() {
        let provider = BioRegistryMetadataProvider::new("https://bioregistry.io/api");
        assert_eq!(provider.api_url, "https://bioregistry.io/api/");

        let provider_existing = BioRegistryMetadataProvider::new("https://bioregistry.io/api/");
        assert_eq!(provider_existing.api_url, "https://bioregistry.io/api/");
    }

    #[test]
    fn test_provide_metadata_success() {
        let mut server = Server::new();
        let ontology_id = "mondo";

        let _m = server
            .mock("GET", "/registry/mondo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(valid_response_json())
            .create();

        let provider = BioRegistryMetadataProvider::new(&server.url());

        let result = provider.provide_metadata(ontology_id);

        assert!(result.is_ok());
        let metadata = result.unwrap();

        assert_eq!(metadata.ontology_id, "mondo");
        assert_eq!(metadata.version, "2024-01-04");
        assert_eq!(metadata.title.unwrap(), "Mondo Disease Ontology");
        assert_eq!(
            metadata.json_file_location.unwrap(),
            "http://purl.obolibrary.org/obo/mondo.json"
        );
        assert!(metadata.obo_file_location.is_none());
    }

    #[test]
    fn test_provide_metadata_missing_version() {
        let mut server = Server::new();
        let ontology_id = "chebi";

        // JSON response missing the "version" field
        let response_no_version = r#"{
            "prefix": "chebi",
            "name": "ChEBI"
        }"#;

        let _m = server
            .mock("GET", "/registry/chebi")
            .with_status(200)
            .with_body(response_no_version)
            .create();

        let provider = BioRegistryMetadataProvider::new(&server.url());
        let result = provider.provide_metadata(ontology_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            OntologyRegistryError::ProvidingMetadata { reason } => {
                assert!(reason.contains("Version not found"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_provide_metadata_malformed_json() {
        let mut server = Server::new();
        let ontology_id = "mondo";

        let _m = server
            .mock("GET", "/registry/mondo")
            .with_status(200)
            .with_body("invalid json {")
            .create();

        let provider = BioRegistryMetadataProvider::new(&server.url());
        let result = provider.provide_metadata(ontology_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            OntologyRegistryError::ProvidingMetadata { reason } => {
                assert!(reason.contains("Cant convert to json"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_provide_metadata_network_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/registry/mondo")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        let provider = BioRegistryMetadataProvider::new(&server.url());
        let result = provider.provide_metadata("mondo");

        assert!(result.is_err());
        match result.unwrap_err() {
            OntologyRegistryError::ProvidingMetadata { reason } => {
                assert!(reason.contains("Cant convert to json"));
            }
            _ => panic!("Wrong error type"),
        }
    }
}
