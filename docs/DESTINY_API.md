# Threepole Custom — Destiny / Bungie API

Implemented baseline: 1.1.4; activity/timer analysis rechecked on `42bd50c` on 2026-10-14. Source: `src-tauri/src/api.rs`, `api/requests.rs`, `api/responses.rs`, `pollers/playerdata.rs` and `pollers/activity_timing.rs`. P1.1 implementation updated 2026-10-15; other recommendations remain pending. PROJECT_CONTEXT's safety and game-performance priorities govern API changes.

## Required safety boundary

Activity data must come from Bungie web/API/manifest sources only. No Destiny process access/inspection, memory read/write, DLL/code injection, game/DirectX/RTSS-style hooks, input hooks/automation or game-file modification. Unsupported features must stay unsupported rather than bypass this boundary.

Package 1 removes the inherited overlay process-name query; window metadata is used only for overlay focus/placement and adds no Bungie requests. Ordinary Windows window queries without direct process access are allowed for the separate overlay, not as a source of gameplay state. Intentional tray operation keeps tracking active; full tray Exit must stop app work. [PROJECT_CONTEXT](PROJECT_CONTEXT.md) owns the findings and binding lifecycle contract.

## Current endpoints and request budget

The API root is `https://www.bungie.net/Platform`. The app sends `X-API-Key` compiled from `BUNGIE_API_KEY`; profile selection is not an OAuth login. Private data is not bypassed.

| Endpoint/component | Purpose | Current call pattern and cache |
| --- | --- | --- |
| SearchDestinyPlayerByBungieName | Find profile by name/tag | User-triggered POST, no polling |
| GetProfile, 100 | Profile information and character IDs | On demand; session cache per profile |
| GetProfile, 204 | Observed activity identity and orbit | Initial request, then two-second pause between completed updates |
| GetProfile, 1000 | Optional current-activity start | First request after initialization, then ten-second pause; latest optional snapshot in memory |
| GetActivityHistory | Today's tracked runs/clears | Initial load, then ten-second pause per full round; per character, `mode=7&count=25&page=...` |
| GetDestinyEntityDefinition / DestinyActivityDefinition | Name, modes and metadata | On demand by hash; session cache plus persistent learned raid/dungeon modes |

Pauses are in addition to request and processing time. Approximate steady-state maxima are 30 requests/minute for 204 and six for 1000. History costs about six times the number of pages fetched across all characters per minute when response time is negligible; three characters at one page each already mean about 18 history requests/minute. Startup, retry, profile search and uncached metadata are additional. “Six extra requests” describes only the optional 1000 path, not the whole application or every earlier polling refactor. Do not raise cadence without measured need and a quantified budget.

## 204 identity, character selection and freshness

204 is requested independently of 1000. The implementation chooses the character observation with the greatest `dateActivityStarted`; P1.1 uses the selected observation's optional mode fields for classification, but selection still does not use an independent online-character signal. Multiple-character selection, equal start times and epoch orbit values therefore need integration/live coverage beyond single-observation timing tests.

`ActivityTiming::resolve` compares primary `responseMintedTimestamp` against its accepted watermark. A strictly newer status can report a different activity or orbit even when the activity-start timestamp moves backwards. Epoch start is accepted for hash 0. Older primary snapshots are rejected; missing/invalid freshness metadata preserves the conservative legacy guard. Hash 0 is the app's observed no-activity/orbit state, not an independent proof of the user's exact screen.

## Optional 1000 timing

`secondaryComponentsMintedTimestamp` is tracked separately. Only a newer secondary snapshot replaces the optional start; missing/private/malformed/future/invalid optional values are tolerated. Network failures leave the latest stored optional snapshot available; freshness guards still apply when resolving a subsequent 204 response.

A Transitory start is considered only when it is valid, not after its secondary mint time, not earlier than the character's start, and not later than the primary mint time. It can advance a same-activity timer without waiting for 204's start value to catch up. An accepted timer does not roll back merely because optional data disappears. Confirmed orbit clears the accepted optional start while retaining the freshness watermark.

Transitory has no activity hash and comes from a non-authoritative system separate from PGCR. Even a fresh primary timestamp cannot prove both sources describe the same run. It must never be used alone to identify an activity, infer orbit or terminate a run.

Official schema references: [CharacterActivities](https://bungie-net.github.io/multi/schema_Destiny-Entities-Characters-DestinyCharacterActivitiesComponent.html), [primary/secondary freshness](https://bungie-net.github.io/multi/schema_Destiny-Responses-DestinyProfileResponse.html), [Transitory current activity](https://bungie-net.github.io/multi/schema_Destiny-Components-Profiles-DestinyProfileTransitoryCurrentActivity.html). These support source/field semantics, not a promise of a fixed Bungie refresh delay.

## History and initialization

A history round scans characters and pages until empty data or entries older than the 17:00 UTC cutoff. It filters by activity start time, supplements incomplete modes from the classification cache, retains Raid/Dungeon/Strike/Lost Sector records, and sorts by period with instance ID as a stable tie-breaker. Full-record comparison accepts late arrivals, corrected completion/duration and reset removals.

Current, history and Transitory futures remain joined after initialization, while P1.1 adds demand-driven metadata enrichment inside the same cancellable task, also during initial history loading; history writes only its own field and cannot overwrite newer current activity. Profile reset aborts the task and recreates timing state. **However, initial publication still waits for successful current activity and history together.** A history error causes another initialization attempt after five seconds; steady-state polling and optional timing have not started yet. This is a confirmed local startup bottleneck, not a Bungie freshness explanation.

The frontend counter and notification delivery are separate: history corrections can change counts without generating notifications. See [CUSTOM_FEATURES](CUSTOM_FEATURES.md) for the confirmed same-ID correction and late-older notification gaps. A clear must never force orbit or stop a subsequent run.

## Metadata caches and confirmed limitations

- Full/partial `ActivityInfo` values and retry bookkeeping are cached by hash, bounded to 256 least-recently-used entries in P1.1. An individual manifest definition supplies name/modes/image; this is not a downloaded manifest database.
- `activity-mode-cache.json` persists only learned raid/dungeon modes. It stores classification, never live timing. Read/parse failures fall back to an empty cache; write failures are ignored. P1.1 performs writes on the existing runtime blocking pool, outside the memory-cache lock; a writer gate and latest snapshot prevent older writes replacing newer snapshots. The persistent learned-mode map still has no manifest-version/TTL invalidation or size bound.
- Raid/dungeon `activityTypeHash` supplements missing/generic mode data. History classification makes no per-item manifest request. With incomplete modes and an unknown hash it drops the record until another path learns that definition. The details window can fetch metadata for retained historical records, so the whole UI is not “manifest-request-free.”

**P1.1 correction:** `update_current` now commits accepted hash/start/modes using memory-only lookups. A coalescing watch slot asks an enrichment future for the latest needed hash; it adds no periodic timer and uses existing status opportunities for retries. The source shares one async request gate across tracking and details IPC, with a cache recheck after acquisition to deduplicate misses. Cache reads, history classification and status never wait on this request gate. Hash 0 requests no definition. Complete cached definitions also bypass the request gate.

**Recovery/association:** network, parse, missing, blank-name and missing/generic-mode results are retryable with pauses of 30, 60, 120, 240, then at most once every 300 seconds per retained entry, after completion. Cancellation also leaves a cooldown and releases the request gate. Good fields remain usable while incomplete metadata recovers. The new worker may update only metadata/classification for the matching hash and profile generation; it cannot replace hash/start/history. A same-hash new run can reuse its definition without losing its newer start. A still-running obsolete lookup can delay the next name lookup, but not status/orbit. Complete-entry eviction can require a later refetch; failure-entry eviction can reset backoff.

## HTTP behavior and remaining risks

Implemented: one session-wide reqwest client, connection reuse, 15-second request timeout, custom in-memory cookie name/value replay without reqwest's cookie-store dependency stack. Affinity itself adds no requests; it does not establish a cause or guaranteed cure for backend inconsistency. The simplified cookie map is not a full expiry/domain/path-aware cookie jar.

Bungie errors expose `ThrottleSeconds`, but callers currently keep their fixed retry pauses instead of honoring that duration. Error state is shared by current/history paths, so a successful status update can clear a history error. After successful initial load, the overlay can continue showing last good data without making subsequent errors obvious. These are follow-up robustness risks, not newly measured live regressions.

## Retained pre-P1.1 analysis — 2026-10-14 (`42bd50c`)

| Finding in the audited code | Consequence and evidence limit |
| --- | --- |
| `should_refresh_current` compares both hash and resolved start | Same-hash restarts are already supported when 204 changes its start or acceptable 1000 timing advances it. Unchanged data from both sources cannot establish an otherwise invisible restart. |
| `ActivityTiming::resolve` accepts fresh backward/epoch orbit and separates raw 204 start from optional timing | Earlier timestamp-guard defects are corrected and unit-covered. This does not prove the older 30-second / 2–3-minute observation is fixed in Destiny. |
| Initialization reads current, then all history, before publishing or starting independent loops | The first published current snapshot can already be old; slow/retried history also delays the first 1000 request. This is a confirmed local gate. |
| Timing is resolved before `update_current` awaits metadata | Accepted internal timing may be newer than published overlay state after a metadata failure. Cached lookups can still wait behind another hash's network request under the shared mutex, including details-window IPC. |
| History clones cached modes under that same mutex; cache persistence is synchronous | Independent joined futures remove serial network scheduling, not every shared-lock or synchronous-work delay. Cost beyond the demonstrated waits needs measurement. |
| Successful 1000 fetches only store a snapshot; the next 204 update consumes it | Already received timing waits for the next status cycle, including its request/processing delays. Event-driven re-evaluation could remove that local wait without another request, but must retain primary identity/freshness guards. |
| 204 character selection takes maximum start time before freshness resolution and discards the selected character ID | Equal timestamps with different hashes have no deterministic tie-break; an epoch-orbit character can lose to another character's older positive start. These are conditional algorithm risks, not a reproduced live cause; inspect multi-character payloads before choosing a new heuristic. |
| Overlay receives current and history in one snapshot; history updates replace only history | No demonstrated history overwrite of a newer timer. Snapshot listeners are registered before initial reads. The numeric timer draws on its existing 2/30 Hz interval, rather than immediately on each event; ordinary render cadence alone does not explain minutes. WebView scheduling cost remains unmeasured. |

There is no application cache of raw 204 HTTP responses in this path; the app retains the accepted state and latest optional snapshot. A response arriving recently can still contain old server data. Primary and secondary mint timestamps describe different source freshness, not the exact time of the player's action. Current guards reject strictly older primary and non-newer secondary timestamps; equal-primary contradictory payloads, missing timestamps and local-clock skew remain regression/diagnostic targets. Bungie-side lag, caching and source ordering are plausible, but no captured live response trace establishes their contribution to the reported delay or a fixed refresh SLA.

**Run identity and safe completion:** `currentActivityHash` identifies a definition, not a run. The overlay's `hash:startDate` key is a display heuristic; a timing correction can change it without a new run. History has a real `instanceId`, but the consumed 204/1000 current-activity fields do not provide the same ID. History uses `directorActivityHash` and merges per-character results without retaining their originating character ID. Hash alone, or hash plus a guessed period/duration match, must never authorize stopping the current timer. Keep notifications keyed to historical instance IDs independently of live timer ownership; if later association is needed, retain the already available profile/character provenance and reject ambiguous mappings without extra PGCR requests. [Official history identity schema](https://bungie-net.github.io/multi/schema_Destiny-HistoricalStats-DestinyHistoricalStatsActivity.html).

**204 modes, now used by P1.1:** the parser accepts optional `currentActivityModeType(s)` without failing status on missing/malformed modes. Accepted observation modes travel through the existing freshness resolver and are published separately from `activityInfo`. Explicit modes win; absent/generic modes may use this hash's manifest or learned classification. Unknown classifications remain hidden in the overlay; no mode is fabricated and no new component is requested. **Still unused:** Transitory current activity also exposes nullable `endTime` for an ended activity the player has not left; the parser currently reads only `startTime`. Thus remaining in the activity after completion is distinct from failed orbit detection. Bungie explicitly treats Transitory as non-authoritative and separate from PGCR. Do not introduce automatic timer freeze from `endTime` until same-run association, freshness, late joining, repeated starts and stale end values are validated and the user approves the behavior. No history clear or optional end alone may force orbit. See the official 204/1000 schemas linked above.

## Rules for endpoint changes

Document purpose, call frequency, caching, privacy/failure fallback, total request impact and why the endpoint stays within the safety boundary. Diagnose the complete API → retrieval → processing → state → overlay chain: Bungie freshness, request duration, metadata/locks, state acceptance and UI separately. Improve speed beyond original through bounded parallelism, deduplication and cache reuse; do not assume more requests produce fresher server data. Include startup/retry/page traffic, not only steady-state polling, in before/after budgets.

Removing waits can raise the effective number of completed cycles toward the existing 30/minute (204) and six/minute (1000) ceilings even when pauses remain unchanged. Report actual before/after counts; “no added endpoint or scheduled fetch” is not proof of identical traffic. Prefer a deduplicated current-activity enrichment path and existing responses; bound outstanding work and recoverable metadata retry traffic explicitly.

Preserve current status/optional/history task ownership and X/tray behavior. Verify that full Exit ends initialization, polling, sleeps, lock waits and one-off IPC profile/search/metadata requests without waiting for the 15-second timeout, with no new app requests after completed exit. An already sent request cannot be recalled from Bungie's server. The current path terminates the process; no surviving request task has been demonstrated. If a concrete failure requires targeted cancellation or cleanup, propose the smallest correction for approval rather than mandate a central shutdown framework. Closing/reopening the main UI must not duplicate polling.
