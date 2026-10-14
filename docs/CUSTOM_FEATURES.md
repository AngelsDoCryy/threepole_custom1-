# Threepole Custom — Features

Verified implementation: 1.1.4, audited on 2026-10-11. This file distinguishes shipped behavior from requirements and pending changes. Safety and lifecycle requirements are owned by [PROJECT_CONTEXT](PROJECT_CONTEXT.md); implementation work is ordered in [ROADMAP](ROADMAP.md).

Permanent preservation rule, confirmed 2026-10-13: retain activity names/auto-hide, per-activity daily clears, clear notifications, optional milliseconds and independent overlay settings; retain dynamic Raid/Dungeon classification, caches and API improvements, disabled updater, separate installation and required Eager Edge branding. Optimize these features without accidental removal or coupling. Existing notification/cache/icon gaps below remain open; this rule does not claim every target is already met.

## Activity name and independent settings — implemented

The overlay displays the name of a recognized tracked activity (Raid, Dungeon, Strike or Lost Sector). It uses an instance key consisting of activity hash and selected start time, so a recognized new run of the same activity shows the name again.

Name display and auto-hide have separate controls. The default delay is ten seconds; the UI offers 0–3600 seconds. This range is a UI constraint, not comprehensive backend validation: Rust stores a `u32` and does not enforce the 3600-second maximum. Input validation and recovery remain polish work.

Separate preferences exist for overlay enablement, timer, milliseconds, name, auto-hide, clear count and clear notifications. UI controls can be disabled when their parent feature is off (for example milliseconds without a timer); their stored preferences remain distinct. Overlay defaults to off; the other display options default to on. Old preference files receive defaults through Serde.

The former `displayActivityIcon` preference and activity-type icon request were removed. Optional class/activity metadata is a deferred idea, not a current setting or permission to override the Eager Edge requirement.

## Timer — implemented, performance not yet benchmarked

The frontend derives elapsed time from its local clock minus the accepted backend start time. 204 supplies activity identity and fallback timing; 1000 can advance the start conservatively. See [DESTINY_API](DESTINY_API.md) for association limits.

The overlay timer interval stops when hidden, disabled or outside recognized activity. It updates at about 30 Hz with the milliseconds option, otherwise 2 Hz; the current formatter displays two fractional digits. The separate details window still runs a 30 Hz interval independently of overlay preferences. No additional Bungie requests are caused by these presentation settings.

## Per-activity daily clears — implemented with limits

The overlay counts completed history records with exactly the current activity hash. Returning to that hash restores its count from the loaded history. Different difficulty/variant hashes are not grouped. The details window retains an aggregate total for all tracked activities.

Counts are reconstructed from Bungie history rather than stored as a separate persistent counter. The cutoff is 17:00 UTC, applied to activity **start** (`period`), not completion time. Completion requires both the API completed value and successful completion reason. History replacement accepts late older records, corrected results and reset removals.

Incomplete history modes are supplemented from the known classification cache; unknown hashes can still be omitted. Day-boundary runs, multiple characters, variant hashes and cache-empty startup need live validation; see DESTINY_API and PERFORMANCE.

## Raid / Dungeon classification — implemented with limits

Activity definitions provide modes and a fallback via raid/dungeon `activityTypeHash`. The persistent activity-mode cache remembers learned raid/dungeon classifications. It is separate from live timing and is not a complete local manifest database.

History classification does not fetch a manifest definition for each record. An unseen hash with incomplete modes is omitted until its classification is learned elsewhere. The details UI can request metadata for retained historical activities. Manifest-error independence and recovery from cached empty names are **requirements not fully met**, documented in DESTINY_API.

## Icons — current requirement and actual assets

**Requirement, confirmed 2026-10-11:** Eager Edge for all Threepole Custom icons, including app, taskbar, tray, installer, startup/clear notifications and the overlay-clear icon. Earlier requests to restore the original app icon or preserve the calendar as the desired clear icon are superseded.

| Surface | Current 1.1.4 source | Status |
| --- | --- | --- |
| App/taskbar/installer and tray | `src-tauri/icons/icon.ico`, selected in `tauri.conf.json` | Eager Edge asset; Windows installation/display recheck remains |
| Startup and clear popups | `public/glyph.png`, same repository blob as `assets/eager-edge-app-icon.png` | Eager Edge implemented |
| Overlay clear counter | Inline calendar SVG in `src/overlay/overlay.html` | **Pending replacement with Eager Edge** |

This documentation cleanup changes no image or HTML asset. Generic functional/platform symbols still exist in the UI; a future icon pass must inventory them explicitly against the user's all-icons requirement rather than claim that every visible symbol has already been converted.

## Clear notifications — implemented independence, confirmed gaps

Notifications are optional overlay popups, queued while the overlay is hidden. A clear may arrive before live orbit detection. Neither a clear nor a notification forces orbit or stops an ongoing/new run. Startup suppresses retrospective clear notifications.

Rechecked on `42bd50c` (2026-10-14): `popups.ts` retains only rendered title/subtext in an unbounded hidden queue and flushes on show. A visible popup can therefore be older than its history arrival, including during a new run. The live timer has no completed/frozen state and continues from its accepted start while the activity remains recognized. This behavior is separate from the missed-notification defects below. DESTINY_API owns the history/current identity limits and the unused Transitory end-time opportunity; no timer-stop change is approved. A future notification fix should retain instance/profile identity until delivery and define bounded queue/expiry behavior with the user, without silently dropping requested notifications.

Current `src/overlay/overlay.ts::refresh` examines only `activityHistory[0]` and remembers its instance ID even when incomplete. Consequently:

- An initially incomplete record corrected to completed with the same ID gets **no notification**.
- A late older completed record behind the newest entry gets **no notification**.
- The backend/history count can update correctly while the notification is missed.

The 2026-10-11 takeover reproduced both cases in an isolated in-memory execution of the production refresh function with UI stubs. That is code-level evidence, not an installed-app/live test. Existing two clear/orbit regressions do not cover these cases. ROADMAP proposes bounded per-instance notification tracking while preserving initial-history suppression, settings independence, profile resets and timer independence.

## Installation and lifecycle

Custom app ID `sh.des.threepole.custom`, config directory `threepole-custom` and named pipe `threepole-custom-open` isolate the custom app. The automatic updater is disabled so official releases cannot overwrite it.

**Current and accepted behavior:** X destroys the UI window while the tray/player-data/overlay work continues; – minimizes normally; simple tray click reopens/activates. Preserve this behavior, including auxiliary windows. Disabling overlay closes its window/poller but not the player-data task. Tray Exit directly terminates the process. Verify that all app-owned work/resources cease; absence of central cancellation/cleanup alone is not a proven failure. No mandatory double-click or retained-hidden-window redesign is required. PROJECT_CONTEXT owns the latest clarification; any concrete exit defect needs a minimal proposal and separate implementation approval. Package 1 removes the separate process-name lookup conflict using window-only overlay recognition. Its observed metadata and remaining live acceptance are recorded in PERFORMANCE; no window/tray redesign accompanies it.

## Deferred polish

After safety, manifest/startup/notification correctness and responsiveness validation: improve hide-delay validation/UX and name presentation, then consider optional class metadata. Cosmetic work must not introduce hot-path network requests, couple independent settings or reintroduce removed activity-icon requests.
