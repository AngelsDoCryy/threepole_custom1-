# Threepole Custom — Destiny / Bungie API

This file documents the API- and manifest-only data model used by the custom Threepole work.

## Safety boundary

Custom functionality must use Bungie web/API/manifest data only.

Do not introduce:

- Destiny 2 memory access
- process inspection for gameplay state
- DLL/code injection
- hooks into `destiny2.exe`
- DirectX/game hooks
- input hooks
- gameplay automation
- Destiny game-file modification

If a desired feature cannot be implemented inside this boundary, treat it as unsupported for this project rather than bypassing the boundary.

## Current activity identity / orbit state

### CharacterActivities — component 204

Current 1.1.4 architecture uses CharacterActivities independently for activity identity and observed orbit state.

Design rules:

- poll on the existing current-activity cadence with a two-second pause
- keep this request independent from optional timing data
- do not let a slow optional request block activity identity/orbit updates
- accept newer status snapshots based on response freshness
- do not reject a newer response merely because `dateActivityStarted` moved backwards

`dateActivityStarted` is an activity-start value, not the authoritative generation timestamp for deciding whether the full status snapshot is newer.

A fresh orbit response may legitimately have an older or epoch-style activity-start value.

## Optional timing source

### Transitory — component 1000

The current design fetches Transitory independently with a ten-second pause/cadence.

Purpose:

- provide optional timing information
- never act as the sole proof of activity identity or orbit

Important limitation:

- Transitory does not carry an activity hash, so it cannot independently identify the current activity

Fallback behavior:

- missing data
- private data
- malformed data
- future schema differences
- older timing data

must fall back safely without preventing CharacterActivities from updating the overlay state.

Confirmed orbit should clear any previously accepted Transitory start so timing cannot leak into the next run.

## Activity history / clear notifications

History processing is a separate state path from live current-activity tracking.

Required invariants:

- a clear result must not force orbit
- a clear result must not stop a later run
- a notification can legitimately arrive before the live current-activity response catches up

Ordering alone is therefore not proof of a bug. State corruption between the paths would be a bug.

## Manifest usage

Manifest data is used for activity metadata/classification where normal mode information is incomplete.

Rules:

- cache stable activity-hash -> mode/classification data locally
- avoid repeated manifest requests in hot paths
- unknown/future activities must fail gracefully
- manifest lookup failure must not break the timer or base activity tracking

## HTTP handling

Current optimization direction:

- use one shared HTTP client for the app session
- preserve Bungie affinity/cookie behavior between requests
- do not add extra requests solely for affinity handling

Bungie controls when API data becomes fresh. Local code can prevent avoidable blocking or stale-state rejection, but it cannot guarantee immediate server-side updates.

## Request-volume principle

Keep request volume conservative and measurable.

Do not increase polling frequency as the first response to a timing complaint. First identify which path is delayed:

- CharacterActivities response freshness
- Transitory timing
- activity history
- manifest/classification lookup
- local state handling
- UI rendering

Only change cadence when there is evidence that cadence itself is the limiting factor and the API impact is understood.

## Future API documentation

When adding a new Bungie endpoint/component or changing how an existing one is used, document:

1. endpoint/component
2. purpose
3. polling/call frequency
4. cache behavior
5. fallback behavior
6. failure/privacy handling
7. effect on request volume
8. why it remains inside the API-/web-only safety boundary