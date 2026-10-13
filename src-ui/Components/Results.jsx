import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { sortJobsByRelevance, sortJobsByDate } from "../Utils/lib.js";

export default function Results({ jobs = [], sort, keywords = [], loading = false, error = "", failedKeywords = [], progress = null, diagnostics = null, searchInfo = null, stopped = false }) {
  const [expandedJob, setExpandedJob] = useState(null);
  const [clickedJobs, setClickedJobs] = useState(new Set());
  const [applyError, setApplyError] = useState("");

  const displayedJobs = useMemo(() => {
    if (sort === "relevant") return sortJobsByRelevance(jobs, keywords);
    if (sort === "new") return sortJobsByDate(jobs, "new");
    if (sort === "old") return sortJobsByDate(jobs, "old");
    return jobs;
  }, [jobs, sort, keywords]);

  async function handleApply(job) {
    const id = `${job.source}:${job.external_id}`;
    const url = job.application_url || job.listing_url;
    if (!url) {
      setApplyError("This job has no application link.");
      return;
    }
    setApplyError("");
    try {
      // The click is recorded, not a confirmed application submission.
      await invoke("mark_apply_clicked", { job });
      setClickedJobs((previous) => new Set([...previous, id]));
    } catch (err) {
      setApplyError(`Could not save this job in Overview: ${String(err)}`);
      return;
    }
    try {
      await openUrl(url);
    } catch (err) {
      setApplyError(`Job saved in Overview, but the browser could not open: ${String(err)}`);
    }
  }

  const pages = [];
  for (let i = 0; i < displayedJobs.length; i += 5) pages.push(displayedJobs.slice(i, i + 5));

  return (
    <div className="results-layout">
      {loading && (
        <div className="search-progress" role="status" aria-live="polite" aria-label="Job search in progress">
          <div className="search-progress-head">
            <span className="search-progress-spinner" aria-hidden="true" />
            <span>{progress ? `Searching ${progress.keyword}` : "Preparing your search..."}</span>
            {progress && <span className="search-progress-count">Request {progress.current} / {progress.total} maximum</span>}
          </div>
          <div className="search-progress-track" role="progressbar" aria-label="Searching job listings" aria-valuetext="Search in progress">
            <div className="search-progress-fill" />
          </div>
          <span className="search-progress-caption">Searching multiple pages. You can stop at any time.</span>
        </div>
      )}
      {searchInfo && !loading && (
        <p className="results-notice" role="status">
          {searchInfo.cached ? "Cached search" : "Updated search"} · {displayedJobs.length} matching jobs · {searchInfo.newJobs} newly discovered
        </p>
      )}
      {stopped && !loading && <p className="search-stopped" role="status">Search stopped. Start another search whenever you’re ready.</p>}
      {error && <p role="alert">{error}</p>}
      {applyError && <p role="alert">{applyError}</p>}
      {failedKeywords.length > 0 && <p role="status">Some searches failed: {failedKeywords.join(", ")}</p>}
      {diagnostics && (
        <details className="search-diagnostics">
          <summary>Search diagnostics</summary>
          <div className="diagnostics-grid">
            <p>Keywords: {keywords.join(", ")}</p>
            <p>API hits: {diagnostics.api_hits}</p>
            <p>Parsed: {diagnostics.parsed}</p>
            <p>Active: {diagnostics.active}</p>
            <p>Keyword matched: {diagnostics.keyword_matched}</p>
            <p>Unique: {diagnostics.unique}</p>
            <p>Pages fetched: {diagnostics.pages_fetched}</p>
            {diagnostics.per_keyword?.map((item) => (
              <p key={item.keyword}>{item.keyword}: {item.fetched} fetched / {item.available} available ({item.pages} pages)</p>
            ))}
          </div>
        </details>
      )}
      <div className="results-container" aria-busy={loading}>
        {!loading && !error && !stopped && displayedJobs.length === 0 && <p className="empty-note">No matching jobs found.</p>}
        {pages.map((page, pageIndex) => (
          <div className="results-page" key={pageIndex}>
            {page.map((job) => {
              const id = `${job.source}:${job.external_id}`;
              const isExpanded = expandedJob === id;
              return (
                <article className={`job-card ${isExpanded ? "job-card-expanded" : ""}`} key={id}>
                  <h3>{job.title}</h3>
                  <p>{job.employer_name || "Unknown employer"}</p>
                  <p>{job.city || "Unknown location"}</p>
                  {job.employment_type && <p>Employment: {job.employment_type}</p>}
                  <button className="apply-button" type="button" onClick={() => handleApply(job)} disabled={!job.application_url && !job.listing_url}>
                    {clickedJobs.has(id) ? "Apply again ↗" : "Apply ↗"}
                  </button>
                  {isExpanded && <div className="job-description"><p>{job.description || "No description available."}</p></div>}
                  <button
                    className="job-expand-button"
                    onClick={() => setExpandedJob((current) => current === id ? null : id)}
                    aria-label={isExpanded ? "Hide job description" : "Show job description"}
                    aria-expanded={isExpanded}
                  >
                    <span className={`job-expand-arrow ${isExpanded ? "open" : ""}`}>⌄</span>
                  </button>
                </article>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
