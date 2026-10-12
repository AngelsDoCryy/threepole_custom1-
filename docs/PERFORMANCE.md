# Threepole Custom — Performance

Audited baseline: 1.1.4, 2026-10-11. Preserve original Threepole's startup speed, low overhead and responsiveness while adding independent optional features. Performance parity is a target, not a completed benchmark.

## Earlier user observations — still require live validation

On an earlier custom build, original Threepole detected orbit sooner; custom kept the previous activity/timer visible longer. A clear notification could arrive before the live overlay changed. That ordering alone is valid because history and live state are separate sources; corruption of a later run would be a bug.

On an older build, the first Dungeon was recognized after about 30 seconds, while an immediate same-Dungeon restart sometimes took 2–3 minutes. The user had not yet tested 1.1.4. Neither persistence nor resolution of that delay is proved for 1.1.4.

## Implemented optimizations to preserve

| Area | Implemented change | Practical limit |
| --- | --- | --- |
| Metadata | Session activity cache and persistent learned raid/dungeon modes; no per-history-item classification request | Unknown incomplete history can be omitted; metadata still blocks current updates |
| Icon metadata | Removed activity-type definition request, icon fields and activity-icon setting | Eager Edge clear replacement is pending and should use a local asset |
| HTTP | Shared client, connection reuse, lightweight cookie affinity, 15-second timeout | No guaranteed backend freshness; throttle duration is not honored yet |
| Polling | Separate steady-state current/history futures; independent optional 1000 in 1.1.4 | Initialization still waits for history; shared metadata locks can contend |
| Timing | Independent freshness guards, newer backward/epoch status acceptance, clear optional start on orbit | Transitory cannot establish run identity; character selection needs live coverage |
| Recovery | Initial failures retry; missing metadata can request refresh; recoverable window positioning/sizing and visibility transitions | Cached empty names defeat source refetch; some later errors remain unobtrusive |
| History | Full-record replacement, stable ordering, late/corrected records and reset handling | Frontend notification logic still misses corrections and older arrivals |
| Overlay | Serialized/retryable show/hide, listener-before-snapshot initialization, stopped hidden/disabled timer interval | Details window retains a separate 30 Hz interval; actual CPU/memory not benchmarked |
| Windows handles | CloseHandle after foreground executable lookup, including lookup failure | Fixing the leak does not make process access compliant with the safety requirement |

No new independent background service was introduced. Current UI close nevertheless leaves tray/polling active; this is a lifecycle conflict, not an optimization or accepted exception. See [PROJECT_CONTEXT](PROJECT_CONTEXT.md).

## Confirmed bottlenecks and unmeasured risks

Confirmed from code: initial history gates first usable data; manifest errors return before committing activity changes; successful empty-name definitions stay cached. Shared source locks span metadata network waits. These local paths deserve focused fixes even before the old restart complaint is reproduced.

Further measurement targets: repeated full daily-history pagination, detail-window metadata contention, full player-data/history cloning and event payloads, synchronous cache writes, cache invalidation/size, foreground handle reuse, details-window timer activity, and stale-data/error presentation. Their existence is observable in code; their real performance cost has not been measured. Do not present them as the proven cause of the historical 2–3 minute delay.

The overlay's milliseconds option changes drawing frequency from about 2 Hz to 30 Hz; it does not change API polling. Name/auto-hide, counters, notifications and local icons must not add requests or materially increase overhead.

## Request-volume guardrail

204 keeps a two-second pause, optional 1000 a ten-second pause, history a ten-second pause per round. Response/processing time is additional. The optional path contributes at most about six requests/minute in steady state. Total history traffic depends on characters and pages, and metadata/startup retries add more; see [DESTINY_API](DESTINY_API.md) for the endpoint budget.

No frequency increase without a measured limiting path and a quantified before/after budget. Reduced startup waiting must not be achieved by duplicate pollers, uncontrolled retries or extra per-history-item requests. Every roadmap change must assess cancellation, profile changes and independently disabled features.

## Diagnostics — proposed, not implemented

If code-level tests do not explain remaining latency, add low-overhead, opt-in diagnostics inside app lifetime only: request start/end and outcome, component generation times, accepted/rejected status/timing, metadata wait/cache hit, history arrival, frontend receipt and visible transition. Redact API keys, cookies and profile identifiers. Bound buffers/logs; no external collector, service, automatic polling increase or logging after exit.

Separate Bungie data age from local request, lock, state and rendering delay. Record request counts and cold/warm CPU, memory and handle behavior before and after fixes. Do not infer affinity/backend routing causes from timing alone.

## Repeatable validation matrix — all live acceptance remains open

Record tested commit/MSI, Windows/display mode, profile/character count, settings, cache cold/warm state and repeated-trial results. Compare original and custom using the same activities/conditions, identifying concurrent-app API load if both run together.

| Scenario | Record / acceptance focus |
| --- | --- |
| Cold and warm launch | Launch to visible UI and first usable profile/activity; history/metadata must not gate accepted status after planned fixes |
| First Raid/Dungeon entry | Identity detection separately from start-time/timer adoption |
| Orbit return | 204 freshness, accepted hash 0, timer/name/counter hide latency, optional-state discard |
| Immediate same-Dungeon restart x3 | New start/name reappearance; no 2–3 minute local stall; distinguish API delay |
| Switch to another activity/variant | Correct identity, separate per-hash clear count, no stale metadata/timing carryover |
| Late join / character or profile switch | Conservative timing association, correct character selection, no old task/state/notification leak |
| Clear before orbit / after next run | Notification and counter update without stopping the active/new run |
| Corrected same-ID / late older / multiple clears | Exactly-once eligible notification after a baseline, stable counts and no startup flood |
| Reset at 17:00 UTC | Cutoff by activity start, empty history replacement, day-boundary run semantics |
| Private/missing/malformed/old/future 1000 | 204 continues; optional disappearance does not roll timer back |
| Slow/failed history, manifest, offline/recovery, throttling | Partial readiness, bounded retries, recovery, visible stale/error state; use mocked failures where live reproduction is impractical |
| Empty/corrupt cache; unknown activity | Conservative classification and later enrichment, no request storm |
| Settings and overlay focus | Independent timer/name/auto-hide/counter/notifications; milliseconds on/off; hide/show, alt-tab, resize and fullscreen behavior |
| Windows lifecycle | UI close versus deliberate tray operation, overlay disabled, profile reset, actual Exit; no remaining app-owned work/requests after close |
| Installation/icons | MSI install/upgrade, custom/original separation, updater disabled, Eager Edge app/tray/installer/popup/clear surfaces |

There is no OAuth flow to time as a login: measure profile loading. Exclusive fullscreen currently hides the separate overlay; do not propose game hooks to change that.

## Completion criteria

A relevant performance/runtime change requires focused regression checks, appropriate build/MSI validation, understood request impact, unchanged safety boundary and live evidence where Bungie/Destiny behavior is involved. Report what is proved and what remains pending. A green build alone cannot close a live-timing issue. [CODEX_HANDOFF](CODEX_HANDOFF.md) records the 37 Rust / five overlay test baseline and its integration-test gaps.
