# Threepole Custom 1.1.2 (Custom.3)

Based on Threepole 1.1.2.

## Added
- Activity name above the timer.
- Activity name can be enabled/disabled independently.
- Activity name can auto-hide.
- Auto-hide delay is configurable in seconds (0-3600).
- The activity name is shown again for every newly started activity, even when repeating the same raid/dungeon.
- Activity-specific daily clear counter: completed clears are counted for the current activity hash.
- Generic Raid/Dungeon type icon beside the clear counter, obtained from Bungie's manifest API.
- Raid/dungeon classification falls back to Bungie's activityTypeHash when activity mode data is missing.
- History classification also falls back to the Bungie manifest if an activity history entry has incomplete mode data.
- This covers newer activities such as Sundered Doctrine without reading Destiny memory.
- Official Threepole updater is disabled in the custom build.
- Custom build has its own Windows app identifier, config directory, and named pipe so it does not overwrite the official Threepole installation/config.
- Custom Eager Edge app/tray/installer icon.

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
