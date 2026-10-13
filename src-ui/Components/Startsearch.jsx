import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Results from "./Results.jsx";

export default function Search({ sort, profileId, location, locationInvalid = false }) {
  const [showResults, setShowResults] = useState(false);
  const [loading, setLoading] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [stopped, setStopped] = useState(false);
  const [jobs, setJobs] = useState([]);
  const [keywords, setKeywords] = useState([]);
  const [failedKeywords, setFailedKeywords] = useState([]);
  const [error, setError] = useState("");
  const [progress, setProgress] = useState(null);
  const [diagnostics, setDiagnostics] = useState(null);
  const [searchInfo, setSearchInfo] = useState(null);
  const activeSearchId = useRef(null);

  useEffect(() => {
    return () => {
      if (activeSearchId.current) {
        invoke("cancel_job_search", { searchId: activeSearchId.current }).catch(() => {});
      }
    };
  }, []);

  useEffect(() => {
    setShowResults(false);
    setJobs([]);
    setDiagnostics(null);
    setSearchInfo(null);
    setStopped(false);
  }, [profileId]);

  async function handleSearch(forceRefresh = false) {
    if (activeSearchId.current || !profileId) return;
    const searchId = crypto.randomUUID();
    activeSearchId.current = searchId;
    setShowResults(true);
    setLoading(true);
    setStopping(false);
    setStopped(false);
    setError("");
    setProgress(null);
    setFailedKeywords([]);
    let unlisten;
    try {
      unlisten = await listen("job-search-progress", (event) => {
        if (event.payload.search_id === searchId) setProgress(event.payload);
      });
      const result = await invoke("search_resume_jobs", {
        profileId,
        forceRefresh,
        searchId,
        location,
      });
      // Results from a cancelled/stale search must not replace the current results.
      if (activeSearchId.current !== searchId) return;
      setJobs(result.jobs);
      setKeywords(result.keywords);
      setFailedKeywords(result.failed_keywords);
      setDiagnostics(result.diagnostics);
      setSearchInfo({ cached: result.cached, newJobs: result.new_jobs, refreshedAt: result.refreshed_at });
    } catch (err) {
      if (activeSearchId.current !== searchId) return;
      if (String(err).toLowerCase().includes("cancelled")) setStopped(true);
      else setError(String(err));
    } finally {
      unlisten?.();
      if (activeSearchId.current === searchId) {
        activeSearchId.current = null;
        setLoading(false);
        setStopping(false);
      }
    }
  }

  async function stopSearch() {
    const searchId = activeSearchId.current;
    if (!searchId || stopping) return;
    setStopping(true);
    try {
      await invoke("cancel_job_search", { searchId });
    } catch (err) {
      setError(`Could not stop search: ${String(err)}`);
      setStopping(false);
    }
  }

  return (
    <div className={`search-area ${showResults ? "active" : ""}`}>
      {!profileId ? (
        <p className="empty-note">No account selected. Upload a résumé or choose an account in Profile.</p>
      ) : (
        <div className="search-actions">
          <button type="button" className="search-button" onClick={() => handleSearch(false)} disabled={loading || locationInvalid}>
            <span className="search-button-icon" aria-hidden="true">⌕</span>
            <span>{loading ? "Finding your matches..." : "Find matching jobs"}</span>
            <span className="search-button-arrow" aria-hidden="true">→</span>
          </button>
          {loading ? (
            <button className="stop-search-button" type="button" disabled={stopping} onClick={stopSearch}>
              {stopping ? "Stopping..." : "■ Stop search"}
            </button>
          ) : showResults ? (
            <button type="button" className="secondary-button search-refresh" disabled={locationInvalid} onClick={() => handleSearch(true)}>↻ Refresh results</button>
          ) : null}
        </div>
      )}
      {showResults && (
        <Results
          jobs={jobs}
          sort={sort}
          keywords={keywords}
          loading={loading}
          stopped={stopped}
          error={error}
          failedKeywords={failedKeywords}
          progress={progress}
          diagnostics={diagnostics}
          searchInfo={searchInfo}
        />
      )}
    </div>
  );
}
