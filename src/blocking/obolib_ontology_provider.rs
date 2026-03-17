use crate::Version;
use crate::error::OntologyRegistryError;
use crate::traits::OntologyProviding;
use std::io::Read;

#[derive(Debug)]
pub struct OboLibraryProvider {
    base_url: String,
    client: reqwest::blocking::Client,
}
impl Default for OboLibraryProvider {
    fn default() -> Self {
        OboLibraryProvider {
            base_url: "https://purl.obolibrary.org/obo".to_string(),
            client: reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; ontology-registry/1.0)")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

impl OboLibraryProvider {
    pub fn new(base_url: String) -> Self {
        OboLibraryProvider {
            base_url,
            client: reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; ontology-registry/1.0)")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

impl OntologyProviding for OboLibraryProvider {
    fn provide_ontology(
        &self,
        ontology_id: &str,
        file_name: &str,
        version: &Version,
    ) -> Result<impl Read, OntologyRegistryError> {
        let urls = match version {
            Version::Latest => {
                vec![format!("{}/{}/{}", self.base_url, ontology_id, file_name)]
            }
            Version::Declared(v) => {
                vec![
                    format!(
                        "{}/{}/releases/{}/{}",
                        self.base_url, ontology_id, v, file_name
                    ),
                    format!("{}/{}/{}/{}", self.base_url, ontology_id, v, file_name),
                ]
            }
        };

        let mut last_status = None;

        for url in &urls {
            let resp = self.client.get(url).send();

            match resp {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                        last_status = Some(response.status());
                        continue;
                    } else {
                        return Err(OntologyRegistryError::ProvidingOntology {
                            reason: format!("HTTP Error {} for {}", response.status(), url),
                        });
                    }
                }
                Err(err) => {
                    return Err(OntologyRegistryError::ProvidingOntology {
                        reason: format!("Network Error: {}", err),
                    });
                }
            }
        }

        Err(OntologyRegistryError::ProvidingOntology {
            reason: format!(
                "Ontology not found after trying {} patterns. Last status: {:?}",
                urls.len(),
                last_status.unwrap_or(reqwest::StatusCode::NOT_FOUND)
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::OntologyProviding;
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

        let version = Version::from("2023-01-01");

        let mut result = provider
            .provide_ontology(ontology_ontology_id, file_name, &version)
            .unwrap();

        mock.assert();
        let mut buffer = String::new();
        let _ = result.read_to_string(&mut buffer).unwrap();

        assert_eq!(buffer, expected_body);
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

        let version = Version::from("2023-01-01");

        let mut result = provider
            .provide_ontology(ontology_ontology_id, file_name, &version)
            .unwrap();

        mock.assert();
        let mut buffer = String::new();
        let _ = result.read_to_string(&mut buffer).unwrap();

        assert_eq!(buffer, expected_body);
    }

    #[test]
    fn test_provide_ontology_server_error() {
        let mut server = Server::new();

        let mock = server
            .mock("GET", "/go/releases/2023-01-01/go.owl")
            .with_status(404)
            .create();

        let provider = OboLibraryProvider::new(server.url());

        let version = Version::from("2023-01-01");

        let result = provider.provide_ontology("go", "go.owl", &version);

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
