# Threepole Custom — Destiny / Bungie API

Implemented baseline: 1.1.4; audited 2026-10-11. Source: `src-tauri/src/api.rs`, `api/requests.rs`, `api/responses.rs`, `pollers/playerdata.rs` and `pollers/activity_timing.rs`.

## Required safety boundary

Activity data must come from Bungie web/API/manifest sources only. No Destiny process access/inspection, memory read/write, DLL/code injection, game/DirectX/RTSS-style hooks, input hooks/automation or game-file modification. Unsupported features must stay unsupported rather than bypass this boundary.

The inherited overlay process-name query is a documented unresolved conflict, not an exception. Ordinary Windows window queries without direct process access are allowed for the separate overlay, not as a source of gameplay state. Intentional tray operation keeps tracking active; full tray Exit must stop app work. [PROJECT_CONTEXT](PROJECT_CONTEXT.md) owns the findings and binding lifecycle contract.

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

204 is requested independently of 1000. The implementation chooses the character observation with the greatest `dateActivityStarted`; it does not currently use the component's mode fields or an independent online-character signal. Multiple-character selection, equal start times and epoch orbit values therefore need integration/live coverage beyond single-observation timing tests.

`ActivityTiming::resolve` compares primary `responseMintedTimestamp` against its accepted watermark. A strictly newer status can report a different activity or orbit even when the activity-start timestamp moves backwards. Epoch start is accepted for hash 0. Older primary snapshots are rejected; missing/invalid freshness metadata preserves the conservative legacy guard. Hash 0 is the app's observed no-activity/orbit state, not an independent proof of the user's exact screen.

## Optional 1000 timing

`secondaryComponentsMintedTimestamp` is tracked separately. Only a newer secondary snapshot replaces the optional start; missing/private/malformed/future/invalid optional values are tolerated. Network failures leave the latest stored optional snapshot available; freshness guards still apply when resolving a subsequent 204 response.

A Transitory start is considered only when it is valid, not after its secondary mint time, not earlier than the character's start, and not later than the primary mint time. It can advance a same-activity timer without waiting for 204's start value to catch up. An accepted timer does not roll back merely because optional data disappears. Confirmed orbit clears the accepted optional start while retaining the freshness watermark.

Transitory has no activity hash and comes from a non-authoritative system separate from PGCR. Even a fresh primary timestamp cannot prove both sources describe the same run. It must never be used alone to identify an activity, infer orbit or terminate a run.

Official schema references: [CharacterActivities](https://bungie-net.github.io/multi/schema_Destiny-Entities-Characters-DestinyCharacterActivitiesComponent.html), [primary/secondary freshness](https://bungie-net.github.io/multi/schema_Destiny-Responses-DestinyProfileResponse.html), [Transitory current activity](https://bungie-net.github.io/multi/schema_Destiny-Components-Profiles-DestinyProfileTransitoryCurrentActivity.html). These support source/field semantics, not a promise of a fixed Bungie refresh delay.

## History and initialization

A history round scans characters and pages until empty data or entries older than the 17:00 UTC cutoff. It filters by activity start time, supplements incomplete modes from the classification cache, retains Raid/Dungeon/Strike/Lost Sector records, and sorts by period with instance ID as a stable tie-breaker. Full-record comparison accepts late arrivals, corrected completion/duration and reset removals.

After initialization, current, history and Transitory futures are joined in one cancellable task; history writes only its own field and cannot overwrite newer current activity. Profile reset aborts the task and recreates timing state. **However, initial publication still waits for successful current activity and history together.** A history error causes another initialization attempt after five seconds; steady-state polling and optional timing have not started yet. This is a confirmed local startup bottleneck, not a Bungie freshness explanation.

The frontend counter and notification delivery are separate: history corrections can change counts without generating notifications. See [CUSTOM_FEATURES](CUSTOM_FEATURES.md) for the confirmed same-ID correction and late-older notification gaps. A clear must never force orbit or stop a subsequent run.

## Metadata caches and confirmed limitations

- Full `ActivityInfo` values are cached in memory by hash. An individual manifest definition supplies name/modes/image; this is not a downloaded manifest database.
- `activity-mode-cache.json` persists only learned raid/dungeon modes. It stores classification, never live timing. Read/parse failures fall back to an empty cache; write failures are ignored. Writes are synchronous, and no manifest-version/TTL invalidation or size bound is implemented.
- Raid/dungeon `activityTypeHash` supplements missing/generic mode data. History classification makes no per-item manifest request. With incomplete modes and an unknown hash it drops the record until another path learns that definition. The details window can fetch metadata for retained historical records, so the whole UI is not “manifest-request-free.”

**Confirmed manifest failure path:** For a nonzero activity hash, `update_current` waits for metadata under the shared activity-source mutex. Only empty names and ResponseMissing become absent metadata; other network/API/deserialization errors return before publishing the new activity. Hash 0 itself skips this lookup, but a preceding slow lookup delays the next status request. A definition with an empty name is still cached as a successful response, so subsequent refresh attempts reuse it indefinitely within the session. The existing helper test checks whether a retry is requested, not whether the source cache actually refetches.

**Required future behavior:** accepted identity/orbit/timing must not wait for metadata; metadata must recover with bounded/deduplicated work and no stale completion overwriting a newer activity/profile. Use already available classification where safe; do not label an unknown activity as a raid/dungeon. This is pending work, not a description of 1.1.4.

## HTTP behavior and remaining risks

Implemented: one session-wide reqwest client, connection reuse, 15-second request timeout, custom in-memory cookie name/value replay without reqwest's cookie-store dependency stack. Affinity itself adds no requests; it does not establish a cause or guaranteed cure for backend inconsistency. The simplified cookie map is not a full expiry/domain/path-aware cookie jar.

Bungie errors expose `ThrottleSeconds`, but callers currently keep their fixed retry pauses instead of honoring that duration. Error state is shared by current/history paths, so a successful status update can clear a history error. After successful initial load, the overlay can continue showing last good data without making subsequent errors obvious. These are follow-up robustness risks, not newly measured live regressions.

## Rules for endpoint changes

Document purpose, call frequency, caching, privacy/failure fallback, total request impact and why the endpoint stays within the safety boundary. Diagnose the complete API → retrieval → processing → state → overlay chain: Bungie freshness, request duration, metadata/locks, state acceptance and UI separately. Improve speed beyond original through bounded parallelism, deduplication and cache reuse; do not assume more requests produce fresher server data. Include startup/retry/page traffic, not only steady-state polling, in before/after budgets.

Keep status/optional/history paths cancellable within app lifetime, including initialization, sleeps, lock waits and one-off IPC profile/search/metadata requests. Hiding/minimizing main UI must not duplicate or restart these paths. Full Exit must prevent new requests and cancel in-flight app futures without waiting for the 15-second timeout; an already sent request cannot be recalled from Bungie's server. No service or custom work after completed exit. These are requirements; central shutdown coordination is not yet implemented.
