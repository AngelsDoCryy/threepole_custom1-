# Threepole Custom — Performance

Audited runtime baseline: 1.1.4, 2026-10-11; development goals aligned 2026-10-13; package 1 source updated 2026-10-14. **Target: measurably exceed original Threepole's speed and responsiveness at preferably equal or lower resource use**, retaining all Custom features. Original 1.1.2 is the permanent technical baseline and minimum standard, not the final goal. Neither a speed advantage nor resource parity/savings has yet been established by a comparable benchmark.

Prioritize first activity detection, activity changes, orbit/timer stop, repeated same-activity restarts, timely clears/notifications, recognized-clear-to-overlay delay and cold/warm startup. Optimize the full Bungie API → retrieval → processing → state → overlay chain. Prefer bounded parallelism, cache reuse, efficient updates and removal of local blocking; more polling, substantially higher system load or riskier game access are not acceptable shortcuts. PROJECT_CONTEXT defines priorities and prior user approval for even a small resource tradeoff.

**Binding gate, 2026-10-14:** account safety comes first, followed by unaffected Destiny performance, before correctness, display speed, app resource savings and convenience. Faster timer updates never justify a relevant FPS or 1%-low loss, worse frametimes, microstutter or input latency. Include CPU/GPU peaks and background work in every proposal. Neither game-performance neutrality nor improvement is established by source inspection or a green build.

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
| Overlay targeting | Package 1 replaces the earlier CloseHandle leak fix with window-only detection; removes the unbounded identity cache and skips unchanged geometry | Metadata observed on one installation; runtime CPU/RAM and cross-configuration reliability remain unmeasured |

No new independent background service was introduced. Current X closes the UI while tracking/overlay continue; simple tray click reopens it. This behavior is explicitly accepted and must be preserved. Complete Exit needs Windows verification; lack of a central shutdown framework alone is not proof of a defect. See [PROJECT_CONTEXT](PROJECT_CONTEXT.md).

## Confirmed bottlenecks and unmeasured risks

Confirmed from code: initial history gates first usable data; manifest errors return before committing activity changes; successful empty-name definitions stay cached. Shared source locks span metadata network waits. These local paths deserve focused fixes even before the old restart complaint is reproduced.

Further measurement targets: repeated full daily-history pagination, detail-window metadata contention, full player-data/history cloning and event payloads, synchronous cache writes, cache invalidation/size, details-window timer activity, and stale-data/error presentation. Their existence is observable in code; their real performance cost has not been measured. Do not present them as the proven cause of the historical 2–3 minute delay.

Package 1 removes the former unbounded HWND/name cache and redundant geometry updates, and explicitly requests hide on invalid detection/geometry/placement. Its source-level behavior and remaining acceptance are recorded below. Hidden popup queues have no bound; name-hide and existing popup timeouts remain scheduled. Details uses a 30 Hz timer plus a 30-second age refresh without an explicit visibility pause while its window exists; X destroys that webview. Their cost needs measurement; they do not justify changing notification semantics or window management silently. Retaining a hidden main webview is no longer a planned optimization; preserve current destruction/recreation and assess unnecessary UI work separately if measurements justify it.

The overlay's milliseconds option changes drawing frequency from about 2 Hz to 30 Hz; it does not change API polling. Name/auto-hide, counters, notifications and local icons must not add requests or materially increase overhead.

## Package 1 — Process-free overlay recognition (2026-10-14)

Compared source: [original 1.1.2, commit 773638cb47ce94d98c11fce141d0bb1a34917c69](https://github.com/des-sh/threepole/blob/773638cb47ce94d98c11fce141d0bb1a34917c69/src-tauri/src/pollers/overlay.rs) and Custom before this package, `628d2ae3733c63f50137549a558b133ab72ef959`. Both used foreground PID/process-name inspection, an unbounded HWND/name cache and geometry updates on every accepted tick; Custom already closed process handles and tolerated placement failures. This package removes the process path entirely.

**Live metadata evidence:** After the user opened Destiny 2 for a requested borderless-window check, a read-only Win32 window query observed a visible, non-minimized window titled `Destiny 2`, class `Tiger D3D Window`, rectangle `0,0–2560,1440`, DPI 120 (125%). A second hidden window had the same title but class `Tiger Input Window` and off-screen geometry. This supports exact title **and** class matching and rejects title-only detection. The sample was taken with another window foreground; it is not an installed-build overlay or Alt-Tab test. No process/PID access was used.

**Implemented:** Fresh foreground/visibility/minimized/class/title/rectangle checks every existing 200 ms pause; exact UTF-16 matching with fixed buffers, class first; reject failed/truncated reads and empty/inverted/overflowing rectangles. Recheck foreground after reading geometry and foreground/identity after placement. Hide on uncertainty, including notification-state query failure or exclusive-fullscreen guard. The existing own-overlay-foreground case is preserved because the pinned Tao show path can activate that window. Only one previous game HWND is retained as a candidate, never as proof of identity: title/class, visibility and geometry are checked again. Missing previous target, changed metadata or failed lookup hides the overlay; other foreground windows replace the candidate. No HWND-to-identity cache survives, so recreated/reused handles are checked again.

Alongside that single candidate, only the last successfully applied rectangle and overlay scale factor are retained. Stable geometry/DPI skips both setters; resize alone skips position. Moving reasserts physical size, and a changed overlay scale factor reapplies both, because Windows/Tao can reposition/resize on [DPI changes](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged). The existing Tauri scale-factor getter is used only for a valid candidate; no new process query or DPI mode is added. Failed placement/scale lookup or lost target clears the rectangle for a full retry. Existing DPI awareness and physical coordinates, including negative monitor coordinates, remain in use without manual scale conversion. Visibility events still repeat so frontend error recovery works; its existing serialization avoids redundant show/hide operations.

**Cost and scope:** Zero additional Bungie requests; unchanged 204/1000/history and 200 ms overlay cadences. Constant-size buffers, one candidate HWND and geometry replace unbounded retained strings. At a stable foreground window, up to five position and five size calls per second become zero after successful placement. Fresh window metadata queries replace cache hits, so net CPU/RAM savings are an expectation to measure, not a benchmark result. No process-based before/after probe may be run against Destiny to measure this. No new dependencies, services, processes, timer, task, user setting or gameplay interaction; timer, clears, notifications and window/tray/exit logic are unchanged.

**Regression coverage and limits:** Eleven new Rust cases cover observed metadata versus input/browser windows, partial titles, failed/truncated reads, own-overlay versus other/absent foreground selection, physical/negative coordinates, invalid geometry, unchanged placement, movement versus resize, DPI-only changes/invalid scale, lost target/re-entry and placement failure/retry. Existing five frontend and 37 Rust tests remain part of CI. These are logic regressions, not real Windows focus/DPI/API-failure tests. Check the commit's Windows workflow for compiled test/build results.

**Manual acceptance still required:** Test the MSI with Alt-Tab both ways (including rapid switching), minimize/restore, overlay off/on, game exit/restart and repeated window recreation, moving/resizing and mixed-DPI/multiple monitors (also left/above the primary), borderless mode, tray reopen and all display settings. Confirm no stale overlay after lost focus/window, and that name/timer/clears/popups continue normally. Sample detection latency around the 200 ms cadence plus scheduling/UI delay; no hard 200 ms maximum is promised. Compare app CPU/RAM and safely observed FPS/frametimes with the recorded baseline, including long sessions.

Window title/class are observed metadata, not Bungie-guaranteed identifiers or executable authentication. A changed/localized title/class can cause a safe false negative; an unrelated window copying both can match. Other installations/versions and exclusive fullscreen are not validated or newly supported. If practical tests fail, investigate actual window metadata or propose explicit window selection; never silently broaden to title substrings or restore process inspection.

## Request-volume guardrail

204 keeps a two-second pause, optional 1000 a ten-second pause, history a ten-second pause per round. Response/processing time is additional. The optional path contributes at most about six requests/minute in steady state. Total history traffic depends on characters and pages, and metadata/startup retries add more; see [DESTINY_API](DESTINY_API.md) for the endpoint budget.

No frequency increase without a measured limiting path and a quantified before/after budget. Reduced startup waiting must not be achieved by duplicate pollers, uncontrolled retries or extra per-history-item requests. Every roadmap change must assess cancellation, profile changes and independently disabled features.

## Diagnostics — proposed, not implemented

If code-level tests do not explain remaining latency, add low-overhead, opt-in diagnostics inside app lifetime only: request start/end and outcome, component generation times, accepted/rejected status/timing, metadata wait/cache hit, history arrival, frontend receipt and visible transition. Redact API keys, cookies and profile identifiers. Bound buffers/logs; no external collector, service, automatic polling increase or logging after exit.

Prefer a fixed-capacity in-memory ring with explicit local export, disabled by default; no full API bodies or per-frame logs. Correlate a local profile generation, response/event sequence, pseudonymous character/run identifiers, raw and accepted starts/hash, history instance arrival, popup creation versus delivery, rejection reason and readiness/cache state. Use monotonic durations within each process and wall-clock server timestamps separately; do not subtract unsynchronized Rust/WebView clocks. Reuse existing requests/events, never query Destiny or add Bungie calls for diagnostics. Buffer size, export I/O and enabled/disabled overhead must be assessed before a separately authorized implementation.

Separate Bungie data age from local request, lock, state and rendering delay. Record request counts and cold/warm CPU, memory and handle behavior before and after fixes. Do not infer affinity/backend routing causes from timing alone.

## Permanent original-versus-Custom comparison method

Source comparison refreshed 2026-10-14: original **1.1.2 / `773638cb47ce94d98c11fce141d0bb1a34917c69`**, Custom **1.1.4 / `42bd50c8f473d7c891e22efff8b2d04dc46945a0`**. Inspected original [player-data scheduling/state](https://github.com/des-sh/threepole/blob/773638cb47ce94d98c11fce141d0bb1a34917c69/src-tauri/src/pollers/playerdata.rs) and [overlay](https://github.com/des-sh/threepole/blob/773638cb47ce94d98c11fce141d0bb1a34917c69/src/overlay/overlay.ts) alongside Custom:

| Original versus current Custom | Source-level result, not measured speed |
| --- | --- |
| Original serially substitutes a history round for a current update; Custom joins independent loops after initialization | Removes that steady-state network gate. Both still gate first publication on history and await current metadata. Shared Custom cache waits remain. |
| Original rejects backward activity starts and does not use optional Transitory timing | Custom freshness guards accept newer backward/epoch orbit and allow same-hash timing advances. Already unit-covered, not proof the old live complaint disappeared. |
| Original updates history only when its newest record advances | Custom replaces corrected/older/reset history; both overlays still use only the first row for notifications. Backend correction does not fix frontend notification eligibility. |
| Original registers update listeners after initial snapshots | Custom registers listeners first and protects against stale initial reads. Both render elapsed time locally at 2/30 Hz; this is not a server-refresh mechanism. |

The original loop performs four current requests plus one history round per steady five iterations after its initial cycle, each with a two-second pause. Ignoring all I/O, this is about 24 current requests/minute versus Custom's current ceiling of 30, plus Custom's optional six 1000 requests. History pagination and I/O further change actual counts. The existing Custom architecture is therefore not already proved request-equivalent to original; proposed optimizations must compare against both baselines without raising the present pauses. Package 1 changed window targeting only and cannot itself fix API/timer delays.

Use [des-sh/threepole](https://github.com/des-sh/threepole) as a continuing source-code reference. Before each relevant optimization compare original and Custom retrieval, scheduling, locks/caches, state publication and rendering paths; record upstream commit/version and Custom commit. Identify necessary feature cost versus avoidable local delay, and separate the original 1.1.2 base from newer upstream changes. Do not claim a fresh source comparison when only historical knowledge or measurements are available. Reference code never overrides the safety boundary.

Use identifiable original and Custom builds under matching Windows, hardware, game/display, activity, profile and feature settings. Compare shared features on equal settings and separately report the cost of Custom-only options. Prefer repeated alternating runs rather than concurrent apps; if concurrent, disclose extra API/load effects. Record cache state, test order, trial count and long-session duration; no arbitrary speed threshold is claimed without a baseline.

Measure startup/loading and each detection/display stage, CPU averages and short peaks, app plus owned-WebView RAM and growth over time, network bytes/API counts, UI/timer work and recovery. Include visible, minimized and tray states, long sessions and repeated Dungeon **and Raid** restarts. Report typical and slow-tail latency (for example median/p95 when sample count supports it), variability and failures, not only the best run. Separate game action → available Bungie data from available response → accepted state → visible overlay; without enough timestamps, label the missing interval unknown rather than assigning it to Bungie or local code.

Assess Destiny FPS, 1% lows, frametimes, microstutter and input latency with already permitted measurement sources; label subjective user observations separately from numerical measurements. Compare against no Threepole as well as original under matching conditions: equality with original alone does not establish no game impact. Measure aggregate app/WebView CPU, GPU and RAM without querying the Destiny process. Measurement itself must not introduce direct Destiny process access, injection, game/input hooks, extra services or unapproved tools. Original contains the prohibited process query, so its role as reference does not authorize running that path for benchmarking; use existing permitted measurements or a separately approved compliant baseline, clearly identifying any modification. If safe game-performance evidence is unavailable, explicitly leave that criterion unverified. Distinguish code-level expectations, automated/mock results and practical Windows/Destiny measurements; never promote one into the other.

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
| Ambiguous identity/freshness | Multiple characters with tied dates/epoch orbit, equal-mint conflicting status, clock skew, late join, 1000 before/after 204, orbit consuming optional timing; unknown identity never authorizes a timer stop |
| Completed but still in activity | Distinguish history completion, optional start/end and actual departure; delayed prior-run end/clear must never freeze a new same-hash run |
| Hidden popup then new run | Record history arrival versus actual popup display; preserve new timer, profile isolation and defined notification settings |
| Slow/failed history, manifest, offline/recovery, throttling | Partial readiness, bounded retries, recovery, visible stale/error state; use mocked failures where live reproduction is impractical |
| Empty/corrupt cache; unknown activity | Conservative classification and later enrichment, no request storm |
| Settings and overlay focus | Independent timer/name/auto-hide/counter/notifications; milliseconds on/off; hide/show, alt-tab, resize and fullscreen behavior |
| Windows lifecycle | Preserve current main X and simple tray-click reopening, normal minimize and auxiliary-window behavior; tracking/overlay continue in tray; verify actual Exit during startup/HTTP/profile changes stops app-owned work, releases pipe/windows and permits restart; propose minimal fixes only for reproduced defects |
| Installation/icons | MSI install/upgrade, custom/original separation, updater disabled, Eager Edge app/tray/installer/popup/clear surfaces |

There is no OAuth flow to time as a login: measure profile loading. Exclusive fullscreen currently hides the separate overlay; do not propose game hooks to change that.

## Completion criteria

A relevant performance/runtime change requires focused regression checks, appropriate build/MSI validation, understood request impact, unchanged safety/game-performance boundaries and live evidence where Bungie/Destiny behavior is involved. Report what is proved and what remains pending. A green build alone cannot close a live-timing issue. [CODEX_HANDOFF](CODEX_HANDOFF.md) records build #25's 48 Rust / five overlay test baseline and its integration-test gaps. The analysis/documentation order adds no installed-app or game-performance measurements.
