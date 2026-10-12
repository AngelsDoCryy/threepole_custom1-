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

Documentation aligned 2026-10-11 against **1.1.4**; this cleanup implements no fixes. CODEX_HANDOFF records the audited commit and successful build #19. Codex handles natural-language development orders, tests and Windows MSI delivery; GitHub holds durable history. See [`BUILD-GITHUB-DE.md`](BUILD-GITHUB-DE.md) for the minimal local environment and existing CI build path.

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

Manifest activity types and a persistent learned-mode cache supplement missing Raid/Dungeon modes without a manifest request for every history record. Unknown historical hashes with incomplete modes can still be omitted. Details-window metadata requests remain possible; manifest publication/recovery defects are recorded in DESTINY_API.

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

The required design is API-/web-/manifest-only, with no direct Destiny process access, hooks or activity after application closure. These requirements have no implicit legacy exceptions.

Activity data comes from Bungie; no gameplay-memory reader, injection, input/gameplay automation or game-file modification was found. **Unresolved conflicts:** inherited overlay visibility opens a process to identify `destiny2.exe`, and window close leaves tray/polling active until Exit. No separate polling service was found. [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md) records the strict requirements; safety/lifecycle review and Windows shutdown validation remain open.

---

## Original project

Threepole was originally created by des-sh.

https://github.com/des-sh/threepole

This custom version is based on that project and retains the original project license.

---

## License

GPL-3.0
