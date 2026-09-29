# Threepole Custom — Project Context

This file is the durable high-level context for the custom Threepole project.

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

## Working model

### GitHub

GitHub is the source of truth for code, branches, pull requests, tests, releases and durable project documentation.

### Codex

Use Codex as the primary workspace for substantial engineering work:

- implementation
- debugging
- refactoring
- test creation
- performance work
- repository-wide analysis
- code review

Codex should read the knowledge-base files before making project-wide changes.

### Normal ChatGPT chat

Use normal chat for lightweight questions and fast analysis, for example:

- Bungie API behavior
- Threepole architecture questions
- safety checks
- interpreting logs/test results
- deciding what to test next
- explaining code or behavior

### Work

Use Work when a larger task benefits from combining multiple sources, documents, repository context or longer research/analysis.

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