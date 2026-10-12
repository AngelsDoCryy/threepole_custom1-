# Threepole Custom 1.1.4

Based on Threepole 1.1.2.

Documentation alignment: 2026-10-11. Sections below describe shipped history; current requirements and unresolved gaps are in `docs/`. This cleanup changes no implementation or version.

## 1.1.4: Orbit and activity status responsiveness
- Restores the standalone CharacterActivities (204) request with the existing two-second pause. Optional Transitory (1000) is fetched independently, immediately after initial data loads and then with a ten-second pause (at most six extra requests per minute in steady state).
- A slow or failed Transitory request cannot hold up the activity-name, restart, or orbit status path. All polling remains in the existing cancellable app task and shares the existing HTTP client.
- A newer primary snapshot can report orbit or a different activity even if its activity start time moves backwards. Older primary snapshots remain rejected; missing freshness metadata retains the conservative fallback.
- Confirmed orbit discards the previous run's optional start time. Transitory alone still cannot identify a new activity or prove that the player has entered orbit.
- Clear notifications remain independent: a completed result may arrive before live status changes, and does not force orbit or stop a subsequent run.
- Adds Rust regressions for backward/epoch orbit timestamps, different activities, missing optional timing and reentry, plus overlay tests for clears arriving before orbit or after the next run starts.
- This addresses local blocking and rejection paths. The exact cause of a live comparison cannot be proved without API captures, and Bungie's own update delay remains.

## 1.1.3: Current activity timing (historical; superseded by 1.1.4)
- Queries components 204 and 1000 in the same profile request, with unchanged polling intervals.
- Uses separate Bungie freshness timestamps and accepts newer Transitory start times only when the character snapshot was generated at or after that start. This reduces mismatched snapshots; Transitory alone cannot confirm activity identity.
- Retains the character-based fallback for missing or invalid optional data, and prevents stale responses from rolling back an accepted timer.
- Includes regression tests for repeated dungeon runs, orbit, delayed identity updates, missing components, and out-of-order snapshots.
- No new background service, game access, input hooks, or runtime dependency is added. Actual switching latency depends on Bungie's data updates.

## Added
- Activity name above the timer.
- Activity name can be enabled/disabled independently.
- Activity name can auto-hide.
- Auto-hide delay is configurable in seconds (0-3600).
- The activity name is shown again for every newly started activity, even when repeating the same raid/dungeon.
- Activity-specific daily clear counter: completed clears are counted for the current activity hash.
- A calendar SVG remains beside the clear counter in 1.1.4. Current requirement: replace it with Eager Edge; implementation is pending.
- Raid/dungeon classification falls back to Bungie's activityTypeHash when activity mode data is missing.
- History classification uses cached activity modes if an activity history entry has incomplete mode data, without extra manifest requests per history entry.
- This supports newer activities such as Sundered Doctrine through available manifest classification, without reading Destiny memory. Incomplete history for an unknown uncached hash can still be omitted.
- Official Threepole updater is disabled in the custom build.
- Custom build has its own Windows app identifier, config directory, and named pipe so it does not overwrite the official Threepole installation/config.
- Custom Eager Edge app/tray/installer icon.
- Eager Edge icon for startup and clear notifications.
- Independent Display timer option in the separate Custom features settings group.

## Safety model
Required: API-/web-/manifest-only, no direct Destiny process access, memory read/write, injection, game/input hooks, gameplay automation, game-file edits or independent background service. No work while Threepole is closed.

Current activity data uses Bungie API/manifest sources. However, inherited overlay visibility uses `OpenProcess` and executable-name inspection, and window close currently leaves the app/polling running in the tray until Exit. These are unresolved conflicts, not approved exceptions. No gameplay-memory reader or injection was found. Details and the safety gate are in [PROJECT_CONTEXT](docs/PROJECT_CONTEXT.md) and [ROADMAP](docs/ROADMAP.md).

## Current gaps and validation

- Eager Edge is required for all Threepole Custom icons; app/taskbar/tray/installer and popups already use its assets, while overlay-clear replacement is pending. Older original-icon requests are superseded.
- Initial history still blocks first data publication. Manifest errors can prevent activity publication; cached empty-name definitions can prevent recovery.
- Corrected same-instance completion and late older clears can be missed by notification logic even when history/counts update.
- Independent steady-state polling and the existing tests do not prove that the earlier orbit/repeated-Dungeon delays are fixed live.
- [Windows build #19](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/36518496000) passed 37 Rust tests, five overlay tests and MSI packaging on the pre-cleanup documentation head. Current CI has no standalone TypeScript/Svelte type-check step. [CODEX_HANDOFF](docs/CODEX_HANDOFF.md) records the evidence; [PERFORMANCE](docs/PERFORMANCE.md) retains optimizations and live checks.

## Windows builds and local environment

Use the existing [BUILD-GITHUB-DE](BUILD-GITHUB-DE.md) guide for the workflow, local requirement tiers and API-key handling. The repository is currently public; do not assume build artifacts keep the compiled-in key secret. The existing local build script can install Yarn globally, so it is not a read-only environment check. No installation or build is part of the documentation cleanup.
