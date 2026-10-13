import { useState } from "react";
import { Link } from "react-router-dom";
import { openUrl } from "@tauri-apps/plugin-opener";

const LINKEDIN_URL = "https://www.linkedin.com/in/oscar-lundin-92b5a41b4";

export default function Sidebar() {
  const [linkError, setLinkError] = useState("");

  async function openLinkedIn() {
    try {
      await openUrl(LINKEDIN_URL);
      setLinkError("");
    } catch (error) {
      setLinkError(`Could not open LinkedIn: ${String(error)}`);
    }
  }

  return (
    <aside className="sidebar">
      <nav className="sidebar-nav" aria-label="Main navigation">
        <Link to="/">Home</Link>
        <Link to="/Overview">Overview</Link>
        <Link to="/Profile">Profile</Link>
        <Link to="/Settings">Settings</Link>
      </nav>
      <div className="sidebar-footer">
        <button className="linkedin-link" type="button" onClick={openLinkedIn}>
          <span className="linkedin-logo" aria-hidden="true">in</span>
          <span>Follow me on LinkedIn ↗</span>
        </button>
        {linkError && <p className="sidebar-link-error" role="alert">{linkError}</p>}
      </div>
    </aside>
  );
}
