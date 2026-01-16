use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    #[serde(rename = "@context")]
    pub context: String,
    pub ontologies: Vec<Ontology>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ontology {
    pub id: String,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(rename = "ontology_purl", default)]
    pub ontology_purl: Option<String>,

    #[serde(rename = "preferredPrefix", default)]
    pub preferred_ontology_id: Option<String>,

    #[serde(rename = "activity_status", default)]
    pub activity_status: Option<String>,

    #[serde(default)]
    pub domain: Option<String>,

    #[serde(default)]
    pub homepage: Option<String>,

    #[serde(default)]
    pub repository: Option<String>,

    #[serde(default)]
    pub tracker: Option<String>,

    #[serde(rename = "mailing_list", default)]
    pub mailing_list: Option<String>,

    #[serde(default)]
    pub layout: Option<String>,

    #[serde(default)]
    pub license: Option<License>,

    #[serde(default)]
    pub contact: Option<Contact>,

    #[serde(default)]
    pub products: Vec<Product>,

    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    #[serde(default)]
    pub publications: Vec<Publication>,

    #[serde(default)]
    pub usages: Vec<Usage>,

    #[serde(default)]
    pub browsers: Vec<Browser>,

    #[serde(default)]
    pub build: Option<BuildInfo>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub taxon: Option<Taxon>,

    #[serde(rename = "in_foundry", default)]
    pub in_foundry: Option<bool>,

    #[serde(rename = "in_foundry_order", default)]
    pub in_foundry_order: Option<u32>,

    #[serde(rename = "depicted_by", default)]
    pub depicted_by: Option<String>,

    #[serde(default)]
    pub twitter: Option<String>,

    #[serde(default)]
    pub slack: Option<String>,

    #[serde(rename = "funded_by", default)]
    pub funded_by: Vec<Funding>,

    #[serde(rename = "integration_server", default)]
    pub integration_server: Option<String>,

    #[serde(default)]
    pub canonical: Option<String>,

    #[serde(rename = "wasDerivedFrom", default)]
    pub was_derived_from: Option<String>,

    #[serde(rename = "replaced_by", default)]
    pub replaced_by: Option<String>,

    #[serde(rename = "is_obsolete", default)]
    pub is_obsolete: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub label: Option<String>,
    pub email: Option<String>,
    pub github: Option<String>,
    pub orcid: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct License {
    pub label: Option<String>,
    pub logo: Option<String>,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,

    #[serde(rename = "ontology_purl", default)]
    pub ontology_purl: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub format: Option<String>,

    #[serde(rename = "is_canonical", default)]
    pub is_canonical: Option<bool>,

    #[serde(rename = "derived_from", default)]
    pub derived_from: Option<String>,

    #[serde(default)]
    pub uses: Vec<String>,

    #[serde(default)]
    pub page: Option<String>,

    #[serde(default)]
    pub status: Option<String>,

    #[serde(rename = "type", default)]
    pub type_field: Option<String>,

    #[serde(default)]
    pub contact: Option<Contact>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub id: Option<String>,

    pub subset: Option<String>,

    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub connects: Vec<DependencyConnection>,
    #[serde(default)]
    pub publications: Vec<Publication>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyConnection {
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Publication {
    pub id: String,
    pub title: Option<String>,
    pub preferred: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub user: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    #[serde(rename = "seeAlso")]
    pub see_also: Option<String>,
    #[serde(default)]
    pub examples: Vec<UsageExample>,

    #[serde(default)]
    pub publications: Vec<Publication>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageExample {
    pub description: Option<String>,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Browser {
    pub label: String,
    pub title: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildInfo {
    pub checkout: Option<String>,
    pub path: Option<String>,
    pub system: Option<String>,
    pub method: Option<String>,
    pub source_url: Option<String>,

    pub infallible: Option<u8>,

    pub notes: Option<String>,
    pub oort_args: Option<String>,
    pub email_cc: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Taxon {
    pub id: String,
    pub label: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Funding {
    pub id: Option<String>,
    pub title: Option<String>,
}
