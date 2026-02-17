#[derive(Debug, Default)]
pub struct OntologyMetadata {
    pub ontology_id: String,
    pub version: String,
    pub json_file_location: Option<String>,
    pub owl_file_location: Option<String>,
    pub obo_file_location: Option<String>,
    pub title: Option<String>,
}
