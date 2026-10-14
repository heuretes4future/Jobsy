# Jobsy — Find jobs that match your skills

**Find relevant Swedish job listings without searching dozens of websites.** Jobsy is a free desktop app that reads the skills in your résumé, searches for matching job opportunities, and helps you keep track of the job ads you have opened to apply.

You don't need to know how to code. Upload your résumé, choose an area (or search all of Sweden), and press **Find matching jobs**.

> **Important:** The **Apply** button opens an employer's application page and records that you opened it. Jobsy **does not submit** your application or verify whether you completed it. The Overview is a history of opened application links, not confirmed submissions.

## Download Jobsy for Windows

[![Download Jobsy for Windows](https://img.shields.io/badge/Download_for_Windows-.exe-7050c8?logo=windows&logoColor=white)](../../releases/latest/download/Jobsy-Windows-Setup.exe)

**[Download Jobsy for Windows (.exe)](../../releases/latest/download/Jobsy-Windows-Setup.exe)**

1. Click **Download Jobsy for Windows** above to download `Jobsy-Windows-Setup.exe` directly. If you prefer to inspect the download first, visit [GitHub Releases](../../releases/latest).
2. Run the installer, then launch Jobsy from the Start menu.
3. You can upload a résumé PDF and start searching right away. No account registration or technical setup is required.

The download becomes available **after the first GitHub Release is published**. Until then, this link won't have an installer. Unsigned installers can trigger a Windows SmartScreen warning; do not bypass security warnings unless you trust the source. For wider distribution, code signing is recommended.

## What Jobsy can do

- **Résumé-based search:** Extract skills from a PDF résumé, then search Swedish job listings with a limited set of keywords.
- **Editable search terms:** Add and remove keywords if the résumé extractor missed something. Prioritize the most useful ones in Profile.
- **Search by area:** Start typing a Swedish city and select a suggestion, or leave it blank to search all of Sweden. Each city uses a radius around its center; it is **not** an exact municipal boundary.
- **Sort job listings:** Browse matching jobs by relevance or date.
- **Review each job:** See the position, employer, location, type of employment and expandable job description.
- **Open application links:** Use **Apply** to visit the employer's site. Jobsy remembers that you opened it and saves the date.
- **Application history:** See your opened application links in Overview, even after closing the app.
- **Manage profiles:** Upload multiple résumés, choose the active profile, rename it, edit keywords or remove a profile.
- **Progress and cancellation:** See the search indicator and stop a search whenever you want.
- **Light and dark themes:** Choose the appearance in Settings.

### A quick example

Someone with **Python, SQL, and Docker** on their résumé uploads their PDF, adds **Data Engineer** as an extra keyword in Profile, selects **Stockholm** from Search area, and clicks **Find matching jobs**. Jobsy searches Jobtech for relevant postings near Stockholm. Clicking Apply opens the employer's page and adds the listing to Overview for later reference.

## Privacy and personal data

Jobsy does not require creating an online account. Your extracted résumé data, named profiles, saved application-click history, preferences, search cache and job database are stored **locally in the application's user-data directory**. They are not included in the public repository or the installer.

**Existing test builds:** This public-release build uses the app identifier `com.jobsy.desktop`, so it has a new app-data folder. In development, it can import a legacy `resume_data.json` and `Store/applied.json` from the project root once. If you already installed an older packaged build, back up the old app-data directory and migrate it before uninstalling; this version does not automatically find older installed-app identifiers.

The app **does send search keywords and any chosen location to Jobtech's JobSearch API** to retrieve job ads. Opening Apply sends you to third-party websites whose own privacy policies apply. Jobsy currently has no cloud sync or cross-device sharing. Only upload résumés you are comfortable processing locally, and review third-party job links before entering personal details.

### Important accuracy limitations

Jobsy searches a limited number of keywords and pages per keyword to avoid excessive API traffic. It cannot guarantee it has found every matching job or that job ads remain open. Skills-based matching is a starting point, not a hiring-eligibility assessment. **The current PDF extractor reads the first page of a résumé**; use Profile to add any skills from later pages. Application history records clicks only.

## Troubleshooting

**No account** — Go to Home and upload a PDF résumé. On Profile you can select your résumé and edit its search keywords.

**Few or no results** — Check Profile > Search keywords. Clear Search area to include all of Sweden. Try a broader keyword such as a job title. Use Refresh results to bypass the cache.

**Wrong area** — Select one of the suggested city names; the search field doesn't accept arbitrary location strings.

**A job link won't open** — It may have expired or be temporarily unavailable. You can try another listing.

**Stopped search** — Search cancellation won't erase previously saved jobs or Apply history. You can start another search whenever you want.

**Need help?** Email [lundinoscar4@gmail.com](mailto:lundinoscar4@gmail.com). You can also [follow the original author on LinkedIn](https://www.linkedin.com/in/oscar-lundin-92b5a41b4).

---

## For developers and contributors

Jobsy is a desktop application built with **React + Vite** for the user interface and **Tauri 2 + Rust** for filesystem storage and API access. The local job index uses SQLite. Search results come from the Swedish Jobtech JobSearch API. The repository contains code and non-personal configuration, **not a copy of anyone's résumé or job-application history**.

### Prerequisites

For Windows development, install Node.js 22+, Rust and the Windows prerequisites from [Tauri's setup guide](https://v2.tauri.app/start/prerequisites/).

```bash
npm ci
npm run tauri dev
```

To build a local Windows installer:

```bash
npm run tauri build
```

The generated NSIS `.exe` installer will be under `src-tauri/target/release/bundle/nsis/`.

### Project structure

| Path | What it contains |
| --- | --- |
| `src-ui/` | React pages and reusable components |
| `src-tauri/src/` | Rust commands, job searching, skill extraction and local data |
| `src-tauri/skills-table/data.db` | Bundled skill taxonomy for résumé parsing |
| `Store/job_sources.json` | Public API provider configuration and request limits |
| `.github/workflows/windows-release.yml` | Windows installer GitHub Actions workflow |
| `LICENSE` | GNU GPL version 3 |

### How search works

Jobsy extracts skills from a PDF résumé into a private local profile. For each search, it selects up to eight ordered keywords and requests up to two pages per keyword (50 listings per page by default). Selecting a city adds a Jobtech **position + radius** filter to each request. Jobsy parses listings, excludes inactive or irrelevant matches, removes duplicates, stores jobs locally and displays results. A four-hour cache reduces repeated requests; **Refresh results** bypasses it. Diagnostics reveal fetched, matched and unique counts.

The API and limits are configured in `Store/job_sources.json`; queries are sent through the Rust JobSearch client. Network availability and provider-side limits apply.

### Private data and public repository hygiene

Do not commit personal profile, history or API credentials. In particular:

- `resume_data.json` and `Store/applied.json` are excluded by `.gitignore` for migration compatibility; current data is stored in the Tauri app-data directory.
- `.env` files and Rust build artifacts are ignored.
- Never add a real résumé, private email conversations, passwords, tokens, personal job histories or screenshots containing personal information to the repository.
- Run `git status --short` and inspect the repository contents before the first public push. **A `.gitignore` does not remove files previously committed or erase their history.

### Contributing and licensing

Jobsy is Copyright © 2026 **Oscar Lundin** and is released under the **GNU General Public License version 3.0 only (GPL-3.0-only)**. See [LICENSE](LICENSE) for the full official license. You may run, modify, redistribute and even sell copies, **provided you comply with GPLv3**, including preserving notices, identifying modifications and providing the corresponding source code when distributing versions. Derivatives must remain GPL-licensed.

This application is distributed **without any warranty**. For license text and terms, see `LICENSE` and [GNU GPLv3](https://www.gnu.org/licenses/gpl-3.0.html).

### Third-party data

Job listings are provided by [Jobtech](https://jobtechdev.se/); Jobsy is not affiliated with or endorsed by Arbetsförmedlingen, Jobtech or employers whose listings appear in searches. Respect the API provider's rate limits and job-ad terms.
