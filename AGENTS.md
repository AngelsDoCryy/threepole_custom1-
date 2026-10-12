# Codex instructions for Threepole Custom

## Start of each project session

Read [CODEX_HANDOFF](docs/CODEX_HANDOFF.md), [PROJECT_CONTEXT](docs/PROJECT_CONTEXT.md) and [ROADMAP](docs/ROADMAP.md) before substantive work, then relevant [CUSTOM_FEATURES](docs/CUSTOM_FEATURES.md), [DESTINY_API](docs/DESTINY_API.md) and [PERFORMANCE](docs/PERFORMANCE.md). On a new takeover, read all six and README. Follow links explicitly: instruction discovery does not automatically load linked files.

Check checkout, branch, working changes and PR head. Report missing context instead of inventing prior chat decisions. Use these existing knowledge files; do not create parallel handoff/status structures. Distinguish verified implementation, requirements, proposals and live evidence. Keep details in their owning file.

## Repository

- Repository: `AngelsDoCryy/threepole_custom1-`; custom base: original Threepole 1.1.2.
- Active branch: `optimize-speed-cache-icon`; draft PR #1 targets `main`; custom version: 1.1.4.
- Do not work directly on `main` or merge PR #1 without explicit instruction.
- Preserve original behavior/performance except where a custom requirement intentionally changes it.

## Hard safety constraints

Required design: API-/web-/manifest-only and non-invasive toward Destiny 2. Existing code is not an implicit exception. Never add:

- Destiny 2 process memory reads/writes or direct process access/inspection;
- DLL/code injection, hooks into `destiny2.exe`, DirectX/game/RTSS-style overlay hooks;
- input hooks, gameplay/input automation or game-file edits;
- an always-running service, scheduled task or persistent helper/runtime dependency.

When Threepole is closed, custom polling and work must stop.

**Unresolved conflicts:** overlay visibility currently opens the foreground process to identify `destiny2.exe`; UI close leaves the tray app polling until Exit. Do not describe either as compliant or grandfather it. PROJECT_CONTEXT records the findings; ROADMAP makes safety/lifecycle the first gate. Documentation does not authorize a runtime change or an exception.

## Priorities and invariants

- Keep startup, activity/orbit/restart detection and overhead close to original Threepole. Minimize requests and unnecessary work; prefer simple, focused changes.
- Preserve independent 204 status (two-second pause), optional 1000 timing (ten-second pause) and history paths. Independent steady-state polling does **not** mean metadata or initialization is nonblocking; manifest/cache and initial-history defects remain open.
- Optional timing failure must not block identity/orbit or roll accepted timing back. Confirmed orbit clears stale optional timing.
- History clears/notifications must not force orbit or stop a later run. Preserve per-hash daily counts and independently configurable timer, name, auto-hide, counter and notifications.
- Raid/dungeon classification uses cached/manifest data without per-history-item classification requests. Do not restore removed activity-type icon requests.
- **Eager Edge is required for all Threepole Custom icons**, including app/taskbar/tray/installer, notifications and overlay-clear. Older original-icon instructions are superseded. The clear counter still has a calendar SVG; replacement is pending.
- Keep the official updater disabled and the custom installation/config separate.
- Preserve earlier optimizations and live-test gaps in PERFORMANCE. Unit tests cannot establish that reported orbit/repeated-Dungeon delays are fixed.

## Validation and delivery

For implementation changes, run or account for:

- `yarn install` and `yarn build`;
- `node --test tests/overlay-visibility.test.mjs tests/overlay-activity.test.mjs`;
- `cargo test --release --manifest-path src-tauri/Cargo.toml --bin app`;
- Windows MSI via `yarn tauri build` for runtime/packaging changes.

The Windows workflow is the reference build path: Tauri includes Vite build and currently precedes Rust tests to configure Cargo features. Vite is not a standalone TypeScript/Svelte type check. For documentation-only work, check scope, links and consistency; do not install tools or claim fresh runtime validation. See [BUILD-GITHUB-DE](BUILD-GITHUB-DE.md) for environment tiers.

## Change discipline

- Inspect implementation first; add focused regressions for timing/state bugs. Keep version notes and owning knowledge files aligned with shipped behavior.
- State API request impact. Do not raise polling frequency without evidence and a quantified budget. Assess startup, locks, caches, settings, profile reset, history and shutdown for relevant changes.
- Separate observed behavior, local-code causes and unverified Bungie-side causes; distinguish automated checks from live Destiny/API evidence.
- The user gives ordinary-language tasks. Handle implementation, tests and identifiable Windows MSI delivery without requiring them to program. Keep unrelated changes separate. Do not install additional programs/services without authorization.
