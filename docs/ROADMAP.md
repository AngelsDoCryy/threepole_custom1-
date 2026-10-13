# Threepole Custom — Technical roadmap

Updated 2026-10-13 with the subsequent clarification: preserve existing window/tray behavior and compare future optimizations directly with original source. **All runtime changes below are proposals unless explicitly marked otherwise.** This clarification changes documentation only; any implementation requires separate approval. Keep PR #1 open and Draft. Current runtime remains 1.1.4.

Safety requirements are in [PROJECT_CONTEXT](PROJECT_CONTEXT.md), source-level API/cache findings in [DESTINY_API](DESTINY_API.md), notification behavior in [CUSTOM_FEATURES](CUSTOM_FEATURES.md), and the complete live matrix in [PERFORMANCE](PERFORMANCE.md). Do not duplicate those inventories here.

Permanent direction: measurably exceed original Threepole's speed/reliability while retaining Custom features at preferably equal or lower resource cost. Apply the priority order safety → correctness → speed beyond original → low resource use → convenience. Each package must assess security, CPU/RAM, API/network volume, overlay latency/reliability and effects on existing settings/features. Technical expectations are not benchmark results; any proposed resource increase requires the user's prior decision. No acceleration through riskier game access, substantially greater system load or unjustified polling increases.

## P0 / Work package 1 — Safety and full-exit verification; no blanket window/tray rebuild

**Existing evidence:** Source-level analysis is complete; Windows exit verification remains open. `pollers/overlay.rs::get_hwnd_exec` opens the foreground process for executable-name inspection, including uncached non-Destiny apps; that is a concrete safety conflict. Main X closes the UI while tracking/overlay continue, simple tray click reopens it, and tray Exit directly terminates the process. The user accepts this window/tray interaction. No independent polling service or surviving Rust poller has been demonstrated.

**Verification and decision sequence:**

1. Keep the established interaction unchanged: main X removes its window while the app remains in the tray; simple tray click reopens it; minimize, auxiliary windows and first-start profile selection retain their behavior. Do not add mandatory double-click, retained hidden main webviews or a window-role redesign.
2. Verify full tray Exit during startup, slow manifest/history, one-off profile/search/metadata requests, profile switch and overlay on/off. Check that current/history/1000, overlay checks, timers/listeners, pipe/activation work and app-owned WebView2/resources cease and immediate restart works. Lack of central task tracking alone does not establish faulty exit. Record concrete reproduction and ownership evidence before proposing a correction.
3. If an exit defect is demonstrated, propose the smallest targeted fix with safety, CPU/RAM, API, overlay and Custom-feature effects. Targeted cancellation, handle cleanup or prevention of late activation are options only where evidence requires them; a central shutdown framework is not a default deliverable. Implement only after user approval. If no defect is found, retain the exit path and record test coverage/limits.
4. Keep the proven process-access conflict separate from exit behavior. Recommend the smallest process-free overlay detection change after validating foreground title/class, geometry and ambiguity handling on Windows; preserve the existing 200 ms cadence initially. Do not use WMI or reduced process-handle permissions as a workaround. This proposal also requires separate implementation approval and must not be bundled with an unnecessary window/tray rebuild.

**Alternatives and limits:** Title/class matching cannot authenticate executable identity and may break with changed window metadata; hide on uncertainty and validate recreation/Alt-Tab/DPI behavior. Manual selection reduces automatic matching assumptions but adds user effort and must handle window replacement. Manual overlay positioning is a weaker fallback. Bungie APIs cannot supply local focus/geometry. Out-of-context Windows event notifications could reduce polling but add hook/message/cleanup complexity; they are not the recommended first implementation or an authorization to add hooks. Exclusive fullscreen remains unsupported by this separate overlay; no game-hook workaround.

**Effects and limits:** Documentation/verification does not change runtime cost or API cadence. Any later fix must quantify its effect instead of presuming a rewrite is faster or safer. Window-only targeting removes forbidden access, but its CPU cost and reliability need comparison against the existing path. Already-sent HTTP cannot be recalled server-side; verify no Threepole-originated work/requests after completed exit. Avoid new services/processes, extra hidden-window RAM and shutdown races/deadlocks introduced by unnecessary complexity. Never globally kill WebView2 or touch Destiny processes. Preserve all Custom settings, name auto-hide, timer/clear independence and profile state.

**Acceptance:** Document exit evidence and remaining limits while preserving X, simple tray click, minimize and auxiliary-window behavior. No app-owned tasks/requests/helpers remain after full Exit; the named pipe is released and restart succeeds. A separately approved safety correction must remove forbidden process access and pass Alt-Tab, movement/resize, DPI/multiple displays and game-window replacement checks without degrading overlay/tracking. Compare relevant original/Custom code and runtime costs. No new service, scheduled task, autostart helper or background dependency; no claim of full safety or resource superiority without evidence.

## P1.1 — Decouple activity identity from manifest availability

**Evidence:** For nonzero hashes, `playerdata.rs::update_current` awaits metadata and returns on most manifest errors before publishing the activity. Orbit itself skips metadata, but waits behind any preceding slow update. `api.rs::ActivityInfoSource::get` caches successful empty-name definitions indefinitely. Metadata source mutexes span network waits; helper tests do not exercise source recovery.

**Proposed implementation:** Publish accepted 204 identity/timing independently, then enrich metadata within the same cancellable app lifetime. Preserve known classification for the same hash where available; represent unknown metadata conservatively. Avoid holding the global metadata lock across network I/O; deduplicate requests by hash and ignore results belonging to an obsolete profile/activity. Make unusable metadata retryable with bounded backoff rather than caching it permanently.

**Speed/API effects:** Removes manifest latency from status propagation. Keep 204/1000/history cadences unchanged. Reuse cached definitions and collapse concurrent misses; do not add per-history-item requests. Recovery of an empty/failed entry can add bounded retry requests compared with today's stuck-cache behavior; define and measure that budget before shipping. Unknown-classification UI may remain limited until enrichment, but must not retain the previous activity as if current.

**Acceptance:** Mock manifest timeout, network/API/parse failure, ResponseMissing and empty name followed by a valid definition. Accepted status/orbit arrives without waiting; current hash/timer never reverts after late metadata. Test profile switch, same-hash restart, concurrent details-window lookups, all display settings and successful enrichment. Verify network deduplication/retry counts, no false raid classification and cancellation.

## P1.2 — Publish initial activity without waiting for history

**Evidence:** Initial current/history calls must both succeed before `last_update` is set and the independent loops start. Slow history delays startup; failure restarts the initialization attempt.

**Proposed implementation:** After required profile/current data, publish partial readiness and start independent current/history/optional paths. Model “history not loaded” separately from “loaded and empty”; keep history failure/recovery separate from status errors. Preserve profile reset cancellation. First successfully loaded history is a notification baseline, so asynchronous loading must not create a flood of old clears.

**Speed/API effects:** First usable activity no longer waits for character/page history scans. Do not increase cadence or create duplicate initial/current/history fetches. Optional timing may begin earlier, so startup request timing/concurrency changes even when steady-state rates do not; measure cold-start counts and bound concurrency.

**Acceptance:** Slow/failing history cannot delay fresh activity/orbit; history later populates correct per-hash counts. Empty/private/offline/profile-switch cases behave predictably. Initial baseline produces no retrospective clear burst. Verify old profile requests cannot publish after reset, no duplicate loops and unchanged steady-state request budget. Depends on the metadata behavior established in P1.1.

## P1.3 — Correct clear-notification delivery

**Evidence:** `overlay.ts::refresh` checks only the newest history row and stores its ID even when incomplete. A false-to-true completion correction of that ID or a late older clear generates no popup. Both were reproduced in a production-function memory harness; existing ordering tests do not cover them.

**Proposed implementation:** Track notification eligibility/completed instance IDs against the loaded-history baseline instead of only the first row. Bound state to the relevant profile/day/history and define reset/pruning behavior. Handle corrected completion, multiple arrivals and repeated snapshots exactly once. Preserve no historical startup flood and explicitly define notification-disable/re-enable behavior without surprising backlog delivery.

**Speed/API effects:** No new endpoint or polling. Process existing bounded daily history and avoid repeated full UI work where unchanged. An instance-ID set costs small per-history-entry memory; specify pruning. Notification logic must not own activity/timer state.

**Acceptance:** Regression cases for normal clear, incomplete-to-complete same ID, older arrival, several new clears, duplicate/reordered snapshots, baseline arrival after delayed history, day reset, profile switch and disabled/re-enabled notifications. A late clear neither forces orbit nor stops a newer run. Per-hash counts, timer/name visibility and auto-hide remain independent. Coordinate with P1.2's readiness/baseline model.

## P2 — Measure remaining latency and harden bounded recovery

Capture available baseline measurements before runtime changes; after each P0/P1 package repeat relevant comparisons, then complete PERFORMANCE's long-session/original comparison matrix. Measurement is not deferred wholesale until P2. Earlier 30-second first-entry / 2–3-minute restart observations do not prove a current 1.1.4 bug. Do not require live reproduction before fixing already-demonstrated local failure paths.

Add opt-in, bounded, redacted timing diagnostics only if needed to distinguish API data age, request/lock delay, state acceptance and frontend display. No additional requests or external logging service. Cover cold/warm startup, orbit, same-Dungeon restart x3, activity/character/profile changes and clear ordering.

Then consider separate evidence-led changes: honor Bungie throttle durations; isolate error sources/stale UI; validate unknown-history classification, cache invalidation and day-boundary behavior; measure history pagination, payload cloning, metadata contention, details-window rendering and window-handle cache behavior. Do not solve incomplete history by an unbounded manifest request per record. Each change needs its own correctness test and before/after request/CPU/startup evidence. Polling increases are not currently justified.

## P3 — Complete Eager Edge and restrained UI polish

Replace the remaining overlay-clear calendar with the existing local Eager Edge asset. Inventory all Threepole Custom icon surfaces, including app/taskbar/tray/installer/popups and remaining UI symbols, against the user's all-icons requirement. Verify Windows scaling/installer rendering and preserve control usability; report scope ambiguities before implementation rather than claiming complete icon coverage.

No remote icon requests or restored activity-type icon endpoint. Keep this independent from timing changes. After core stability, consider hide-delay validation, name presentation and optional class metadata only if separately requested. The removed activity-icon setting is not a pending default restoration.

## Validation and delivery for implementation orders

- Focused frontend/Rust regressions plus the existing five overlay and 37 Rust baseline checks where applicable.
- Vite build; separate type-check coverage should be added/evaluated in a dedicated tooling change, because current CI does not provide it.
- Windows MSI build for runtime/packaging changes; test after Tauri configures Cargo features as the current workflow does.
- Record commit, test results, build/artifact link, request-volume/CPU/RAM impact and remaining live acceptance, including FPS/frametime/microstutter evidence where safely available. Distinguish measured gains from expectations. A green build does not prove API latency, resource savings or exit behavior.
- Keep version notes and the owning knowledge file aligned with shipped behavior. Do not merge PR #1 automatically.

## Minimal environment decision

Use existing Codex/Git plus the repository's on-demand GitHub Windows workflow initially; no local Rust/C++ installation is needed for documentation work and CI-built MSI delivery. Local frontend checks need Yarn/project dependencies; local full builds additionally need Rust MSVC and C++/SDK tooling. The read-only inventory and exact tiers are in [BUILD-GITHUB-DE](../BUILD-GITHUB-DE.md). No tools/services were installed for the documentation cleanup.

## Threepole Status

Use [PROJECT_CONTEXT](PROJECT_CONTEXT.md)'s project-wide status convention: latest changes; branch/PR/build and features; safety/API/performance state; open bugs and live gaps; risks/improvements; next priorities. Include available GitHub/Codex and retained ChatGPT/Work decisions. State unavailable evidence instead of inventing it; do not restrict to one week unless asked.
