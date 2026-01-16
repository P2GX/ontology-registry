#![allow(dead_code)]
use crate::blocking::obolib_metadata_json::Registry;
use crate::error::OntologyRegistryError;

pub struct ObolibMetadataProvider {
    metadata: Registry,
}

impl ObolibMetadataProvider {
    pub fn _new(data_url: &str) -> Result<Self, OntologyRegistryError> {
        let metadata = reqwest::blocking::get(data_url)
            .and_then(|res| res.error_for_status())
            .and_then(|res| res.json::<Registry>())
            .map_err(|err| OntologyRegistryError::ProvidingOntology {
                reason: err.to_string(),
            })?;

        Ok(Self { metadata })
    }

    pub fn _with_default_url() -> Result<Self, OntologyRegistryError> {
        Self::_new("https://purl.obolibrary.org/meta/ontologies.jsonld")
    }
}
/*
impl OntologyMetadataProvider for ObolibMetadataProvider {
    fn provide_metadata(&self, ontology_id: &str) -> Result<OntologyMetadata, OntologyRegistryError> {
        for on in &self.metadata.ontologies {
            if on.id.to_lowercase() == ontology_id.to_lowercase() {
                return Ok(OntologyMetadata {
                    ontology_id: on.id.to_string(),
                    version: "".to_string(),
                    json_file_location: on
                        .products
                        .iter()
                        .find(|p| {
                            p.ontology_purl
                                .as_ref()
                                .map_or(false, |url| url.ends_with(".json"))
                        })
                        .and_then(|p| p.ontology_purl.clone()),
                    owl_file_location: on
                        .products
                        .iter()
                        .find(|p| {
                            p.ontology_purl
                                .as_ref()
                                .map_or(false, |url| url.ends_with(".owl"))
                        })
                        .and_then(|p| p.ontology_purl.clone()),
                    obo_file_location: on
                        .products
                        .iter()
                        .find(|p| {
                            p.ontology_purl
                                .as_ref()
                                .map_or(false, |url| url.ends_with(".obo"))
                        })
                        .and_then(|p| p.ontology_purl.clone()),
                    title: on.title.clone(),
                });
            }
        }

        Err(OntologyRegistryError::ProvidingMetadata {
            reason: format!("Could not retrieve metadata for given ontology_id: {}", ontology_id),
        })
    }
}
*/
