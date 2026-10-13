use std::path::Path;

use crate::jobs::model::JobAd;

pub fn save_jobs(
    db_path: &Path,
    jobs: &[JobAd],
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = sqlite::open(db_path)?;

    connection.execute("BEGIN TRANSACTION;")?;

    let mut statement = connection.prepare(
        r#"
        INSERT INTO jobs (
            source,
            external_id,
            title,
            description,
            organization_number,
            employer_name,
            employer_website,
            city,
            municipality,
            region,
            occupation,
            occupation_group,
            occupation_field,
            employment_type,
            working_hours,
            number_of_vacancies,
            experience_required,
            listing_url,
            application_url,
            published_at,
            application_deadline,
            active
        )
        VALUES (
            :source,
            :external_id,
            :title,
            :description,
            :organization_number,
            :employer_name,
            :employer_website,
            :city,
            :municipality,
            :region,
            :occupation,
            :occupation_group,
            :occupation_field,
            :employment_type,
            :working_hours,
            :number_of_vacancies,
            :experience_required,
            :listing_url,
            :application_url,
            :published_at,
            :application_deadline,
            :active
        )
        ON CONFLICT(source, external_id)
        DO UPDATE SET
            title = excluded.title,
            description = excluded.description,
            organization_number = excluded.organization_number,
            employer_name = excluded.employer_name,
            employer_website = excluded.employer_website,
            city = excluded.city,
            municipality = excluded.municipality,
            region = excluded.region,
            occupation = excluded.occupation,
            occupation_group = excluded.occupation_group,
            occupation_field = excluded.occupation_field,
            employment_type = excluded.employment_type,
            working_hours = excluded.working_hours,
            number_of_vacancies = excluded.number_of_vacancies,
            experience_required = excluded.experience_required,
            listing_url = excluded.listing_url,
            application_url = excluded.application_url,
            published_at = excluded.published_at,
            application_deadline = excluded.application_deadline,
            active = excluded.active,
            updated_at = CURRENT_TIMESTAMP;
        "#,
    )?;

    for job in jobs {
        let number_of_vacancies =
            job.number_of_vacancies.map(i64::from);

        let experience_required =
            job.experience_required
                .map(|value| if value { 1_i64 } else { 0_i64 });

        let active =
            if job.active { 1_i64 } else { 0_i64 };

        statement.bind((":source", job.source.as_str()))?;
        statement.bind((":external_id", job.external_id.as_str()))?;
        statement.bind((":title", job.title.as_str()))?;
        statement.bind((":description", job.description.as_deref()))?;

        statement.bind((
            ":organization_number",
            job.organization_number.as_deref(),
        ))?;

        statement.bind((
            ":employer_name",
            job.employer_name.as_deref(),
        ))?;

        statement.bind((
            ":employer_website",
            job.employer_website.as_deref(),
        ))?;

        statement.bind((":city", job.city.as_deref()))?;
        statement.bind((":municipality", job.municipality.as_deref()))?;
        statement.bind((":region", job.region.as_deref()))?;

        statement.bind((":occupation", job.occupation.as_deref()))?;

        statement.bind((
            ":occupation_group",
            job.occupation_group.as_deref(),
        ))?;

        statement.bind((
            ":occupation_field",
            job.occupation_field.as_deref(),
        ))?;

        statement.bind((
            ":employment_type",
            job.employment_type.as_deref(),
        ))?;

        statement.bind((
            ":working_hours",
            job.working_hours.as_deref(),
        ))?;

        statement.bind((
            ":number_of_vacancies",
            number_of_vacancies,
        ))?;

        statement.bind((
            ":experience_required",
            experience_required,
        ))?;

        statement.bind((
            ":listing_url",
            job.listing_url.as_deref(),
        ))?;

        statement.bind((
            ":application_url",
            job.application_url.as_deref(),
        ))?;

        statement.bind((
            ":published_at",
            job.published_at.as_deref(),
        ))?;

        statement.bind((
            ":application_deadline",
            job.application_deadline.as_deref(),
        ))?;

        statement.bind((":active", active))?;

        statement.next()?;
        statement.reset()?;
    }

    drop(statement);

    connection.execute("COMMIT;")?;

    Ok(())
}

pub fn get_jobs(
    db_path: &Path,
) -> Result<Vec<JobAd>, Box<dyn std::error::Error>> {
    let connection = sqlite::open(db_path)?;

    let mut statement = connection.prepare(
        r#"
        SELECT
            source,
            external_id,
            title,
            description,
            organization_number,
            employer_name,
            employer_website,
            city,
            municipality,
            region,
            occupation,
            occupation_group,
            occupation_field,
            employment_type,
            working_hours,
            number_of_vacancies,
            experience_required,
            listing_url,
            application_url,
            published_at,
            application_deadline,
            active
        FROM jobs
        WHERE active = 1
        ORDER BY published_at DESC;
        "#,
    )?;

    let mut jobs = Vec::new();

    while let sqlite::State::Row = statement.next()? {
        let number_of_vacancies: Option<i64> =
            statement.read("number_of_vacancies")?;

        let experience_required: Option<i64> =
            statement.read("experience_required")?;

        let active: i64 =
            statement.read("active")?;

        jobs.push(JobAd {
            source: statement.read("source")?,
            external_id: statement.read("external_id")?,
            title: statement.read("title")?,
            description: statement.read("description")?,

            organization_number: statement.read("organization_number")?,
            employer_name: statement.read("employer_name")?,
            employer_website: statement.read("employer_website")?,

            city: statement.read("city")?,
            municipality: statement.read("municipality")?,
            region: statement.read("region")?,

            occupation: statement.read("occupation")?,
            occupation_group: statement.read("occupation_group")?,
            occupation_field: statement.read("occupation_field")?,

            employment_type: statement.read("employment_type")?,
            working_hours: statement.read("working_hours")?,

            number_of_vacancies:
                number_of_vacancies.map(|value| value as i32),

            experience_required:
                experience_required.map(|value| value != 0),

            listing_url: statement.read("listing_url")?,
            application_url: statement.read("application_url")?,

            published_at: statement.read("published_at")?,
            application_deadline: statement.read("application_deadline")?,

            active: active != 0,
        });
    }

    Ok(jobs)
}

pub fn count_jobs(
    db_path: &Path,
) -> Result<i64, Box<dyn std::error::Error>> {
    let connection = sqlite::open(db_path)?;

    let mut statement = connection.prepare(
        "SELECT COUNT(*) FROM jobs;",
    )?;

    if let sqlite::State::Row = statement.next()? {
        let count: i64 = statement.read(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}