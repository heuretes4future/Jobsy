use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResumeData {
    #[serde(default)]
    pub display_name: String,
    pub country: String,
    pub technical_skills: Vec<String>,
    #[serde(default)]
    pub search_keywords: Vec<String>,
    #[serde(default)]
    pub keyword_mode: bool,
}

// Profiles are PRIVATE and writable only in the user's Tauri app-data directory.
// On first launch during development, migrate a legacy résumé file if present.
// Never embed résumé information into installers or public source releases.
pub fn resume_path(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app.path().app_data_dir().map_err(|err| err.to_string())?
        .join("resume_data.json");
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let old = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().ok_or("Cannot resolve project directory")?.join("resume_data.json");
        if cfg!(debug_assertions) && old.exists() {
            fs::copy(old, &path).map_err(|err| err.to_string())?;
        } else {
            fs::write(&path, "{}").map_err(|err| err.to_string())?;
        }
    }
    Ok(path)
}

pub fn read_profiles(app: &AppHandle) -> Result<HashMap<String, ResumeData>, String> {
    let data = fs::read_to_string(resume_path(app)?)
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&data).map_err(|err| err.to_string())
}

pub fn get_profile(app: &AppHandle, profile_id: &str) -> Result<ResumeData, String> {
    read_profiles(app)?
        .remove(profile_id)
        .ok_or_else(|| format!("Resume profile '{profile_id}' was not found"))
}

// Explicit keywords can include additions that the PDF parser missed.
// Existing imported profiles without explicit keywords fall back to resume skills.
pub fn search_keywords(profile: &ResumeData, max: usize) -> Vec<String> {
    let terms = if profile.keyword_mode || !profile.search_keywords.is_empty() {
        &profile.search_keywords
    } else {
        &profile.technical_skills
    };
    // When no manual selection exists, prioritize concrete technologies over
    // generic software or administrative terms extracted from the PDF.
    let mut ordered = terms.to_vec();
    if !profile.keyword_mode && profile.search_keywords.is_empty() {
        const PREFERRED: &[&str] = &[
            "Python", "Rust", "React", "Docker", "Machine learning", "JavaScript",
            "TypeScript", "Java", "SQL", "PostgreSQL", "Linux", "C++", "Spring Boot",
            "Kubernetes", "Mjukvaruutvecklare", "PyTorch", "Git", "Flask",
        ];
        ordered.sort_by_key(|skill| PREFERRED.iter().position(|preferred| preferred.eq_ignore_ascii_case(skill)).unwrap_or(usize::MAX));
    }
    let mut selected = Vec::new();
    for term in &ordered {
        let word = term.trim();
        if word.is_empty() || selected.iter().any(|previous: &String| previous.eq_ignore_ascii_case(word)) {
            continue;
        }
        selected.push(word.to_owned());
        if selected.len() >= max { break; }
    }
    selected
}

pub fn write_profiles(app: &AppHandle, profiles: &HashMap<String, ResumeData>) -> Result<(), String> {
    let content = serde_json::to_string_pretty(profiles).map_err(|err| err.to_string())?;
    fs::write(resume_path(app)?, content).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_custom_keywords_without_duplicates() {
        let profile = ResumeData {
            display_name: "user".into(),
            country: "Sweden".into(),
            technical_skills: vec!["Rust".into(), "Python".into()],
            search_keywords: vec!["Python".into(), "Not a skill".into(), "python".into(), "Rust".into()],
            keyword_mode: true,
        };
        assert_eq!(search_keywords(&profile, 4), vec!["Python", "Not a skill", "Rust"]);
    }
}
