use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tokio::sync::watch;
use std::fs;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::sqlite_db::DbState;
use crate::jobs::model::JobAd;
use crate::jobs::repo;
use crate::resume;
use crate::search;
use crate::sources::arbetsformedlingen::{client::AfClient, model::AfJobAd, normalize::normalize_job};

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub jobs: Vec<JobAd>,
    pub keywords: Vec<String>,
    pub failed_keywords: Vec<String>,
    pub diagnostics: SearchDiagnostics,
    pub new_jobs: usize,
    pub cached: bool,
    pub refreshed_at: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SearchDiagnostics {
    pub api_hits: usize,
    pub parsed: usize,
    pub active: usize,
    pub keyword_matched: usize,
    pub unique: usize,
    pub pages_fetched: usize,
    pub per_keyword: Vec<KeywordStats>,
}

#[derive(Serialize, Deserialize)]
pub struct KeywordStats {
    pub keyword: String,
    pub available: u64,
    pub fetched: usize,
    pub pages: usize,
}

// JobSearch uses latitude/longitude + radius for geographical filtering.
#[derive(Clone, Serialize, Deserialize)]
pub struct SearchLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: u32,
}

#[derive(Serialize, Deserialize)]
struct SearchCache {
    signature: String,
    saved_at: i64,
    result: SearchResult,
}

#[derive(Clone, Serialize)]
struct SearchProgress {
    current: usize,
    total: usize,
    keyword: String,
    search_id: String,
}

// One cancellation signal per search: allows the HTTP request itself to be
// dropped as soon as Stop is clicked. Unknown IDs are remembered briefly to
// handle clicks made before a search has registered.
#[derive(Default)]
pub struct SearchControl {
    inner: Mutex<SearchRegistry>,
}

#[derive(Default)]
struct SearchRegistry {
    active: HashMap<String, watch::Sender<bool>>,
    pending: HashSet<String>,
}

struct SearchGuard<'a> {
    control: &'a SearchControl,
    id: String,
}

impl Drop for SearchGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.control.inner.lock() {
            state.active.remove(&self.id);
            state.pending.remove(&self.id);
        }
    }
}

#[tauri::command]
pub fn cancel_job_search(control: State<'_, SearchControl>, search_id: String) -> Result<(), String> {
    if search_id.len() > 100 || search_id.is_empty() ||
        !search_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Invalid search ID".into());
    }
    let mut state = control.inner.lock().map_err(|_| "Search lock poisoned".to_owned())?;
    if let Some(sender) = state.active.get(&search_id) {
        let _ = sender.send(true);
    } else {
        // Bounded to prevent unbounded growth from invalid/unmatched IDs.
        if state.pending.len() >= 64 { state.pending.clear(); }
        state.pending.insert(search_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn search_resume_jobs(
    app: AppHandle,
    db: State<'_, DbState>,
    control: State<'_, SearchControl>,
    profile_id: String,
    force_refresh: Option<bool>,
    location: Option<SearchLocation>,
    search_id: String,
) -> Result<SearchResult, String> {
    if !profile_id.chars().all(|character| character.is_ascii_alphanumeric()) {
        return Err("Invalid account ID".into());
    }
    if search_id.len() > 100 || search_id.is_empty() ||
        !search_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Invalid search ID".into());
    }
    // A bounded Swedish location is required. Prevent arbitrary high-radius queries.
    if let Some(ref area) = location {
        if area.name.trim().is_empty() || area.name.len() > 80
            || !area.latitude.is_finite() || !area.longitude.is_finite()
            || !(55.0..=69.2).contains(&area.latitude)
            || !(10.0..=24.3).contains(&area.longitude)
            || !(1..=100).contains(&area.radius_km) {
            return Err("Invalid Swedish area selection".into());
        }
    }
    let coordinates = location.as_ref().map(|area| (area.latitude, area.longitude, area.radius_km));
    let (sender, mut cancelled) = watch::channel(false);
    {
        let mut state = control.inner.lock().map_err(|_| "Search lock poisoned".to_owned())?;
        if state.pending.remove(&search_id) { return Err("Search cancelled".into()); }
        if state.active.contains_key(&search_id) { return Err("Duplicate search ID".into()); }
        state.active.insert(search_id.clone(), sender);
    }
    // RAII cleanup covers successful, failed and cancelled searches.
    let _guard = SearchGuard { control: &control, id: search_id.clone() };
    let profile = resume::get_profile(&app, &profile_id)?;
    let source = search::enabled_jobsearch(&app)?;
    let keywords = resume::search_keywords(&profile, source.max_keywords);
    if keywords.is_empty() {
        return Err("This account has no search keywords. Add some on the Profile page.".into());
    }

    let signature = format!("{}:{}:{}:{}:{}:{}", source.id, source.base_url,
        source.results_per_keyword, source.max_pages_per_keyword, keywords.join("|"),
        location.as_ref().map(|area| format!("{:.5},{:.5}:{}", area.latitude, area.longitude, area.radius_km)).unwrap_or_else(|| "all-sweden".into()));
    let cache_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    fs::create_dir_all(&cache_dir).map_err(|err| err.to_string())?;
    let cache_path = cache_dir.join(format!("search_cache_{profile_id}.json"));
    if !force_refresh.unwrap_or(false) {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(mut cache) = serde_json::from_str::<SearchCache>(&content) {
                if cache.signature == signature &&
                   Utc::now().timestamp().saturating_sub(cache.saved_at) < source.refresh_minutes * 60 {
                    cache.result.cached = true;
                    cache.result.new_jobs = 0;
                    return Ok(cache.result);
                }
            }
        }
    }

    let client = AfClient::new(&source.base_url).map_err(|err| err.to_string())?;
    let mut seen = HashSet::new();
    let mut jobs = Vec::new();
    let mut failed_keywords = Vec::new();
    let mut diagnostics = SearchDiagnostics::default();
    let limit = source.results_per_keyword;
    let max_pages = source.max_pages_per_keyword;

    for (index, keyword) in keywords.iter().enumerate() {
        let mut stats = KeywordStats { keyword: keyword.clone(), available: 0, fetched: 0, pages: 0 };
        for page in 0..max_pages {
            if *cancelled.borrow() { return Err("Search cancelled".into()); }
            let _ = app.emit("job-search-progress", SearchProgress {
                current: index * max_pages as usize + page as usize + 1,
                total: keywords.len() * max_pages as usize,
                keyword: format!("{} (page {})", keyword, page + 1),
                search_id: search_id.clone(),
            });
            let offset = page * limit;
            let request = tokio::select! {
                biased;
                _ = cancelled.changed() => return Err("Search cancelled".into()),
                response = client.search_jobs(keyword, limit, offset, coordinates) => response,
            };
            let response = match request {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("JobSearch '{keyword}', page {}: {error}", page + 1);
                    failed_keywords.push(keyword.to_owned());
                    break;
                }
            };
            let Some(hits) = response.get("hits").and_then(|value| value.as_array()) else {
                failed_keywords.push(keyword.to_owned());
                break;
            };
            if let Some(total) = response.pointer("/total/value").and_then(|value| value.as_u64()) {
                stats.available = total;
            }
            stats.fetched += hits.len();
            stats.pages += 1;
            diagnostics.pages_fetched += 1;
            diagnostics.api_hits += hits.len();

            for hit in hits {
                let Ok(ad) = serde_json::from_value::<AfJobAd>(hit.clone()) else { continue; };
                diagnostics.parsed += 1;
                let job = normalize_job(ad);
                if !job.active { continue; }
                diagnostics.active += 1;
                let text = format!("{} {} {}", job.title,
                    job.description.as_deref().unwrap_or(""), job.occupation.as_deref().unwrap_or(""));
                if !keywords.iter().any(|word| search::contains_skill(&text, word)) { continue; }
                diagnostics.keyword_matched += 1;
                let key = format!("{}:{}", job.source, job.external_id);
                if seen.insert(key) { jobs.push(job); }
            }
            if hits.len() < limit as usize || (stats.available > 0 && (offset as u64 + hits.len() as u64) >= stats.available) {
                break;
            }
        }
        diagnostics.per_keyword.push(stats);
    }

    if diagnostics.pages_fetched == 0 {
        return Err("JobSearch returned no successful pages. Check your connection and provider configuration.".into());
    }
    diagnostics.unique = jobs.len();
    if *cancelled.borrow() { return Err("Search cancelled".into()); }

    let path = db.path.clone();
    let (jobs, new_jobs) = tauri::async_runtime::spawn_blocking(move || {
        let old = repo::get_jobs(&path).map_err(|err| err.to_string())?;
        let old_ids: HashSet<String> = old.iter().map(|job| format!("{}:{}", job.source, job.external_id)).collect();
        let new_jobs = jobs.iter().filter(|job| !old_ids.contains(&format!("{}:{}", job.source, job.external_id))).count();
        repo::save_jobs(&path, &jobs).map_err(|err| err.to_string())?;
        Ok::<(Vec<JobAd>, usize), String>((jobs, new_jobs))
    }).await.map_err(|err| err.to_string())??;

    if *cancelled.borrow() { return Err("Search cancelled".into()); }
    let result = SearchResult {
        jobs, keywords, failed_keywords, diagnostics, new_jobs,
        cached: false, refreshed_at: Utc::now().to_rfc3339(),
    };
    let cache = SearchCache { signature, saved_at: Utc::now().timestamp(), result };
    if let Ok(data) = serde_json::to_vec(&cache) {
        if let Err(err) = fs::write(&cache_path, data) {
            eprintln!("Failed to save search cache: {err}");
        }
    }
    Ok(cache.result)
}

#[tauri::command]
pub async fn get_jobs(db: State<'_, DbState>) -> Result<Vec<JobAd>, String> {
    let path = db.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        repo::get_jobs(&path).map_err(|error| error.to_string())
    }).await.map_err(|err| format!("Database read task failed: {err}"))?
}
