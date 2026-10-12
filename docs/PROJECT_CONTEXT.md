# Threepole Custom — Project Context

This file owns the durable user decisions and workflow. Last alignment: 2026-10-11; implementation remains 1.1.4. Implemented behavior, unresolved conflicts and required future behavior must not be conflated.

## Project identity

- Repository: `AngelsDoCryy/threepole_custom1-`
- Upstream project: `des-sh/threepole`
- Custom project base: original Threepole 1.1.2
- Current development branch: `optimize-speed-cache-icon`
- Current draft PR: #1 — `Improve custom overlay tracking and orbit responsiveness`
- Current custom version documented by the active work: 1.1.4

The project exists to keep the original Threepole experience lightweight and responsive while adding optional Destiny 2 overlay and tracking features useful for speedruns.

## Core priorities

In order of importance:

1. Destiny/Bungie safety: custom logic must remain API-/web-/manifest-only.
2. Responsiveness: startup, activity changes, orbit detection and timer state should feel at least as fast as original Threepole where local code controls the result.
3. Low overhead: avoid unnecessary network, manifest, CPU, background or UI work.
4. Optional features: custom functionality should be independently configurable instead of tightly coupled.
5. Maintainability: future ChatGPT, Work and Codex sessions should be able to understand the current architecture without relying on old chat history.

## Non-negotiable safety constraints

The custom project must not add:

- Destiny 2 process injection
- DLL or code injection
- process memory reading or writing
- direct `destiny2.exe` access or hooks
- DirectX/game-overlay injection comparable to RTSS-style hooking
- input hooks or gameplay automation
- edits to Destiny 2 game files
- unrelated persistent services or background processes

When Threepole exits, custom polling/tasks must stop. Bungie API request volume should remain conservative and measurable.

### Current implementation conflicts — not approved exceptions

`src-tauri/src/pollers/overlay.rs::get_hwnd_exec` uses `OpenProcess(PROCESS_QUERY_INFORMATION)` and `K32GetModuleFileNameExW` to compare the foreground executable name with `destiny2.exe`. This predates the current PR's fixes; the PR closes the acquired handle. It is not an activity-memory reader, injection or input hook, but it **is direct process access**, so the current implementation cannot be described as fully meeting the boundary above. Review and replace the mechanism within the boundary; do not weaken the requirement by changing its wording.

`src/window/window.ts` closes the UI window, while `src-tauri/src/main.rs` prevents ordinary application exit and exposes tray **Exit**. Thus window close currently leaves polling active in the tray. Actual Exit ends the app; there is no separate polling service in the inspected source. Runtime verification that requests and app-owned processes cease after Exit is still pending. Disabling the overlay also does not disable player-data polling.

The user's requirement is no work while Threepole is closed. Window-close-to-tray must not be silently accepted as satisfying that requirement. ROADMAP requires explicit product semantics: application close should stop app work; any retained tray operation must be visibly an intentional running state, not an invisible substitute for closing. The existing behavior is documented, not authorized as an exception.

### Icon decision

As confirmed on 2026-10-11, **all Threepole Custom icons must use Eager Edge**, including app/taskbar/tray/installer, notifications and the overlay-clear icon. Older requests to restore the original Threepole icon are superseded. The calendar SVG still present beside the counter is an implementation gap, not the desired design. See CUSTOM_FEATURES for asset mapping.

## Working model

### GitHub

GitHub is the source of truth for code, branches, pull requests, tests, releases and durable project documentation.

### Codex and the user

Codex is now the central workspace for development, debugging, tests, performance review, API questions and documentation. The user gives ordinary-language tasks and is not expected to program or maintain the toolchain manually. Codex prepares focused changes, validates them, explains risks/request-volume changes and supplies Windows MSI builds tied to a commit for the user's practical checks. Prior ChatGPT/Work decisions are retained in the existing knowledge base; neither application has to remain running as a project dependency.

Start sessions inside the correct repository checkout and follow root AGENTS.md. It explicitly routes future sessions to the existing knowledge files. Those linked documents still need to be read; automatic instruction discovery does not automatically include all of `docs/`. Keep AGENTS concise, preserve its location and inspect any applicable override before work. No extra global configuration or duplicate instruction tree is required. [Official instruction-discovery documentation](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Maintain a single source for each kind of detail using the map below. Preserve observations and their evidence level; do not turn a roadmap proposal into an implemented feature. The 2026-10-11 cleanup is documentation-only. Environment inspection is allowed, installation of additional programs/background services is not part of that assignment.

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
