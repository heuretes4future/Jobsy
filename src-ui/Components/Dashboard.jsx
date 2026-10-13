import { useState } from "react";
import Dropdown from "./Dropdown.jsx";
import Search from "./Startsearch.jsx";
import { usePdfLoad } from "../Utils/lib.js";
import { SWEDISH_AREAS } from "../Utils/areas.js";

export default function Dashboard({ activeProfileId, onResumeUploaded }) {
  const { pdfRunLoad } = usePdfLoad();
  const [sort, setSort] = useState("relevant");
  const [areaQuery, setAreaQuery] = useState("");
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState("");
  const [uploadSuccess, setUploadSuccess] = useState("");

  async function uploadResume() {
    setUploading(true);
    setError("");
    setUploadSuccess("");
    try {
      const profileId = await pdfRunLoad();
      if (profileId) {
        await onResumeUploaded(profileId);
        setUploadSuccess("Résumé added successfully · Ready to find your matches");
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setUploading(false);
    }
  }

  const location = SWEDISH_AREAS.find((area) => area.name.toLocaleLowerCase("sv-SE") === areaQuery.trim().toLocaleLowerCase("sv-SE")) || null;
  const locationInvalid = areaQuery.trim() !== "" && !location;

  return (
    <section className="dashboard">
      <h1>Jobsy</h1>
      <div className="dashboard-center">
        <div className="resume-upload">
          <button id="upload-button" onClick={uploadResume} disabled={uploading}>
            {uploading ? "Reading résumé..." : "Upload resume"}
          </button>
          {uploadSuccess && (
            <p className="resume-upload-success" role="status">
              <span className="resume-check" aria-hidden="true">✓</span>
              {uploadSuccess}
            </p>
          )}
          {error && <p className="resume-upload-error" role="alert">{error}</p>}
        </div>
        <div className="search-toolbar">
          <div className="toolbar-field">
            <label htmlFor="sort-select">Sort jobs</label>
            <Dropdown
              id="sort-select"
              value={sort}
              onChange={setSort}
              options={[
                { title: "Relevant", value: "relevant" },
                { title: "Recent", value: "new" },
                { title: "Oldest", value: "old" },
              ]}
            />
          </div>
          <div className="toolbar-field area-field">
            <label htmlFor="job-area">Search area</label>
            <input
              id="job-area"
              list="swedish-job-areas"
              type="search"
              value={areaQuery}
              onChange={(event) => setAreaQuery(event.target.value)}
              placeholder="All of Sweden · type a city"
              autoComplete="off"
              aria-invalid={locationInvalid}
              aria-describedby={locationInvalid ? "area-hint" : undefined}
            />
            <datalist id="swedish-job-areas">
              {SWEDISH_AREAS.map((area) => <option key={area.name} value={area.name} />)}
            </datalist>
          </div>
        </div>
        {locationInvalid && <p className="area-hint" id="area-hint">Choose a suggested Swedish city or clear the field for all of Sweden.</p>}
        <Search sort={sort} profileId={activeProfileId} location={location} locationInvalid={locationInvalid} />
      </div>
    </section>
  );
}
