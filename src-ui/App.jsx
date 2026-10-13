import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Routes, Route } from "react-router-dom";
import "./App.css";
import Sidebar from "./Components/Sidebar";
import Dashboard from "./Components/Dashboard";
import Overview from "./Pages/Overview";
import Profile from "./Pages/Profile";
import Settings from "./Pages/Settings";

export default function App() {
  const [profiles, setProfiles] = useState([]);
  const [preferences, setPreferences] = useState({ active_profile_id: null, theme: "dark" });
  const [error, setError] = useState("");
  const [ready, setReady] = useState(false);

  async function reload() {
    try {
      const [accounts, prefs] = await Promise.all([
        invoke("list_profiles"),
        invoke("get_preferences"),
      ]);
      setProfiles(accounts);
      setPreferences(prefs);
      setError("");
    } catch (err) {
      setError(String(err));
    } finally {
      setReady(true);
    }
  }

  useEffect(() => { reload(); }, []);
  useEffect(() => {
    document.documentElement.dataset.theme = preferences.theme;
  }, [preferences.theme]);

  async function savePreferences(next) {
    await invoke("save_preferences", { preferences: next });
    setPreferences(next);
  }

  async function selectProfile(profileId) {
    await savePreferences({ ...preferences, active_profile_id: profileId });
  }

  async function selectTheme(theme) {
    await savePreferences({ ...preferences, theme });
  }

  async function onResumeUploaded(profileId) {
    await reload();
    // Select the newly created profile for all future searches.
    await invoke("save_preferences", {
      preferences: { ...preferences, active_profile_id: profileId },
    });
    setPreferences((current) => ({ ...current, active_profile_id: profileId }));
  }

  return (
    <main className="app">
      <Sidebar />
      {error && <p className="app-error" role="alert">{error}</p>}
      {!ready ? <div className="page-shell"><p>Loading Jobsy...</p></div> : (
        <Routes>
          <Route path="/" element={<Dashboard activeProfileId={preferences.active_profile_id} onResumeUploaded={onResumeUploaded} />} />
          <Route path="/Overview" element={<Overview />} />
          <Route path="/Profile" element={<Profile profiles={profiles} activeProfileId={preferences.active_profile_id} onSelect={selectProfile} onReload={reload} />} />
          <Route path="/Settings" element={<Settings theme={preferences.theme} onThemeChange={selectTheme} />} />
        </Routes>
      )}
    </main>
  );
}
