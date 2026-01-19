use crate::error::OntologyRegistryError;
use crate::traits::OntologyProvider;
use log::debug;

pub struct OboLibraryProvider {
    base_url: String,
    client: reqwest::blocking::Client,
}
impl Default for OboLibraryProvider {
    fn default() -> Self {
        OboLibraryProvider {
            base_url: "https://purl.obolibrary.org/obo".to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl OboLibraryProvider {
    pub fn new(base_url: String) -> Self {
        OboLibraryProvider {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl OntologyProvider for OboLibraryProvider {
    fn provide_ontology(
        &self,
        ontology_id: &str,
        file_name: &str,
        version: &str,
    ) -> Result<String, OntologyRegistryError> {
        let url = format!(
            "{}/{}/releases/{}/{}",
            self.base_url, ontology_id, version, file_name
        );

        let resp = self.client.get(url.clone()).send();

        match resp {
            Ok(response) => {
                let response = response.error_for_status().map_err(|err| {
                    OntologyRegistryError::ProvidingOntology {
                        reason: err.to_string(),
                    }
                })?;

                debug!(
                    "Got file '{}' for ontology '{}' and version '{}'",
                    file_name, ontology_id, version
                );

                Ok(response
                    .text()
                    .map_err(|err| OntologyRegistryError::ProvidingOntology {
                        reason: err.to_string(),
                    })?)
            }
            Err(err) => Err(OntologyRegistryError::ProvidingOntology {
                reason: err.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::OntologyProvider;
    use mockito::Server;

    #[test]
    fn test_provide_ontology_latest_success() {
        let mut server = Server::new();

        let ontology_ontology_id = "go";
        let file_name = "go.owl";
        let expected_body = "OWL Content";

        let mock = server
            .mock("GET", "/go/releases/2023-01-01/go.owl")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(expected_body)
            .create();

        let provider = OboLibraryProvider::new(server.url());

        let result = provider.provide_ontology(ontology_ontology_id, file_name, "2023-01-01");

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body);
    }

    #[test]
    fn test_provide_ontology_specific_version_success() {
        let mut server = Server::new();

        let ontology_ontology_id = "go";
        let file_name = "go.owl";
        let expected_body = "Versioned Content";

        let expected_path = format!(
            "/{}/releases/{}/{}",
            ontology_ontology_id, "2023-01-01", file_name
        );

        let mock = server
            .mock("GET", expected_path.as_str())
            .with_status(200)
            .with_body(expected_body)
            .create();

        let provider = OboLibraryProvider::new(server.url());

        let result = provider.provide_ontology(ontology_ontology_id, file_name, "2023-01-01");

        mock.assert();
        assert_eq!(result.unwrap(), expected_body);
    }

    #[test]
    fn test_provide_ontology_server_error() {
        let mut server = Server::new();

        let mock = server
            .mock("GET", "/go/releases/2023-01-01/go.owl")
            .with_status(404)
            .create();

        let provider = OboLibraryProvider::new(server.url());

        let result = provider.provide_ontology("go", "go.owl", "2023-01-01");

        mock.assert();
        assert!(result.is_err());

        match result {
            Err(OntologyRegistryError::ProvidingOntology { reason }) => {
                assert!(!reason.is_empty());
            }
            _ => panic!("Wrong error type returned"),
        }
    }
}
