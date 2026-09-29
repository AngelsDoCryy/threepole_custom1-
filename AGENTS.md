# Codex instructions for Threepole Custom

Read `docs/CODEX_HANDOFF.md` before making substantive changes. Treat it as the project handoff and durable context from prior ChatGPT/Work sessions.

## Repository and active work

- Repository: `AngelsDoCryy/threepole_custom1-`
- Do not work directly on `main` unless the user explicitly asks for that.
- The current custom development branch is `optimize-speed-cache-icon` and draft PR #1 targets `main`.
- Current custom version on that branch: 1.1.4, based on original Threepole 1.1.2.
- Preserve original Threepole behavior and performance unless a custom requirement intentionally changes it.

## Hard safety constraints

The app must remain API/manifest based and non-invasive toward Destiny 2.

Never add:
- Destiny 2 process memory reads or writes
- DLL/code injection
- hooks into `destiny2.exe`
- input hooks or gameplay/input automation
- edits to Destiny 2 game files
- RTSS-style/game-process overlay hooking
- a new always-running background service or runtime dependency

When Threepole is closed, custom code must not continue polling or performing work.

## Product priorities

1. Activity/orbit/restart detection should be at least as responsive as the original Threepole wherever local code can control it.
2. Minimize startup/loading overhead and keep the custom build close to original Threepole performance.
3. Keep Bungie API traffic conservative and explain any added requests.
4. Keep custom settings independent so features can be disabled separately.
5. Prefer simple, maintainable changes over invasive optimizations.

## Current behavior that must be preserved

- CharacterActivities component 204 is polled independently on the existing two-second path.
- Optional Transitory component 1000 is fetched independently on a ten-second cadence; failure/latency there must not block activity identity or orbit status.
- Activity-history clear notifications are independent from live activity state. A clear must not force orbit or stop a later run.
- Confirmed orbit clears stale optional timing from the previous run.
- Per-activity daily clears are preserved and resume when returning to the same activity.
- Activity name display and configurable auto-hide remain optional/independent.
- Raid/Dungeon classification uses the local cache plus Bungie manifest data without adding per-history-item manifest requests.
- The original Threepole calendar/clear icon is used beside the clear counter.
- The automatic updater remains disabled so official releases cannot overwrite the custom build.

## Known open validation points

- Live-test 1.1.4 against original Threepole for orbit detection and timer stop latency.
- Re-test rapid restart of the same Dungeon. On an older build, the first run started in about 30 seconds but a restart sometimes took 2–3 minutes; 1.1.4 was not yet validated for that report.
- Verify ordering when a clear notification arrives before the overlay/timer reflects orbit or the next activity.
- Latest user preference is to return the Windows/app/tray/installer icon to the standard Threepole icon; the current 1.1.4 branch still documents Eager Edge icons, so treat this as an open requested change rather than an already-completed change.

## Validation before proposing a completed change

At minimum, run or account for:
- `yarn install`
- `yarn build`
- `node --test tests/overlay-visibility.test.mjs tests/overlay-activity.test.mjs`
- `cargo test --release --manifest-path src-tauri/Cargo.toml --bin app`
- Windows/Tauri MSI build (`yarn tauri build`) when the change can affect packaging or runtime behavior.

The repository GitHub Actions workflow on `optimize-speed-cache-icon` is the reference Windows build path. Do not claim a live Destiny/Bungie timing issue is fixed solely from unit tests; clearly separate local-code validation from in-game/API validation.

## Change discipline

- Inspect the existing implementation before editing; do not re-implement already-solved behavior from scratch.
- Add regression tests for timing/state-machine bugs when feasible.
- Keep README/version notes aligned with behavior actually shipped.
- State API request-rate changes explicitly.
- Avoid speculative claims about Bungie backend behavior; distinguish observed app behavior, code-level causes, and unverified API-side causes.
