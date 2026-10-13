use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AfJobAd {
    pub id: String,
    pub headline: Option<String>,

    pub description: Option<AfDescription>,
    pub employer: Option<AfEmployer>,
    pub application_details: Option<AfApplicationDetails>,
    pub workplace_address: Option<AfWorkplaceAddress>,

    pub occupation: Option<AfTaxonomyItem>,
    pub occupation_group: Option<AfTaxonomyItem>,
    pub occupation_field: Option<AfTaxonomyItem>,

    pub employment_type: Option<AfTaxonomyItem>,
    pub working_hours_type: Option<AfTaxonomyItem>,
    pub duration: Option<AfTaxonomyItem>,

    pub number_of_vacancies: Option<i32>,
    pub experience_required: Option<bool>,

    pub application_deadline: Option<String>,
    pub publication_date: Option<String>,

    pub webpage_url: Option<String>,
    pub logo_url: Option<String>,

    pub timestamp: Option<i64>,

    #[serde(default)]
    pub removed: bool,

    pub removed_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AfDescription {
    pub text: Option<String>,
    pub text_formatted: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AfEmployer {
    pub name: Option<String>,
    pub workplace: Option<String>,
    pub organization_number: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AfApplicationDetails {
    pub url: Option<String>,
    pub email: Option<String>,
    pub reference: Option<String>,
    pub information: Option<String>,
    pub other: Option<String>,
    pub via_af: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AfWorkplaceAddress {
    pub city: Option<String>,
    pub municipality: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postcode: Option<String>,
    pub street_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AfTaxonomyItem {
    pub concept_id: Option<String>,
    pub label: Option<String>,
    pub legacy_ams_taxonomy_id: Option<String>,
}