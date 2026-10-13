# Threepole Custom — Project Context

This file owns the durable user decisions and workflow. Last alignment: 2026-10-13; implementation remains 1.1.4. Implemented behavior, unresolved conflicts and required future behavior must not be conflated.

## Project identity

- Repository: `AngelsDoCryy/threepole_custom1-`
- Upstream project: `des-sh/threepole`
- Custom project base: original Threepole 1.1.2
- Current development branch: `optimize-speed-cache-icon`
- Current draft PR: #1 — `Improve custom overlay tracking and orbit responsiveness`
- Current custom version documented by the active work: 1.1.4

The permanent goal is to make Threepole Custom **measurably faster, more reliable and richer in useful features than original Threepole**, with preferably equal or lower CPU, RAM and network use and without additional or riskier interaction with Destiny 2. Original Threepole 1.1.2 is the permanent technical comparison baseline and minimum standard, not the final performance goal. Earlier “close to original” or parity-only targets are superseded; a measured advantage is not yet established.

## Core priorities

In order of importance:

1. Safety and Destiny account protection.
2. Reliability and correct function.
3. Measurably higher speed than original Threepole.
4. Preferably equal or lower resource consumption.
5. Additional convenience features.

Speed and resource use must improve together: use efficient architecture, bounded parallel work, caching and removal of local delays rather than substantially higher system load or excessive requests. A small resource increase may be acceptable only for a relevant, measurable benefit without compromising safety or game performance; explain the tradeoff and obtain the user's decision beforehand. Significantly higher resource use is not an acceptable acceleration strategy. Maintainability and preservation of existing independent features apply throughout.

## Non-negotiable safety constraints

Activity data must come exclusively from official Bungie APIs and provided web/manifest data. The overlay remains an independent Windows window. Ordinary Windows queries for window focus, title/class or geometry are allowed when they do not access the Destiny process directly. The custom project must not add:

- Destiny 2 process injection
- DLL or code injection
- process memory reading or writing
- direct `destiny2.exe` access or hooks
- DirectX/game-overlay injection comparable to RTSS-style hooking
- input hooks or gameplay automation
- edits to Destiny 2 game files
- anti-cheat bypass or manipulation, or other risky game-state detection
- unrelated persistent services or background processes

When Threepole fully exits, all its polling/tasks and resources must stop. Intentional tray operation is a running state as defined below. Bungie API request volume should remain conservative and measurable. Speed never justifies riskier game interaction. No absolute account-safety guarantee is technically possible; document the remaining limits and offer a safe alternative if a feature cannot meet this boundary.

### Current safety and lifecycle findings — not approved exceptions

`src-tauri/src/pollers/overlay.rs::get_hwnd_exec` uses `OpenProcess(PROCESS_QUERY_INFORMATION)` and `K32GetModuleFileNameExW` to compare the foreground executable name with `destiny2.exe`. This predates the current PR's fixes; the PR closes the acquired handle. It is not an activity-memory reader, injection or input hook, but it **is direct process access**, so the current implementation cannot be described as fully meeting the boundary above. Review and replace the mechanism within the boundary; do not weaken the requirement by changing its wording.

`src/window/window.ts` currently calls `close()` for every X and `minimize()` for –. `main.rs` prevents ordinary application exit; the tray uses **single** left-click activation. Existing windows are unminimized/focused without an explicit show call. The primary window is `details`, or `profiles` before profile selection; preferences and normally profile management are auxiliary. X destroys that webview while player-data/overlay work continues; it does not yet implement the agreed hide/restore contract. Disabling the overlay closes its window and aborts its poller, not player-data polling.

Tray Exit calls `AppHandle::exit(0)`. In the pinned [Tauri 1.2.4 implementation](https://github.com/tauri-apps/tauri/blob/tauri-v1.2.4/core/tauri/src/app.rs), this performs framework cleanup then `std::process::exit`; the ordinary `ExitRequested` prevention does not block that direct exit. There is no separate polling service in the inspected source and no evidence that Rust pollers survive successful process termination. What is missing is coordinated cancellation/cleanup and Windows evidence for all app-owned resources, including WebView2 helpers; do not misreport that gap as a demonstrated post-exit polling leak.

Player-data reset and overlay replacement abort stored handles without awaiting completion. Startup, tray activation and named-pipe tasks are spawned without retained handles; one-off profile/search/metadata IPC requests are outside the player-data task. There is no central stopping state preventing new work. These are the lifecycle coverage gaps addressed by ROADMAP, not an instruction to add a background supervisor.

### Binding window and tray behavior — decided, not implemented

The user's explicit decision supersedes the earlier unresolved close-to-tray wording:

| Action | Required behavior |
| --- | --- |
| X in main window | Hide into Windows system tray, without a taskbar window; tracking and overlay continue |
| – in main window | Normal minimize into the Windows taskbar |
| Tray double-click | Show, restore and focus the main window |
| X in settings/profile management | Close only that auxiliary window |
| Tray → Exit / Beenden | Fully end the app, polling, in-flight HTTP work, timers, windows and all other app-owned resources |

Tray operation is intentionally running, not “closed.” Pause unnecessary rendering in hidden/minimized UI while preserving tracking and overlay behavior. Add no unnecessary process, service, autostart, timer or poller. Window roles must account for the initial profile screen; do not silently treat a startup primary window as an auxiliary one. The complete behavior requires a separate implementation order and Windows acceptance tests.

### Icon decision

As confirmed on 2026-10-11, **all Threepole Custom icons must use Eager Edge**, including app/taskbar/tray/installer, notifications and the overlay-clear icon. Older requests to restore the original Threepole icon are superseded. The calendar SVG still present beside the counter is an implementation gap, not the desired design. See CUSTOM_FEATURES for asset mapping.

## Working model

### GitHub

GitHub is the source of truth for code, branches, pull requests, tests, releases and durable project documentation.

### Codex and the user

Codex is now the central workspace for development, debugging, tests, performance review, API questions and documentation. The user gives ordinary-language tasks and is not expected to program or maintain the toolchain manually. Codex prepares focused changes, validates them, explains risks/request-volume changes and supplies Windows MSI builds tied to a commit for the user's practical checks. Prior ChatGPT/Work decisions are retained in the existing knowledge base; neither application has to remain running as a project dependency.

Start sessions inside the correct repository checkout and follow root AGENTS.md. It explicitly routes future sessions to the existing knowledge files. Those linked documents still need to be read; automatic instruction discovery does not automatically include all of `docs/`. Keep AGENTS concise, preserve its location and inspect any applicable override before work. No extra global configuration or duplicate instruction tree is required. [Official instruction-discovery documentation](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Maintain a single source for each kind of detail using the map below. Preserve observations and their evidence level; do not turn a roadmap proposal into an implemented feature. The 2026-10-13 order authorizes necessary existing-document changes and a commit/push on the active branch only. It does not authorize implementation of work package 1, dependency/tool installation, CI changes, merge or removal of Draft status. Future implementation requires its own user order; these durable rules guide that work rather than authorizing it automatically.

## Knowledge-base map

- `docs/PROJECT_CONTEXT.md` — this file; project identity, constraints and workflow
- `docs/CUSTOM_FEATURES.md` — custom feature behavior and state
- `docs/DESTINY_API.md` — Bungie API/manifest architecture and data rules
- `docs/PERFORMANCE.md` — performance targets, known regressions and validation plan
- `docs/ROADMAP.md` — open work and priorities
- `docs/CODEX_HANDOFF.md` — detailed handoff from prior ChatGPT/Work development sessions

## “Threepole Status” convention

When the user asks for **“Threepole Status”**, treat it as a project-wide status request rather than a weekly report.

The status should combine, where accessible:

- current GitHub branch/PR/commit state
- recent code or documentation changes
- prior ChatGPT and Work decisions
- Codex progress
- current custom Threepole behavior
- relevant Destiny/Bungie API findings

The response should cover:

1. Last changes
2. Current state
3. Open issues / validation gaps
4. Safety or API risks
5. Performance risks
6. Possible improvements
7. Next priorities

Do not limit the status to the previous week unless the user explicitly asks for a weekly view.

## Development rule

Prefer work on a separate branch and pull request rather than direct edits to `main`. Do not silently merge the active PR.

Before changing timing or polling behavior, distinguish between:

- local Threepole state-management delay,
- UI/overlay delay,
- Bungie API freshness delay,
- history/notification timing,
- optional timing-source behavior.

Do not increase polling frequency as the default response to an unverified delay.

For every development change: inspect the current code, choose a simple maintainable solution, assess correctness/speed and resource/API impact across the full API → retrieval → processing → state → overlay chain, preserve Custom features/settings, add suitable regression coverage, check related components and update the owning knowledge files. Compare with original where feasible using PERFORMANCE's method; explicitly label technical expectations and missing Windows/Destiny evidence. Reassess existing features with known defects without presenting them as complete or permitting accidental removal.
