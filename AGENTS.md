# Codex instructions for Threepole Custom

## Start of each project session

Read [CODEX_HANDOFF](docs/CODEX_HANDOFF.md), [PROJECT_CONTEXT](docs/PROJECT_CONTEXT.md) and [ROADMAP](docs/ROADMAP.md) before substantive work, then relevant [CUSTOM_FEATURES](docs/CUSTOM_FEATURES.md), [DESTINY_API](docs/DESTINY_API.md) and [PERFORMANCE](docs/PERFORMANCE.md). On a new takeover, read all six and README. Follow links explicitly: instruction discovery does not automatically load linked files.

Check checkout, branch, working changes and PR head. Report missing context instead of inventing prior chat decisions. Use these existing knowledge files; do not create parallel handoff/status structures. Distinguish verified implementation, requirements, proposals and live evidence. Keep details in their owning file.

## Repository

- Repository: `AngelsDoCryy/threepole_custom1-`; custom base: original Threepole 1.1.2.
- Active branch: `optimize-speed-cache-icon`; draft PR #1 targets `main`; custom version: 1.1.4.
- Do not work directly on `main` or merge PR #1 without explicit instruction.
- Original Threepole 1.1.2 is the permanent technical comparison baseline and minimum standard, not the final performance goal. Preserve existing features/settings except for explicitly requested changes.

## Permanent development objective

Make Threepole Custom measurably faster, more reliable and richer in useful features than original Threepole, at preferably equal or lower resource use and without additional or riskier Destiny interaction. Priority: safety/account protection, correctness/reliability, speed beyond original, low resource use, then convenience. Optimize speed and resources together; significantly higher resource use is not an acceptable shortcut. Explain even a small proposed resource increase and obtain the user's decision before accepting that tradeoff.

Optimize the full Bungie API → retrieval → processing → state → overlay path through simple architecture, bounded parallelism, caching and removal of local waits. Separate Bungie freshness from local latency. Do not claim a speed/resource advantage without reproducible evidence; PERFORMANCE owns comparison methods and open live tests.

## Hard safety constraints

Activity data must use official Bungie APIs/web/manifest data. The overlay must remain a separate Windows window; ordinary window queries without direct game-process access are allowed. Existing code is not an implicit exception. Never add:

- Destiny 2 process memory reads/writes or direct process access/inspection;
- DLL/code injection, hooks into `destiny2.exe`, DirectX/game/RTSS-style overlay hooks;
- input hooks, gameplay/input automation or game-file edits;
- anti-cheat bypass/manipulation or other risky game-state inspection;
- an always-running service, scheduled task or persistent helper/runtime dependency.

Binding lifecycle: main X hides to system tray while tracking/overlay continue; – minimizes to taskbar; tray double-click restores/focuses; auxiliary X closes that window only; tray Exit fully stops all app work/resources. Pause unnecessary hidden UI updates. Intentional tray operation means running; complete exit means no remaining Threepole-owned work. This target still requires separate implementation approval.

**Unresolved gaps:** overlay visibility opens the foreground process to identify `destiny2.exe`; current X destroys the UI rather than hiding it, tray activation uses single click, and orderly shutdown is not centrally coordinated. PROJECT_CONTEXT distinguishes current process exit from the unimplemented lifecycle target. ROADMAP makes safety/lifecycle the first gate. Do not claim full compliance or absolute account safety; explain unsupported features and safe alternatives. Documentation does not authorize runtime changes.

## Priorities and invariants

- Improve startup, activity/orbit/repeated-run detection and clear-to-overlay latency beyond original where local code permits; measure rather than assume. Minimize requests and unnecessary work; prefer simple, focused changes.
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
- For each change assess CPU/spikes, RAM/leaks, network/API volume, task/timer/render cost, overlay reliability and all existing Custom settings. Compare with original under matching conditions where possible, including long sessions and repeated Raid/Dungeon restarts; explicitly report missing Windows/Destiny, FPS/frametime and microstutter evidence.
- The user gives ordinary-language tasks. Handle implementation, tests and identifiable Windows MSI delivery without requiring them to program. Keep unrelated changes separate. Do not install additional programs/services without authorization.
