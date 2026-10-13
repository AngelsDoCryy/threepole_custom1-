# Threepole Custom — Project Context

This file owns the durable user decisions and workflow. Last alignment: 2026-10-14; package 1 is implemented on the development branch with version metadata still 1.1.4. Implemented behavior, unresolved conflicts and required future behavior must not be conflated.

## Project identity

- Repository: `AngelsDoCryy/threepole_custom1-`
- Upstream project and permanent technical reference: [des-sh/threepole](https://github.com/des-sh/threepole)
- Custom project base: original Threepole 1.1.2
- Current development branch: `optimize-speed-cache-icon`
- Current draft PR: #1 — `Improve custom overlay tracking and orbit responsiveness`
- Current custom version documented by the active work: 1.1.4

The permanent goal is to make Threepole Custom **measurably faster, more reliable and richer in useful features than original Threepole**, with preferably equal or lower CPU, RAM and network use and without additional or riskier interaction with Destiny 2. Original Threepole 1.1.2 is the permanent technical comparison baseline and minimum standard, not the final performance goal. Earlier “close to original” or parity-only targets are superseded; a measured advantage is not yet established.

For future optimizations inspect the relevant code paths in the original repository alongside Custom, not only benchmark results. Record the compared upstream commit/version; distinguish the 1.1.2 base from any newer upstream revision instead of silently changing the reference. Explain which differences are required by Custom features and which add avoidable delay or resource cost. Original code is a reference, not an exception to our safety rules.

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

The inherited foreground executable lookup was a confirmed direct-process-access conflict, despite the earlier handle-leak fix. Separately authorized package 1 now removes `OpenProcess`, `K32GetModuleFileNameExW`, the PID query and the unbounded HWND/name cache. `pollers/overlay.rs` identifies the foreground window by freshly read title/class, visibility and valid geometry only; failure hides the overlay. No process-based fallback is permitted. PERFORMANCE owns the observed window metadata, source comparison and remaining Windows acceptance tests. This resolves that source-level conflict, not every live safety/reliability question or the pending full-exit verification.

`src/window/window.ts` currently calls `close()` for every X and `minimize()` for –. `main.rs` prevents ordinary application exit; the tray uses **single** left-click activation. Existing windows are unminimized/focused; closed windows are recreated. The primary window is `details`, or `profiles` before profile selection; preferences and normally profile management are auxiliary. X destroys that webview while player-data/overlay work continues. The user now explicitly accepts this behavior; destruction/recreation is not a defect requiring replacement with hide/show. Disabling the overlay closes its window and aborts its poller, not player-data polling.

Tray Exit calls `AppHandle::exit(0)`. In the pinned [Tauri 1.2.4 implementation](https://github.com/tauri-apps/tauri/blob/tauri-v1.2.4/core/tauri/src/app.rs), this performs framework cleanup then `std::process::exit`; the ordinary `ExitRequested` prevention does not block that direct exit. There is no separate polling service in the inspected source and no evidence that Rust pollers survive successful process termination. Windows evidence for all app-owned resources, including WebView2 helpers, remains pending. The absence of central cancellation/cleanup is not itself a demonstrated exit defect or authorization for an architectural rewrite.

Player-data reset and overlay replacement abort stored handles without awaiting completion. Startup, tray activation and named-pipe tasks are spawned without retained handles; one-off profile/search/metadata IPC requests are outside the player-data task. There is no central stopping state preventing new work. Preserve these findings as test targets, not proof that tasks survive process termination. Propose a minimal correction only if concrete failure is demonstrated; add no background supervisor.

### Binding window and tray behavior — preserve current behavior

The latest user clarification supersedes the previous double-click and retained-hidden-window requirements. The user is satisfied with the existing interaction:

| Action | Required behavior |
| --- | --- |
| X in main window | Visible window disappears; preserve existing close/recreation behavior while tracking and overlay continue in the tray |
| – in main window | Normal minimize into the Windows taskbar |
| Simple tray click | Reopen/activate the main window as today; no mandatory double-click |
| X in settings/profile management | Close only that auxiliary window |
| Tray → Exit / Beenden | Fully end the app, polling, in-flight HTTP work, timers, windows and all other app-owned resources |

Tray operation is intentionally running, not “closed.” Do not rebuild window management or retain an extra hidden webview merely to satisfy the superseded wording. Preserve initial profile setup and auxiliary-window behavior. Review complete Exit for remaining app-owned tasks, HTTP work, timers, windows/helpers and pipe resources; no fresh Windows verification is claimed by this documentation clarification. If a concrete problem is found, explain evidence, the smallest correction and its resource/API/feature effects, then obtain implementation approval. Unnecessary UI work remains an optimization candidate only when justified by measurement. Add no unnecessary process, service, autostart, timer or poller.

### Icon decision

As confirmed on 2026-10-11, **all Threepole Custom icons must use Eager Edge**, including app/taskbar/tray/installer, notifications and the overlay-clear icon. Older requests to restore the original Threepole icon are superseded. The calendar SVG still present beside the counter is an implementation gap, not the desired design. See CUSTOM_FEATURES for asset mapping.

## Working model

### GitHub

GitHub is the source of truth for code, branches, pull requests, tests, releases and durable project documentation.

### Codex and the user

Codex is now the central workspace for development, debugging, tests, performance review, API questions and documentation. The user gives ordinary-language tasks and is not expected to program or maintain the toolchain manually. Codex prepares focused changes, validates them, explains risks/request-volume changes and supplies Windows MSI builds tied to a commit for the user's practical checks. Prior ChatGPT/Work decisions are retained in the existing knowledge base; neither application has to remain running as a project dependency.

Start sessions inside the correct repository checkout and follow root AGENTS.md. It explicitly routes future sessions to the existing knowledge files. Those linked documents still need to be read; automatic instruction discovery does not automatically include all of `docs/`. Keep AGENTS concise, preserve its location and inspect any applicable override before work. No extra global configuration or duplicate instruction tree is required. [Official instruction-discovery documentation](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Maintain a single source for each kind of detail using the map below. Preserve observations and their evidence level; do not turn a roadmap proposal into an implemented feature. The window/tray clarification supersedes the previously planned rebuild. The user subsequently authorized package 1 only: process-free overlay recognition, focused tests, existing-document updates and a CI MSI testbuild. Window/tray, shutdown, Bungie pollers, manifest/history and notification changes remain outside that order. Keep CI unchanged and PR #1 open/Draft; no merge. Future packages require their own user order.

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
