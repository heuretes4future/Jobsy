use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JobAd {
    pub source: String,
    pub external_id: String,

    pub title: String,
    pub description: Option<String>,

    pub organization_number: Option<String>,
    pub employer_name: Option<String>,
    pub employer_website: Option<String>,

    pub city: Option<String>,
    pub municipality: Option<String>,
    pub region: Option<String>,

    pub occupation: Option<String>,
    pub occupation_group: Option<String>,
    pub occupation_field: Option<String>,

    pub employment_type: Option<String>,
    pub working_hours: Option<String>,

    pub number_of_vacancies: Option<i32>,
    pub experience_required: Option<bool>,

    pub listing_url: Option<String>,
    pub application_url: Option<String>,

    pub published_at: Option<String>,
    pub application_deadline: Option<String>,

    pub active: bool,
}