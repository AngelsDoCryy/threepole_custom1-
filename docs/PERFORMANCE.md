# Threepole Custom — Performance

Audited runtime baseline: 1.1.4, 2026-10-11; development goals aligned 2026-10-13. **Target: measurably exceed original Threepole's speed and responsiveness at preferably equal or lower resource use**, retaining all Custom features. Original 1.1.2 is the permanent technical baseline and minimum standard, not the final goal. Neither a speed advantage nor resource parity/savings has yet been established by a comparable benchmark.

Prioritize first activity detection, activity changes, orbit/timer stop, repeated same-activity restarts, timely clears/notifications, recognized-clear-to-overlay delay and cold/warm startup. Optimize the full Bungie API → retrieval → processing → state → overlay chain. Prefer bounded parallelism, cache reuse, efficient updates and removal of local blocking; more polling, substantially higher system load or riskier game access are not acceptable shortcuts. PROJECT_CONTEXT defines priorities and prior user approval for even a small resource tradeoff.

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

No new independent background service was introduced. Current X closes the UI while tracking/overlay continue; simple tray click reopens it. This behavior is explicitly accepted and must be preserved. Complete Exit needs Windows verification; lack of a central shutdown framework alone is not proof of a defect. See [PROJECT_CONTEXT](PROJECT_CONTEXT.md).

## Confirmed bottlenecks and unmeasured risks

Confirmed from code: initial history gates first usable data; manifest errors return before committing activity changes; successful empty-name definitions stay cached. Shared source locks span metadata network waits. These local paths deserve focused fixes even before the old restart complaint is reproduced.

Further measurement targets: repeated full daily-history pagination, detail-window metadata contention, full player-data/history cloning and event payloads, synchronous cache writes, cache invalidation/size, foreground handle reuse, details-window timer activity, and stale-data/error presentation. Their existence is observable in code; their real performance cost has not been measured. Do not present them as the proven cause of the historical 2–3 minute delay.

The foreground HWND/name cache is unbounded and does not invalidate reused handles; it must not survive as a substitute for the forbidden process query. Overlay geometry is reapplied every 200 ms even when unchanged, and invalid geometry does not explicitly hide a previously visible overlay. Hidden popup queues have no bound; name-hide and existing popup timeouts remain scheduled. Details uses a 30 Hz timer plus a 30-second age refresh without an explicit visibility pause while its window exists; X destroys that webview. Their cost needs measurement; they do not justify changing notification semantics or window management silently. Retaining a hidden main webview is no longer a planned optimization; preserve current destruction/recreation and assess unnecessary UI work separately if measurements justify it.

The overlay's milliseconds option changes drawing frequency from about 2 Hz to 30 Hz; it does not change API polling. Name/auto-hide, counters, notifications and local icons must not add requests or materially increase overhead.

## Request-volume guardrail

204 keeps a two-second pause, optional 1000 a ten-second pause, history a ten-second pause per round. Response/processing time is additional. The optional path contributes at most about six requests/minute in steady state. Total history traffic depends on characters and pages, and metadata/startup retries add more; see [DESTINY_API](DESTINY_API.md) for the endpoint budget.

No frequency increase without a measured limiting path and a quantified before/after budget. Reduced startup waiting must not be achieved by duplicate pollers, uncontrolled retries or extra per-history-item requests. Every roadmap change must assess cancellation, profile changes and independently disabled features.

## Diagnostics — proposed, not implemented

If code-level tests do not explain remaining latency, add low-overhead, opt-in diagnostics inside app lifetime only: request start/end and outcome, component generation times, accepted/rejected status/timing, metadata wait/cache hit, history arrival, frontend receipt and visible transition. Redact API keys, cookies and profile identifiers. Bound buffers/logs; no external collector, service, automatic polling increase or logging after exit.

Separate Bungie data age from local request, lock, state and rendering delay. Record request counts and cold/warm CPU, memory and handle behavior before and after fixes. Do not infer affinity/backend routing causes from timing alone.

## Permanent original-versus-Custom comparison method

Use [des-sh/threepole](https://github.com/des-sh/threepole) as a continuing source-code reference. Before each relevant optimization compare original and Custom retrieval, scheduling, locks/caches, state publication and rendering paths; record upstream commit/version and Custom commit. Identify necessary feature cost versus avoidable local delay, and separate the original 1.1.2 base from newer upstream changes. Do not claim a fresh source comparison when only historical knowledge or measurements are available. Reference code never overrides the safety boundary.

Use identifiable original and Custom builds under matching Windows, hardware, game/display, activity, profile and feature settings. Compare shared features on equal settings and separately report the cost of Custom-only options. Prefer repeated alternating runs rather than concurrent apps; if concurrent, disclose extra API/load effects. Record cache state, test order, trial count and long-session duration; no arbitrary speed threshold is claimed without a baseline.

Measure startup/loading and each detection/display stage, CPU averages and short peaks, app plus owned-WebView RAM and growth over time, network bytes/API counts, UI/timer work and recovery. Include visible, minimized and tray states, long sessions and repeated Dungeon **and Raid** restarts. Report typical and slow-tail latency (for example median/p95 when sample count supports it), variability and failures, not only the best run. Separate game action → available Bungie data from available response → accepted state → visible overlay; without enough timestamps, label the missing interval unknown rather than assigning it to Bungie or local code.

Assess Destiny FPS, frametimes and microstutter with user observations or already permitted measurement sources. Measurement itself must not introduce direct Destiny process access, injection, game/input hooks, extra services or unapproved tools. If safe frametime evidence is unavailable, explicitly leave that criterion unverified. Distinguish code-level expectations, automated/mock results and practical Windows/Destiny measurements; never promote one into the other.

## Repeatable validation matrix — all live acceptance remains open

Record tested commit/MSI, Windows/display mode, profile/character count, settings, cache cold/warm state and repeated-trial results. Compare original and custom using the same activities/conditions, identifying concurrent-app API load if both run together.

| Scenario | Record / acceptance focus |
| --- | --- |
| Cold and warm launch | Launch to visible UI and first usable profile/activity; history/metadata must not gate accepted status after planned fixes |
| First Raid/Dungeon entry | Identity detection separately from start-time/timer adoption |
| Orbit return | 204 freshness, accepted hash 0, timer/name/counter hide latency, optional-state discard |
| Immediate same-Dungeon restart x3 | New start/name reappearance; no 2–3 minute local stall; distinguish API delay |
| Repeated Raid restarts and long sessions | Same-run identity correctness, stable CPU/RAM/cache/queue sizes, no accumulated pollers/listeners, FPS/frametime/microstutter observations versus original |
| Switch to another activity/variant | Correct identity, separate per-hash clear count, no stale metadata/timing carryover |
| Late join / character or profile switch | Conservative timing association, correct character selection, no old task/state/notification leak |
| Clear before orbit / after next run | Notification and counter update without stopping the active/new run |
| Corrected same-ID / late older / multiple clears | Exactly-once eligible notification after a baseline, stable counts and no startup flood |
| Reset at 17:00 UTC | Cutoff by activity start, empty history replacement, day-boundary run semantics |
| Private/missing/malformed/old/future 1000 | 204 continues; optional disappearance does not roll timer back |
| Slow/failed history, manifest, offline/recovery, throttling | Partial readiness, bounded retries, recovery, visible stale/error state; use mocked failures where live reproduction is impractical |
| Empty/corrupt cache; unknown activity | Conservative classification and later enrichment, no request storm |
| Settings and overlay focus | Independent timer/name/auto-hide/counter/notifications; milliseconds on/off; hide/show, alt-tab, resize and fullscreen behavior |
| Windows lifecycle | Preserve current main X and simple tray-click reopening, normal minimize and auxiliary-window behavior; tracking/overlay continue in tray; verify actual Exit during startup/HTTP/profile changes stops app-owned work, releases pipe/windows and permits restart; propose minimal fixes only for reproduced defects |
| Installation/icons | MSI install/upgrade, custom/original separation, updater disabled, Eager Edge app/tray/installer/popup/clear surfaces |

There is no OAuth flow to time as a login: measure profile loading. Exclusive fullscreen currently hides the separate overlay; do not propose game hooks to change that.

## Completion criteria

A relevant performance/runtime change requires focused regression checks, appropriate build/MSI validation, understood request impact, unchanged safety boundary and live evidence where Bungie/Destiny behavior is involved. Report what is proved and what remains pending. A green build alone cannot close a live-timing issue. [CODEX_HANDOFF](CODEX_HANDOFF.md) records the 37 Rust / five overlay test baseline and its integration-test gaps.
