# Threepole Custom — Roadmap

This file tracks current priorities, validation gaps and future work.

## Priority 1 — Validate 1.1.4 live behavior

Compare current custom build against original Threepole in actual Destiny 2 use.

Test:

- first activity recognition
- return to orbit
- timer stop/update timing
- immediate restart of the same Dungeon
- repeated same-Dungeon restarts
- switching to a different activity
- clear-notification timing versus live overlay state

Do not assume the earlier 2–3 minute repeated-run delay is fixed or still present until 1.1.4 or later is tested.

## Priority 2 — Isolate any remaining delay

If custom Threepole still feels slower than original, determine whether the delay is in:

- Bungie API freshness
- CharacterActivities/component 204
- Transitory/component 1000
- response freshness/state acceptance
- activity history
- manifest/classification lookup
- local frontend state
- overlay rendering

Do not increase polling frequency before identifying the delayed path.

## Priority 3 — Preserve API-/web-only safety

Every code change should continue to satisfy:

- no process injection
- no DLL/code injection
- no memory read/write
- no `destiny2.exe` hooks/access
- no DirectX/game hooks
- no input hooks or automation
- no Destiny file edits
- no persistent helper service

Any new Bungie endpoint/component should document its purpose, frequency, caching and fallback behavior.

## Priority 4 — Revert application icon

Latest requested direction is to restore the standard Threepole icon for:

- application
- taskbar
- tray
- installer

Keep the original Threepole calendar/clear icon beside clear counts.

Treat this task as open until the code and resulting Windows build have been verified.

## Priority 5 — Documentation alignment

After behavior is verified:

- update README statements that are no longer true
- keep custom and original Threepole features clearly separated
- keep this knowledge base synchronized with meaningful architecture/behavior changes
- avoid documenting a feature as complete before implementation/build/live validation supports it

## Priority 6 — Optional feature polish

Only after responsiveness and correctness are stable, consider UI/feature polish such as:

- activity-name presentation refinements
- hide-delay UX
- activity icon fallback behavior
- clearer custom-settings grouping
- additional optional UI metadata

Cosmetic features must not add meaningful hot-path API or manifest work.

## Suggested engineering improvements

### Lightweight timing diagnostics

Add development/debug instrumentation that can distinguish timestamps for:

- API request start/end
- accepted CharacterActivities update
- accepted optional timing update
- history clear result
- frontend state receipt
- visible overlay transition

Keep diagnostics disabled or low-overhead in normal use. The goal is to identify which layer is late without guessing.

### Repeatable live test matrix

Maintain a small manual test matrix for releases that touch activity timing:

- cold app launch
- first Dungeon/Raid entry
- orbit return
- same-activity restart x3
- different-activity switch
- clear notification
- missing/private optional timing fallback

### Regression tests

For every reproducible state bug, prefer adding a focused regression test before or alongside the fix.

## “Threepole Status” output

When asked for **Threepole Status**, summarize the entire project rather than only recent weekly work.

Use this order:

1. Latest GitHub/Codex changes
2. Current branch/PR/build state
3. Current feature state
4. API/safety status
5. Performance status
6. Open bugs and validation gaps
7. Risks
8. Improvement ideas
9. Next priorities

Sources should include GitHub plus available ChatGPT/Work/Codex context. If a source is unavailable, say so instead of assuming its state.

## Current next action

The highest-value next action is live testing of 1.1.4 or later against original Threepole, especially orbit responsiveness and repeated same-Dungeon restart behavior. Use those results to decide whether another timing/state code change is actually needed.