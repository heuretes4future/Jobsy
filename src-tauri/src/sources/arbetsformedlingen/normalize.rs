use crate::jobs::model::JobAd;

use super::model::AfJobAd;

pub fn normalize_job(job: AfJobAd) -> JobAd {
    let employer = job.employer.as_ref();
    let address = job.workplace_address.as_ref();

    let organization_number = employer
        .and_then(|e| e.organization_number.clone());

    let employer_name = employer
        .and_then(|e| e.name.clone());

    let employer_website = employer
        .and_then(|e| e.url.clone());

    let city = address
        .and_then(|a| a.city.clone());

    let municipality = address
        .and_then(|a| a.municipality.clone());

    let region = address
        .and_then(|a| a.region.clone());

    let application_url = job
        .application_details
        .as_ref()
        .and_then(|details| details.url.clone());

    JobAd {
        source: "arbetsformedlingen".to_string(),

        external_id: job.id,

        title: job
            .headline
            .unwrap_or_else(|| "Unknown title".to_string()),

        description: job
            .description
            .and_then(|d| d.text),

        organization_number,
        employer_name,
        employer_website,

        city,
        municipality,
        region,

        occupation: job
            .occupation
            .and_then(|x| x.label),

        occupation_group: job
            .occupation_group
            .and_then(|x| x.label),

        occupation_field: job
            .occupation_field
            .and_then(|x| x.label),

        employment_type: job
            .employment_type
            .and_then(|x| x.label),

        working_hours: job
            .working_hours_type
            .and_then(|x| x.label),

        number_of_vacancies: job.number_of_vacancies,
        experience_required: job.experience_required,

        listing_url: job.webpage_url,
        application_url,

        published_at: job.publication_date,
        application_deadline: job.application_deadline,

        active: !job.removed,
    }
}