import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";

export default function Settings({ theme, onThemeChange }) {
  const [error, setError] = useState("");
  async function changeTheme(value) {
    try { await onThemeChange(value); setError(""); }
    catch (err) { setError(String(err)); }
  }
  return (
    <section className="page-shell">
      <h1>Settings</h1>
      <div className="panel">
        <h2>Appearance</h2>
        <label htmlFor="theme-select">Theme</label>
        <select id="theme-select" value={theme} onChange={(event) => changeTheme(event.target.value)}>
          <option value="dark">Dark mode</option>
          <option value="light">Light mode</option>
        </select>
      </div>
      <div className="panel">
        <h2>Job search</h2>
        <p>Search results are cached for 4 hours to reduce API traffic. Use <strong>Refresh results</strong> on Home to look for new jobs.</p>
        <p className="muted">Jobsy limits the number of searches to keep the app fast and avoid overloading the job provider.</p>
      </div>
      <div className="panel">
        <h2>Help & support</h2>
        <p>Questions or issues? Email <button className="text-link" onClick={() => openUrl("mailto:lundinoscar4@gmail.com").catch((err) => setError(String(err)))}>lundinoscar4@gmail.com</button>.</p>
        <p className="muted">Jobsy version 0.1.3</p>
      </div>
      <div className="panel">
        <h2>About &amp; licensing</h2>
        <p className="muted">Jobsy © 2026 Oscar Lundin. Licensed under GNU GPLv3. You may use, modify and share Jobsy, including commercially, provided that the GPLv3 requirements are met, including preservation of copyright notices and availability of source code for distributed versions. No warranty is provided.</p>
        <p><button type="button" className="text-link" onClick={() => openUrl("https://www.gnu.org/licenses/gpl-3.0.html").catch((err) => setError(String(err)))}>View GNU GPLv3 license ↗</button></p>
      </div>
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
