use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Deserialize)]
pub struct SearchSource {
    pub id: String,
    pub kind: String,
    pub base_url: String,
    pub enabled: bool,
    pub max_keywords: usize,
    pub results_per_keyword: u32,
    #[serde(default = "default_pages")]
    pub max_pages_per_keyword: u32,
    #[serde(default = "default_refresh")]
    pub refresh_minutes: i64,
}

fn default_pages() -> u32 { 1 }
fn default_refresh() -> i64 { 240 }

#[derive(Deserialize)]
struct SearchSources {
    sources: Vec<SearchSource>,
}

// URLs, provider enablement and request budgets live outside the Rust source.
pub fn enabled_jobsearch(app: &AppHandle) -> Result<SearchSource, String> {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Cannot resolve project directory")?
        .join("Store/job_sources.json");

    let contents = if cfg!(debug_assertions) && dev_path.is_file() {
        fs::read_to_string(dev_path).map_err(|err| err.to_string())?
    } else {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|err| err.to_string())?
            .join("job_sources.json");
        if path.exists() {
            fs::read_to_string(path).map_err(|err| err.to_string())?
        } else {
            include_str!("../../Store/job_sources.json").to_owned()
        }
    };

    let config: SearchSources = serde_json::from_str(&contents)
        .map_err(|err| format!("Invalid job source configuration: {err}"))?;

    let source = config.sources.into_iter()
        .find(|source| source.enabled && source.kind == "jobsearch")
        .ok_or("No enabled JobSearch provider in job_sources.json")?;

    if source.max_keywords == 0 || source.max_keywords > 20
        || source.results_per_keyword == 0 || source.results_per_keyword > 100
        || source.max_pages_per_keyword == 0 || source.max_pages_per_keyword > 5
        || source.refresh_minutes < 1
        || !source.base_url.starts_with("https://") {
        return Err("Invalid JobSearch provider settings".into());
    }
    Ok(source)
}

// Match whole keywords only: e.g. Git must not match "digital".
pub fn contains_skill(text: &str, keyword: &str) -> bool {
    let text = text.to_lowercase();
    let keyword = keyword.trim().to_lowercase();
    if keyword.is_empty() { return false; }

    text.match_indices(&keyword).any(|(start, _)| {
        let end = start + keyword.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        before.map_or(true, |c| !c.is_alphanumeric())
            && after.map_or(true, |c| !c.is_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundaries() {
        assert!(contains_skill("Rust and Python", "rust"));
        assert!(contains_skill("Machine learning engineer", "Machine learning"));
        assert!(!contains_skill("digital products", "git"));
    }
}
