# Threepole Custom

Custom build of Threepole for Destiny 2.

This project is based on the original Threepole application by des-sh and adds several custom overlay and tracking features.

---

## Project knowledge base

Start Codex sessions with [`AGENTS.md`](AGENTS.md). Durable context from ChatGPT, Work and Codex lives in the existing `docs/` files:

- [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md) — project goals, safety constraints, workflow and the **Threepole Status** convention
- [`docs/CUSTOM_FEATURES.md`](docs/CUSTOM_FEATURES.md) — custom feature behavior and open feature changes
- [`docs/DESTINY_API.md`](docs/DESTINY_API.md) — Bungie API / manifest architecture, polling and fallback rules
- [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) — responsiveness goals, regressions and validation plan
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — current priorities, open validation and future work
- [`docs/CODEX_HANDOFF.md`](docs/CODEX_HANDOFF.md) — detailed handoff from prior ChatGPT and Work development sessions

Package 1 was implemented on 2026-10-14 for Windows MSI testing; version metadata remains **1.1.4**, so identify the build by commit/workflow. CODEX_HANDOFF records the baseline and validation scope. Codex handles natural-language development orders, tests and Windows MSI delivery; GitHub holds durable history. See [`BUILD-GITHUB-DE.md`](BUILD-GITHUB-DE.md) for the minimal local environment and existing CI build path.

**Permanent goal:** measurably faster, more reliable and richer than original Threepole, at preferably equal or lower resource use and without additional/riskier Destiny interaction. Original is the comparison baseline and minimum standard, not the final target. A measured advantage is not established yet; PROJECT_CONTEXT owns the rules and PERFORMANCE the benchmark method. Further implementation packages require separate user approval.

---

## Original Threepole features

The following functionality comes from the original Threepole project:

- Destiny 2 activity timer
- Overlay mode
- Daily activity clears
- Activity clear notifications
- Timer milliseconds
- Bungie.net profile integration
- Multiple profile/account support
- Background tray operation
- Current activity detection
- Activity history display

## Custom features

The following features were added in this custom build:

### Activity name display

- Displays the current activity name above the timer
- Automatically updates when the activity changes
- Can be enabled or disabled independently

### Configurable activity name auto-hide

- Activity name can automatically disappear after a configurable amount of time
- Auto-hide can be enabled or disabled independently
- Hide delay can be changed at any time

### Per-activity daily clear counter

The overlay counts completed history for the current activity hash; returning to that hash restores its count. Variants with different hashes remain separate. History is filtered at 17:00 UTC by activity start time; the details window retains an aggregate total. The counter's calendar SVG still needs replacement with Eager Edge.

### Additional Raid / Dungeon detection

Manifest activity types and a persistent learned-mode cache supplement missing Raid/Dungeon modes without a manifest request for every history record. Unknown historical hashes with incomplete modes can still be omitted. P1.1 uses existing 204 modes/cached classification to update current activity independently of manifest/name loading, with bounded metadata recovery. Details-window metadata requests remain possible; behavior and remaining limits are recorded in DESTINY_API.

### Current activity timing (1.1.4)

- Independent 204 status with a two-second pause; optional 1000 timing with a ten-second pause, at most six optional requests/minute in steady state.
- Separate freshness guards; newer primary status can accept backward/epoch orbit starts. Optional data cannot establish identity/orbit and must not roll an accepted timer back when missing.
- Confirmed orbit clears optional timing. History notifications never force orbit or stop a later run; display preferences remain independent.

Initialization still waits for current activity and history. Manifest failures/cached empty names can prevent publication/recovery; corrected and late older clears can miss notifications. The specialist files document these defects, request budgets and acceptance tests. Bungie data freshness and real switching latency remain subject to live validation.

### Bungie API affinity handling

One shared HTTP client reuses connections and replays cookies without adding requests. This is intended to improve affinity, but neither proves a backend-routing cause nor guarantees fresher Bungie data. DESTINY_API records the simplified cookie handling and timeout limits.

### Custom application icon

**Eager Edge is required for all Threepole Custom icons**, including overlay-clear. Older original-icon instructions are superseded. App/taskbar/tray/installer and popup assets already use Eager Edge; the counter's calendar SVG remains pending. CUSTOM_FEATURES maps the assets and remaining inventory work. No icons changed in this documentation cleanup.

### Custom updater behavior

The original Threepole automatic updater is disabled in this build to prevent official releases from overwriting the custom version.

---

## Safety / Destiny interaction

Activity data must use official Bungie API/web/manifest sources; ordinary Windows window queries without direct process access are allowed for the separate overlay. No game-process access, injection/hooks, anti-cheat manipulation or app work after full exit is permitted. These requirements have no implicit legacy exceptions.

Activity data comes from Bungie; no gameplay-memory reader, injection, input/gameplay automation or game-file modification was found. **Package 1:** overlay visibility uses freshly read foreground window title/class, visibility and geometry; the inherited process/PID lookup and unbounded identity cache are removed. Matching metadata was observed on the user's game; full Windows/overlay acceptance and CPU/RAM comparison remain pending (see PERFORMANCE). No separate polling service was found. Preserve current window/tray behavior: X removes the visible window while tracking/overlay continue, simple tray click reopens it, and tray Exit ends the app. Verify full exit on Windows; propose minimal corrections only for concrete defects, with separate approval before implementation. No mandatory double-click or blanket window-management rewrite is planned. [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md) owns the action table and safety limits.

---

## Original project

Threepole was originally created by des-sh.

https://github.com/des-sh/threepole

This custom version is based on that project and retains the original project license.

Use the original repository as a continuing technical reference for future optimizations: compare relevant original/Custom code paths and record the compared commits/versions alongside performance evidence.

---

## License

GPL-3.0
