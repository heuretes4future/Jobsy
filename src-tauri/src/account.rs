use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use chrono::Utc;
use crate::jobs::model::JobAd;
use crate::resume::{self, ResumeData};

#[derive(Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub active_profile_id: Option<String>,
    pub theme: String,
}
impl Default for Preferences {
    fn default() -> Self { Self { active_profile_id: None, theme: "dark".into() } }
}

fn data_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join(filename))
}

fn applied_path(app: &AppHandle) -> Result<PathBuf, String> {
    let path = data_path(app, "applied.json")?;
    if !path.exists() {
        let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().ok_or("Cannot resolve project folder")?.join("Store/applied.json");
        if cfg!(debug_assertions) && legacy.is_file() {
            fs::copy(legacy, &path).map_err(|err| err.to_string())?;
        } else {
            fs::write(&path, "[]").map_err(|err| err.to_string())?;
        }
    }
    Ok(path)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppliedJob {
    pub id: String,
    pub company: String,
    pub role: String,
    pub date: String,
    pub summary: String,
}

fn read_preferences(app: &AppHandle) -> Result<Preferences, String> {
    let path = data_path(app, "preferences.json")?;
    if !path.exists() { return Ok(Preferences::default()); }
    serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?)
        .map_err(|err| format!("Invalid preferences: {err}"))
}
fn write_preferences(app: &AppHandle, preferences: &Preferences) -> Result<(), String> {
    fs::write(data_path(app, "preferences.json")?, serde_json::to_string_pretty(preferences).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_profiles(app: AppHandle) -> Result<Vec<(String, ResumeData)>, String> {
    let mut profiles: Vec<_> = resume::read_profiles(&app)?.into_iter().collect();
    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    // Legacy 'person1' IDs remain stable in storage, but aren't shown in the UI.
    for (index, (id, profile)) in profiles.iter_mut().enumerate() {
        if profile.display_name.trim().is_empty() {
            profile.display_name = if id.as_str() == "user" || (id.starts_with("person") && index == 0) {
                "user".into()
            } else {
                format!("user {}", index + 1)
            };
        }
    }
    Ok(profiles)
}

#[tauri::command]
pub fn save_profile_keywords(app: AppHandle, profile_id: String, keywords: Vec<String>) -> Result<(), String> {
    if keywords.len() > 100 { return Err("Maximum 100 keywords per profile".into()); }
    let mut cleaned: Vec<String> = Vec::new();
    for word in keywords {
        let trimmed = word.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 80 {
            return Err("Keywords must be between 1 and 80 characters".into());
        }
        if !cleaned.iter().any(|previous| previous.eq_ignore_ascii_case(trimmed)) {
            cleaned.push(trimmed.to_string());
        }
    }
    let mut profiles = resume::read_profiles(&app)?;
    let profile = profiles.get_mut(&profile_id).ok_or("Account not found")?;
    profile.search_keywords = cleaned;
    profile.keyword_mode = true;
    resume::write_profiles(&app, &profiles)
}

#[tauri::command]
pub fn rename_profile(app: AppHandle, profile_id: String, display_name: String) -> Result<(), String> {
    let name = display_name.trim();
    if name.is_empty() || name.chars().count() > 50 || name.chars().any(char::is_control) {
        return Err("Account name must contain 1–50 printable characters".into());
    }
    let mut profiles = resume::read_profiles(&app)?;
    if profiles.iter().any(|(id, profile)| id != &profile_id && profile.display_name.eq_ignore_ascii_case(name)) {
        return Err("Another account already uses this name".into());
    }
    let profile = profiles.get_mut(&profile_id).ok_or("Account not found")?;
    profile.display_name = name.to_string();
    resume::write_profiles(&app, &profiles)
}

#[tauri::command]
pub fn remove_profile(app: AppHandle, profile_id: String) -> Result<(), String> {
    let mut profiles = resume::read_profiles(&app)?;
    if profiles.remove(&profile_id).is_none() { return Err("Account not found".into()); }
    resume::write_profiles(&app, &profiles)?;
    let mut prefs = read_preferences(&app)?;
    if prefs.active_profile_id.as_deref() == Some(profile_id.as_str()) {
        let mut ids: Vec<_> = profiles.keys().cloned().collect();
        ids.sort();
        prefs.active_profile_id = ids.into_iter().next();
        write_preferences(&app, &prefs)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_preferences(app: AppHandle) -> Result<Preferences, String> {
    let mut prefs = read_preferences(&app)?;
    let mut ids: Vec<String> = resume::read_profiles(&app)?.keys().cloned().collect();
    ids.sort();
    if !prefs.active_profile_id.as_ref().is_some_and(|id| ids.contains(id)) {
        prefs.active_profile_id = ids.into_iter().next();
        write_preferences(&app, &prefs)?;
    }
    Ok(prefs)
}

#[tauri::command]
pub fn save_preferences(app: AppHandle, preferences: Preferences) -> Result<(), String> {
    if preferences.theme != "dark" && preferences.theme != "light" {
        return Err("Unsupported theme".into());
    }
    if let Some(ref id) = preferences.active_profile_id {
        if !resume::read_profiles(&app)?.contains_key(id) { return Err("Account not found".into()); }
    }
    write_preferences(&app, &preferences)
}

#[tauri::command]
pub fn get_applied_jobs(app: AppHandle) -> Result<Vec<AppliedJob>, String> {
    serde_json::from_str(&fs::read_to_string(applied_path(&app)?).map_err(|err| err.to_string())?)
        .map_err(|err| format!("Invalid applied.json: {err}"))
}

#[tauri::command]
pub fn mark_apply_clicked(app: AppHandle, job: JobAd) -> Result<AppliedJob, String> {
    let url = job.application_url.as_deref().filter(|s| !s.is_empty())
        .or(job.listing_url.as_deref()).ok_or("No application link for this job")?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Invalid application URL".into());
    }
    let id = format!("{}:{}", job.source, job.external_id);
    let mut applied = get_applied_jobs(app.clone())?;
    if let Some(existing) = applied.iter().find(|entry| entry.id == id) {
        return Ok(existing.clone());
    }
    let entry = AppliedJob {
        id,
        company: job.employer_name.unwrap_or_else(|| "Unknown employer".to_owned()),
        role: job.title,
        date: Utc::now().to_rfc3339(),
        summary: job.description.unwrap_or_default(),
    };
    applied.push(entry.clone());
    fs::write(applied_path(&app)?, serde_json::to_string_pretty(&applied).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())?;
    Ok(entry)
}
