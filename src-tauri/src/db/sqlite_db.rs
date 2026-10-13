use std::fs;
use std::path::{Path, PathBuf};

pub struct DbState {
    pub path: PathBuf,
}

pub fn init_database(
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {

    // Create the app-data directory if it doesn't already exist.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Opens jobsy.db if it exists.
    // Creates it if it doesn't.
    let connection = sqlite::open(path)?;

    connection.execute(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,

            source TEXT NOT NULL,
            external_id TEXT NOT NULL,

            title TEXT NOT NULL,
            description TEXT,

            organization_number TEXT,
            employer_name TEXT,
            employer_website TEXT,

            city TEXT,
            municipality TEXT,
            region TEXT,

            occupation TEXT,
            occupation_group TEXT,
            occupation_field TEXT,

            employment_type TEXT,
            working_hours TEXT,

            number_of_vacancies INTEGER,
            experience_required INTEGER,

            listing_url TEXT,
            application_url TEXT,

            published_at TEXT,
            application_deadline TEXT,

            active INTEGER NOT NULL DEFAULT 1,

            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

            UNIQUE(source, external_id)
        );
        "#,
    )?;

    println!("SQLite ready: {}", path.display());

    Ok(())
}