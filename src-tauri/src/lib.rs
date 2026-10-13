use sqlite::State;
use std::collections::{HashMap, HashSet};
use std::fs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{path::BaseDirectory, AppHandle, Manager};
mod commands;
mod sources;
mod jobs;
mod db;
mod resume;
mod search;
mod account;

#[derive(Serialize, Deserialize, Clone)]
struct ResumeData {
    country: String,
    technical_skills: Vec<String>,
}


#[derive(Serialize)]
struct ResumeExtraction {
    text: String,
    profile_id: String,
}

const SKILLS_DB: &str = "skills-table/data.db";

fn resource_path(app: &AppHandle, relative: &str) -> Result<PathBuf, String> {
    app.path()
        .resolve(relative, BaseDirectory::Resource)
        .map_err(|e| e.to_string())
}

// PDF parsing and SQLite skill extraction can be expensive. Do not perform
// them on the Tauri main thread or the async runtime's worker threads.
#[tauri::command]
async fn pdf_to_string(app: AppHandle, url: String) -> Result<ResumeExtraction, String> {
    let db_path = resource_path(&app, SKILLS_DB)?;
    let resume_path = resume::resume_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || -> Result<ResumeExtraction, String> {
        let doc = pdf_oxide::PdfDocument::open(&url)
            .map_err(|err| format!("Failed to open file: {err}"))?;
        let text = doc.extract_text(0)
            .map_err(|err| format!("Extraction failed: {err}"))?;
        let profile_id = select_skills(&text, &db_path, &resume_path)?;
        Ok(ResumeExtraction { text, profile_id })
    })
    .await
    .map_err(|err| format!("Resume processing task failed: {err}"))?
}

fn normalize_word(word: &str) -> String {
    word
        .trim_matches(|c: char| {
            matches!(
                c,
                ',' | ':' | ';' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
            )
        })
        .to_lowercase()
}



fn select_skills(pdf_string: &str, db_path: &std::path::Path, file_path: &std::path::Path) -> Result<String, String> {
   
    let country = "Sweden".to_string();

    let words: Vec<String> = pdf_string
    .split_whitespace()
    .map(normalize_word)
    .filter(|word| !word.is_empty())
    .collect();

    let connection = sqlite::Connection::open_with_flags(
    db_path,
    sqlite::OpenFlags::new().with_read_only(),
)
.map_err(|e| e.to_string())?;
        

    let mut technical_skills = Vec::new();
    let mut seen = HashSet::new();

    let mut statement = connection
        .prepare(
                "
        SELECT canonical_skill
        FROM technical_skills
        WHERE normalized_keyword = ?
        LIMIT 1
        "
        )
        .map_err(|e| e.to_string())?;

    let mut i = 0;
    
    while i < words.len() {

        let remaining = words.len() - i;
        let max_length = remaining.min(3);

        let mut matched = false;

        // Try 3 words -> 2 words -> 1 word.
        for length in (1..=max_length).rev() {

            let phrase =
                words[i..i + length].join(" ");

            statement
                .bind((1, phrase.as_str()))
                .map_err(|e| e.to_string())?;

            match statement.next() {

                Ok(State::Row) => {

                    let skill: String =
                        statement
                            .read(0)
                            .map_err(|e| e.to_string())?;

                    if seen.insert(skill.clone()) {
                        technical_skills.push(skill);
                    }

                    i += length;
                    matched = true;

                    statement
                        .reset()
                        .map_err(|e| e.to_string())?;

                    break;
                }

                Ok(State::Done) => {

                    statement
                        .reset()
                        .map_err(|e| e.to_string())?;
                }

                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }

        if !matched {
            i += 1;
        }
    }

    let resume_data = ResumeData {
        country,
        technical_skills,
    };


    // Save new resume profiles without duplicating existing skill sets.
    let mut people: HashMap<String, resume::ResumeData> =
        if file_path.exists() {
            serde_json::from_str(&fs::read_to_string(file_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        } else {
            HashMap::new()
        };

    let mut new_skills = resume_data.technical_skills.clone();
    new_skills.sort();
    if let Some((profile_id, _)) = people.iter().find(|(_, person)| {
        let mut existing = person.technical_skills.clone();
        existing.sort();
        person.country == resume_data.country && existing == new_skills
    }) {
        return Ok(profile_id.clone());
    }

    // IDs are internal; use a friendly default instead of 'person1'.
    let mut next_number = people.len() + 1;
    let profile_id = loop {
        let candidate = if next_number == 1 { "user".to_owned() }
                        else { format!("user{next_number}") };
        if !people.contains_key(&candidate) { break candidate; }
        next_number += 1;
    };
    people.insert(profile_id.clone(), resume::ResumeData {
        display_name: profile_id.clone(),
        country: resume_data.country,
        technical_skills: resume_data.technical_skills,
        search_keywords: Vec::new(),
        keyword_mode: false,
    });
    fs::write(file_path, serde_json::to_string_pretty(&people).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    Ok(profile_id)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()?
                .join("jobsy.db");

            db::sqlite_db::init_database(&db_path)?;

            app.manage(db::sqlite_db::DbState {
                path: db_path,
            });
            app.manage(commands::jobs::SearchControl::default());

            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(
                    tauri_plugin_log::log::LevelFilter::Info
                )
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                pdf_to_string,
                commands::jobs::search_resume_jobs,
                commands::jobs::cancel_job_search,
                commands::jobs::get_jobs,
                account::list_profiles,
                account::save_profile_keywords,
                account::rename_profile,
                account::remove_profile,
                account::get_preferences,
                account::save_preferences,
                account::get_applied_jobs,
                account::mark_apply_clicked
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}