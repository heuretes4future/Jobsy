import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const preferred = ["Python", "Rust", "React", "Docker", "Machine learning", "JavaScript", "TypeScript", "Java", "SQL", "PostgreSQL", "Linux", "C++", "Spring Boot", "Kubernetes", "Mjukvaruutvecklare", "PyTorch", "Git", "Flask"];

export default function Profile({ profiles, activeProfileId, onSelect, onReload }) {
  const [newKeyword, setNewKeyword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [keywords, setKeywords] = useState([]);
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const active = profiles.find(([id]) => id === activeProfileId)?.[1];

  useEffect(() => {
    setKeywords(active ? (
      active.keyword_mode || active.search_keywords?.length
        ? active.search_keywords || []
        : [...(active.technical_skills || [])].sort((a, b) => {
            const rank = (word) => {
              const index = preferred.findIndex((item) => item.toLowerCase() === word.toLowerCase());
              return index === -1 ? 999 : index;
            };
            return rank(a) - rank(b);
          })
    ) : []);
    setNewKeyword("");
    setDisplayName(active?.display_name || "user");
    setMessage("");
  }, [activeProfileId, profiles]);

  function addKeyword(event) {
    event.preventDefault();
    const value = newKeyword.trim();
    if (!value) return;
    if (value.length > 80) return setMessage("Keywords must be 80 characters or shorter.");
    if (keywords.some((keyword) => keyword.toLowerCase() === value.toLowerCase())) return setMessage("Keyword already added.");
    if (keywords.length >= 100) return setMessage("Maximum 100 keywords.");
    setKeywords((current) => [value, ...current]);
    setNewKeyword("");
    setMessage("");
  }

  async function saveKeywords() {
    setBusy(true);
    try {
      await invoke("save_profile_keywords", { profileId: activeProfileId, keywords });
      await onReload();
      setMessage("Keywords saved. The next search will use your changes.");
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function saveName(event) {
    event.preventDefault();
    const name = displayName.trim();
    if (!name || name.length > 50) { setMessage("Please enter a name of 1–50 characters."); return; }
    setBusy(true);
    try {
      await invoke("rename_profile", { profileId: activeProfileId, displayName: name });
      await onReload();
      setMessage("Account name saved.");
    } catch (err) { setMessage(String(err)); }
    finally { setBusy(false); }
  }

  async function deleteAccount() {
    if (!window.confirm(`Permanently remove account ${active?.display_name || "user"} and its keywords? Job and Apply history will be kept.`)) return;
    setBusy(true);
    try {
      await invoke("remove_profile", { profileId: activeProfileId });
      await onReload();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleSelect(event) {
    try {
      await onSelect(event.target.value);
    } catch (err) {
      setMessage(String(err));
    }
  }

  return (
    <section className="page-shell">
      <h1>Profile</h1>
      {profiles.length === 0 ? (
        <div className="panel"><p>No account. Upload a résumé from Home to create one.</p></div>
      ) : (
        <>
          <div className="panel">
            <h2>Active account</h2>
            <label htmlFor="account-select">Choose which résumé/account to search with</label>
            <select id="account-select" value={activeProfileId || ""} onChange={handleSelect}>
              {profiles.map(([id, profile]) => <option value={id} key={id}>{profile.display_name || "user"}</option>)}
            </select>
            <form className="profile-name-form" onSubmit={saveName}>
              <label htmlFor="profile-display-name">Account name</label>
              <div className="profile-name-row">
                <input id="profile-display-name" value={displayName} onChange={(event) => setDisplayName(event.target.value)} maxLength={50} placeholder="user" aria-label="Account name" />
                <button className="secondary-button" type="submit" disabled={busy || !displayName.trim()}>Save name</button>
              </div>
            </form>
            <p className="muted">Country: {active?.country || "Not specified"} · {active?.technical_skills?.length || 0} skills extracted from résumé</p>
            <button className="danger-button" disabled={busy} onClick={deleteAccount}>Remove account</button>
          </div>
          <div className="panel">
            <h2>Search keywords</h2>
            <p className="muted">Add skills or job titles the résumé extractor missed. Keywords are searched in this order, up to the limit in job_sources.json. New additions go to the front. Dragging isn't required: use the arrows to reorder.</p>
            <div className="keyword-list">
              {keywords.map((keyword) => (
                <span className="keyword-chip" key={keyword}>
                  {keyword}
                  <button type="button" aria-label={`Move ${keyword} up`} onClick={() => setKeywords((current) => {
                    const items = [...current];
                    const index = items.indexOf(keyword);
                    if (index > 0) [items[index - 1], items[index]] = [items[index], items[index - 1]];
                    return items;
                  })}>↑</button>
                  <button type="button" aria-label={`Remove ${keyword}`} onClick={() => setKeywords((current) => current.filter((item) => item !== keyword))}>×</button>
                </span>
              ))}
              {keywords.length === 0 && <p className="muted">No keywords. Add at least one to search.</p>}
            </div>
            <form className="inline-form" onSubmit={addKeyword}>
              <input value={newKeyword} onChange={(event) => setNewKeyword(event.target.value)} placeholder="Add keyword (e.g. Kubernetes)" maxLength={80} />
              <button type="submit" className="secondary-button">Add</button>
            </form>
            <button className="primary-button" disabled={busy} onClick={saveKeywords}>Save keywords</button>
          </div>
        </>
      )}
      {message && <p role="status" className="status-message">{message}</p>}
    </section>
  );
}
