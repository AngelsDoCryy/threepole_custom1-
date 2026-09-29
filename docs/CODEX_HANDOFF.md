# Threepole Custom — Codex handoff

This document carries forward the durable project context from prior ChatGPT and Work sessions so future Codex work does not depend on chat history.

## 1. Project snapshot

- Repository: `AngelsDoCryy/threepole_custom1-`
- Original upstream: `des-sh/threepole`
- Custom project base: original Threepole 1.1.2
- Current custom branch: `optimize-speed-cache-icon`
- Current draft PR: #1, `Improve custom overlay tracking and orbit responsiveness`
- Current custom version: 1.1.4
- 1.1.4 implementation head before this handoff documentation: `8285cc406f496e02911d469eb74c4372147ad867`
- Main development goal: preserve the simplicity, startup speed and responsiveness of original Threepole while adding optional overlay/tracking features.

## 2. User goals and non-negotiable constraints

The user primarily uses Threepole for Destiny 2 speedruns and wants the overlay to be low-overhead, fast to update, and safe to use.

Hard constraints:

- Bungie API / manifest data only for the custom logic.
- No Destiny 2 process injection.
- No DLL/code injection.
- No memory reading/writing.
- No direct `destiny2.exe` access or hooks.
- No input hooks, gameplay automation, or Destiny file edits.
- Do not introduce RTSS-style hooking.
- No new persistent service or unrelated background process.
- When the app exits, custom polling/tasks must stop.
- Keep Bungie request volume conservative and measurable.
- Prefer separate branch + PR work rather than direct edits to `main`.

Performance target:

- Custom Threepole should feel as fast as original Threepole at startup and during activity/orbit transitions.
- Avoid extra manifest/network work in hot paths when the same data can be cached locally.
- Timer milliseconds and UI-only custom features should not cause material performance degradation.

## 3. Custom functionality added/planned

### Activity name above timer

Implemented intent:

- Show the current Raid/Dungeon/activity name above the timer.
- Update automatically when the activity changes.
- Feature can be enabled/disabled independently.
- Activity name can auto-hide after a configurable delay.
- Auto-hide itself can be enabled/disabled.

Initial preferred hide delay was 10 seconds, but it must remain configurable rather than hard-coded as a permanent preference.

### Per-activity daily clears

Implemented intent:

- Daily clear counts are tracked separately per activity rather than as one global total.
- Switching activity changes the displayed count to that activity's count.
- Returning to a previously played activity resumes that activity's count.
- Preserve the normal daily-reset behavior.
- Use the original Threepole calendar/clear icon beside the count.

### Raid/Dungeon classification

Implemented intent:

- Support activities that do not expose the normal activity-mode information cleanly.
- Use a small local activity-hash -> mode cache.
- Dynamically classify previously unseen Raid/Dungeon activities from Bungie manifest data.
- Do not perform a new manifest request for every history entry.
- Unknown/future activities should degrade gracefully rather than breaking the overlay.

### Independent settings

The custom features should remain individually controllable. In particular, do not couple:

- activity-name visibility,
- activity-name auto-hide,
- clear counter,
- clear notifications,
- timer visibility,
- timer milliseconds.

### Icons

Current code/docs on 1.1.4 still describe a custom Eager Edge icon for app/taskbar/tray/installer, while the original calendar icon is used for the clear counter.

Latest user preference from the prior work is to return the application/Windows/tray/installer icon to the standard Threepole icon. Treat that as an open requested change; do not assume it is already complete.

### Updater

The original automatic updater is disabled in the custom build so an official upstream release cannot overwrite the custom installation.

## 4. Current activity timing architecture — 1.1.4

The main regression being worked on was that the custom build could remain on the previous activity or stop its timer later than original Threepole after returning to orbit.

The 1.1.4 design intentionally restores separation between identity/status and optional timing data:

### CharacterActivities / component 204

- Requested independently.
- Uses the existing two-second pause/cadence from the original current-activity path.
- This is the authoritative path for activity identity / observed orbit state.
- A slow optional timing request must never block this status request.

### Transitory / component 1000

- Fetched independently from component 204.
- Ten-second pause/cadence.
- Adds at most six requests per minute in steady state.
- Used only as an optional timing source.
- It does not contain an activity hash, so it cannot independently prove activity identity or orbit.
- Missing/private/malformed/future/older data must fall back safely to CharacterActivities timing.

### Freshness / state rules

- Compare Bungie's primary and secondary generation timestamps independently.
- Reject genuinely older primary snapshots.
- Do not reject a newer status solely because `dateActivityStarted` moved backwards; that field is an activity start time, not a status-generation timestamp.
- A fresh orbit snapshot may legitimately carry an older or epoch activity-start value.
- A fresh different activity may also have an earlier start time than the previous observation.
- Confirmed orbit clears the previous accepted Transitory start so timing cannot leak across runs.

### Clear-history independence

Activity clear notifications are derived from history and may arrive before the live current-activity status catches up.

Required invariant:

- A clear result must not itself force orbit.
- A clear result must not stop a later run that has already started.
- History processing and live current-activity tracking remain separate state paths.

## 5. HTTP / Bungie handling

Earlier optimization work added/reinforced:

- one shared HTTP client for the app session,
- preservation of Bungie affinity/cookie behavior between requests,
- no extra request volume from affinity handling itself.

Do not attribute every timing delay to affinity/backend routing without evidence. Bungie controls when API data becomes fresh; local code can remove blocking/rejection mistakes but cannot guarantee instant API updates.

## 6. Performance work already considered

The project previously focused on reducing custom overhead relative to original Threepole:

- local activity-mode caching to avoid repeated manifest requests,
- keeping activity status polling on the original two-second path,
- separating optional timing so it cannot hold up identity/orbit,
- shared HTTP client,
- overlay visibility/recovery regression work,
- avoiding new runtime dependencies/services.

The user explicitly reported that original Threepole appeared to load/update faster than the custom build, so regressions in startup, orbit detection or restart detection should be treated as high-priority performance bugs.

## 7. Known observations and validation gaps

### Observation A — orbit responsiveness

User live comparison on an earlier custom build:

- original Threepole recognized the return to orbit sooner,
- custom Threepole kept the previous activity/timer longer,
- custom clear notification could appear before the overlay timer/state visibly updated.

1.1.4 was created specifically to remove local coupling and stale-status rejection paths associated with those symptoms.

### Observation B — same-Dungeon restart

On an older build, the user observed:

- first Dungeon run timer/activity recognition: roughly 30 seconds,
- immediate restart of the same Dungeon: roughly 2–3 minutes before recognition.

The user explicitly noted that this test was still on the older build and had not yet tested 1.1.4. Therefore:

- do not treat this issue as confirmed fixed,
- do not treat it as confirmed present in 1.1.4 either,
- the next meaningful validation is a live side-by-side/repeated-run test on 1.1.4 or later.

### Observation C — history vs live overlay ordering

A clear notification may legitimately arrive before current-activity status changes. The bug to avoid is not the ordering itself; the bug would be allowing history state to terminate or corrupt a later active run.

## 8. Current regression coverage / build state

The 1.1.4 PR reported:

- local frontend build and TypeScript validation passed,
- five overlay regression tests passed,
- 37 Rust tests passed in the Windows build,
- MSI build succeeded.

The branch workflow currently runs:

1. `yarn install`
2. `node --test tests/overlay-visibility.test.mjs tests/overlay-activity.test.mjs`
3. `yarn tauri build`
4. `cargo test --release --manifest-path src-tauri/Cargo.toml --bin app`
5. upload the MSI artifact

For code changes, also run `yarn build` locally/where available before claiming frontend completion.

Tests added around 1.1.4 cover cases including:

- fresh orbit with backwards activity-start time,
- epoch-style orbit start time,
- switching to a different activity whose start time is earlier,
- transitions with missing optional timing,
- orbit and re-entry,
- standalone Transitory parsing,
- clear events arriving before orbit or after another run has begun.

## 9. Development workflow for future Codex sessions

Before changing code:

1. Read `AGENTS.md` and this file.
2. Inspect the current PR/branch rather than assuming `main` contains the custom implementation.
3. Identify whether the issue is frontend overlay state, Rust/Bungie polling, history processing, packaging, or only a live API freshness limitation.
4. Preserve the safety constraints.

For a bug fix:

1. Reproduce logically from code/tests where possible.
2. Add a regression test first or alongside the fix if practical.
3. Make the smallest change that fixes the state/timing issue.
4. Run the relevant frontend/Rust tests.
5. Run/build Windows MSI when runtime/package behavior is affected.
6. Update README/version notes only when behavior actually changes.
7. Summarize what was proven by tests versus what still needs live Destiny/Bungie validation.

For GitHub work:

- Prefer continuing the existing feature branch/PR until its work is resolved.
- Do not silently merge PR #1 into `main`.
- Keep commits focused and describe timing/API changes precisely.

## 10. Next recommended work sequence

Unless the user gives a newer priority, the logical continuation is:

1. Live-test 1.1.4 against original Threepole for orbit recognition and timer-stop latency.
2. Test repeated restart of the same Dungeon several times.
3. Record whether delay is in activity identity, activity start timestamp, optional Transitory timing, overlay rendering, or history/notification ordering.
4. Only then change polling/state logic if 1.1.4 still shows a local regression.
5. Separately revert the Eager Edge application icon to the standard Threepole icon per the latest user preference, without changing the original calendar clear icon.

Do not increase polling frequency as the first response to an unverified delay. First determine which response/state path is actually late.
