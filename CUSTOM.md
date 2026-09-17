# Threepole Custom 1.1.3

Based on Threepole 1.1.2.

## 1.1.3: Current activity timing
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
- Original Threepole calendar icon beside the clear counter.
- Raid/dungeon classification falls back to Bungie's activityTypeHash when activity mode data is missing.
- History classification uses cached activity modes if an activity history entry has incomplete mode data, without extra manifest requests per history entry.
- This covers newer activities such as Sundered Doctrine without reading Destiny memory.
- Official Threepole updater is disabled in the custom build.
- Custom build has its own Windows app identifier, config directory, and named pipe so it does not overwrite the official Threepole installation/config.
- Custom Eager Edge app/tray/installer icon.
- Eager Edge icon for startup and clear notifications.
- Independent Display timer option in the separate Custom features settings group.

## Safety model
The custom code uses Bungie API/manifest data plus Threepole's own UI/configuration.
No Destiny memory read/write, injection, DLL loading into Destiny, Destiny file modification, or input automation was added.

## Easy Windows build with GitHub Actions
A manual workflow is included at `.github/workflows/build-windows.yml`.

1. Put this source in a GitHub repository.
2. In repository Settings -> Secrets and variables -> Actions, create a repository secret named `BUNGIE_API_KEY`.
3. Open Actions -> `Build Threepole Custom for Windows` -> Run workflow.
4. When the run finishes, download the `threepole-custom-windows-msi` artifact.
5. Extract the artifact ZIP and run the MSI.

Important: the Bungie API key is compiled into the resulting binary by the original Threepole architecture. Keep the repository/build artifact private if you do not want to distribute that key.

## Local Windows build
Requirements: Node.js, Yarn, Rust, Microsoft C++ Build Tools / Windows SDK, WebView2 Runtime, and `BUNGIE_API_KEY` in the environment.

Run:

    .\build-windows.ps1

The MSI is normally created under `src-tauri\target\release\bundle\msi\`.
