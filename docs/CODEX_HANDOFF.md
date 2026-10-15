# Threepole Custom — Codex handoff

Aligned 2026-10-14. Entry point for retained ChatGPT/Work decisions, Codex takeover and permanent development rules; not a claim that every requirement is satisfied.

## Verified baseline

- Repository: `AngelsDoCryy/threepole_custom1-`; upstream: `des-sh/threepole` 1.1.2; custom: **1.1.4**.
- Branch: `threepole-custom-dev`; [draft PR #2](https://github.com/AngelsDoCryy/threepole_custom1-/pull/2) targets `main`; no merge without explicit instruction.
- Before package 1, last implementation: `8285cc406f496e02911d469eb74c4372147ad867`; initial audited head: `fe083270b8a733239507f06bafde4048a879bfd7`. The eight intervening commits only changed documentation. The prior ten-file cleanup was committed/pushed as `bb3f21f1877cb16f703888b9afb45493d28c4b8d`; subsequent permanent rules were saved as noted below.
- Permanent goals were saved in `6dd79726c1ddaf202dbe408b91b58672216a422a`; window/tray clarification in `628d2ae3733c63f50137549a558b133ab72ef959`. The user subsequently authorized package 1 only: process-free overlay recognition, focused tests, documentation, commit/push and a CI MSI testbuild. This branch now contains that implementation; inspect its Git commit and corresponding workflow. No version/CI change, window/tray/exit redesign, other performance package, merge or removal of Draft status accompanies it.

## Development branch migration — 2026-10-14

The permanent development branch is now `threepole-custom-dev`, renamed from `optimize-speed-cache-icon` through GitHub's rename operation. The entire history through `a2e67edb094dcd5a841cf48bed17be063489bc4b` is retained. GitHub automatically closed unmerged [PR #1](https://github.com/AngelsDoCryy/threepole_custom1-/pull/1) when its head branch was renamed; [Draft PR #2](https://github.com/AngelsDoCryy/threepole_custom1-/pull/2) continues it against unchanged `main`. Both PR descriptions retain the connection. Do not recreate the old development branch.

The local branch tracks `origin/threepole-custom-dev`; the repository URL and GitHub default branch `main` remain unchanged. AGENTS directs future sessions to the permanent development checkout. Only the Windows workflow's push-branch filter changes in this migration; build steps, application code and version remain unchanged. Verify the migration commit's automatically triggered Windows run before declaring Actions healthy.

The last implementation before the rename passed [Windows build #24](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38478039050) on `a2e67ed`: 48 Rust tests, five frontend tests and MSI packaging. This is the retained pre-migration validation baseline, not a new live Destiny test.

Migration completed at `42bd50c8f473d7c891e22efff8b2d04dc46945a0`; [Windows build #25](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38484093088) succeeded on that commit (48 Rust tests, five frontend tests, MSI). Its [MSI artifact](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38484093088/artifacts/11858899185) is the current analysis baseline. Local branch/upstream and open Draft PR #2 were rechecked at the same commit before this documentation update. Do not repeat the rename; package 1 still needs full Destiny live acceptance.

## Activity/timer investigation — 2026-10-14

The user authorizes analysis and existing-document commit/push only. PROJECT_CONTEXT now gives unaffected Destiny FPS, 1% lows, frametimes and input latency second priority immediately after account safety, ahead of correctness/display speed; no speed gain permits weakening either boundary. AGENTS routes subsequent sessions to this binding order. No new runtime or diagnostics implementation is authorized.

Compared original 1.1.2 `773638cb47ce94d98c11fce141d0bb1a34917c69` with Custom 1.1.4 `42bd50c`; PERFORMANCE owns source differences and unmeasured live outcomes. DESTINY_API owns the confirmed metadata/initial-history gates, delayed consumption of optional timing, conditional multi-character/freshness risks and unused API mode/end-time fields. CUSTOM_FEATURES separates hidden-popup delivery and missed notifications from live timer ownership. Same-hash restart/fresh-orbit guards already exist; the historical multi-minute delay is not proved current or resolved. ROADMAP recommends separate manifest, initial-readiness and notification fixes, followed only by evidence-led timing changes. Never stop a new run because an older clear arrived. No application, dependency, CI, tray or shutdown change accompanies this documentation analysis.

## P1.1 implementation — 2026-10-15

The user separately authorized manifest-independent state, focused tests, documentation, commit/push and a Windows MSI testbuild, then explicitly requested full completion. P1.1 now uses accepted 204 modes and memory-only cache reads for status, plus bounded coalescing metadata enrichment inside the existing cancellable task. Names arrive independently; no clear/timer-stop, initial-history readiness, tray, process-free detector, polling-pause or CI redesign. DESTINY_API owns exact retry/bounds/association rules; PERFORMANCE owns request/resource effects and live gaps.

Local Node frontend regressions (ten tests) and the Vite production build pass, with the same two profile-widget accessibility warnings. No local Rust installation was added; use the implementation commit's existing Windows workflow for Rust tests and MSI result. Dependency installation is validated by that workflow. Version remains 1.1.4; identify this testbuild by commit/run. Existing timing/history/overlay coverage is retained, with the old predicate-only metadata tests replaced by cache/state tests. No installed-app/Destiny or FPS/frametime benchmark is claimed.

## Knowledge ownership

Follow root [AGENTS.md](../AGENTS.md). Read PROJECT_CONTEXT and ROADMAP at session start, then relevant specialist files; read all on a takeover. Check the actual checkout/PR/build before relying on this dated snapshot. Linked documents require explicit reading; old chats are not a substitute for recorded decisions.

| File | Owns |
| --- | --- |
| [PROJECT_CONTEXT](PROJECT_CONTEXT.md) | User decisions, strict safety/close boundary, Codex workflow, Threepole Status |
| [CUSTOM_FEATURES](CUSTOM_FEATURES.md) | Actual settings/features, icon assets and notification defects |
| [DESTINY_API](DESTINY_API.md) | Endpoints, timing, history, caches, failure behavior and request budget |
| [PERFORMANCE](PERFORMANCE.md) | Retained optimizations, observations, residual risks and full live matrix |
| [ROADMAP](ROADMAP.md) | Prioritized work, acceptance criteria and cross-feature effects |
| [BUILD-GITHUB-DE](../BUILD-GITHUB-DE.md) | Local environment tiers, Windows builds and API-key handling |
| [CUSTOM.md](../CUSTOM.md) | Version history; 1.1.3 behavior is historical |

Use this existing structure. GitHub holds durable history; the checkout is the working copy. Do not create duplicate handoffs or claim inaccessible chats were reviewed.

## Current decisions and architecture

Eager Edge is required for every Threepole Custom icon, including overlay-clear; the calendar SVG remains pending. Older original-icon instructions are superseded. Codex handles natural-language development orders, validation and Windows build delivery.

The permanent goal is measurably faster, more reliable and richer Custom behavior than original Threepole at preferably equal or lower resource use, without additional/riskier game interaction. [Original source](https://github.com/des-sh/threepole) is a permanent technical reference: future optimizations must inspect relevant original/Custom code differences and identify compared revisions. PROJECT_CONTEXT owns the priority order and required prior decision on resource tradeoffs; PERFORMANCE owns the comparison protocol. No measured speed/resource advantage is established yet.

Activity data stays official Bungie API/web/manifest-only; ordinary process-free Windows window queries are permitted for the independent overlay. Package 1 removes foreground process inspection; window-only matching resolves that source-level conflict, with remaining live reliability limits recorded in PERFORMANCE. The latest decision preserves current X/close-and-recreate and simple tray-click behavior, superseding the previous double-click/retained-hidden-window target. Intentional tray operation retains tracking/overlay. Verify full Exit; missing central cleanup coordination alone does not prove faulty exit. ROADMAP owns the narrowed verification scope and evidence-based minimal proposals. No polling increase or background-service dependency is authorized.

Package 1 now uses the observed exact Destiny title/class, visibility and valid physical geometry, without a process/PID query or identity cache. Failed detection/placement hides the overlay; unchanged geometry skips redundant calls at the existing 200 ms cadence. Eleven Rust regressions are added. It is not a window/tray rebuild; full-exit verification and the focused live overlay matrix remain open. Propose only demonstrated further corrections and obtain separate implementation approval. P1.1 addresses manifest/cache blocking; initial-history blocking and missed clear notifications remain separate next priorities. Preserve every existing Custom feature and independent setting, all earlier API findings/optimizations and pending live tests.

Tauri 1/Rust/Tokio owns requests, configuration, tray and windows; Svelte 3/TypeScript/Vite supplies the UI and a separate HTML/TypeScript webview supplies the overlay. IPC/events carry data/preferences. Custom identity/config/pipe separate it from official Threepole; the updater is disabled. One cancellable player-data task joins current/history/optional timing futures **after** initial current and history both succeed. Specialist files document the remaining blocking paths and retained invariants.

## Test/build evidence

The pre-package baseline [Windows build #22](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38475047295) succeeded on `628d2ae3733c63f50137549a558b133ab72ef959`. Package 1 adds eleven Rust cases (48 total with the existing 37); consult the workflow associated with the package commit for its compilation, five frontend tests, Rust tests and MSI result. The local machine has no Rust toolchain. A live window-only metadata sample established title/class and 125% DPI geometry, not installed-MSI behavior or a performance benchmark. PERFORMANCE owns details and remaining live tests.

[Windows build #19, 2026-09-29](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/36518496000) passed on the audited head: **37 Rust tests, five overlay tests and MSI packaging**. [MSI artifact](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/36518496000/artifacts/11011229394) was available during takeover; links can expire. Historical PR #1 text references older build #11. Always tie delivered builds to their commit.

[Windows build #20, 2026-10-12](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38243304601) subsequently succeeded on `bb3f21f1877cb16f703888b9afb45493d28c4b8d`: 37 Rust tests, five overlay tests and `Threepole Custom_1.1.4_x64_en-US.msi` packaging. Its [MSI artifact](https://github.com/AngelsDoCryy/threepole_custom1-/actions/runs/38243304601/artifacts/11744142438) is build evidence, not a Windows/Destiny live test. This documentation update adds no runtime validation; its push can trigger the unchanged workflow, and any resulting run must be checked separately.

Current Rust coverage: 19 timing-state, six parser, ten history/current-helper, two preference and eleven process-free overlay tests (48 total). Frontend: three visibility and two clear/orbit ordering tests. They cover backward/epoch orbit, earlier starts for another activity, optional fallback, orbit/re-entry, stale responses, standalone Transitory parsing and late-clear independence. They do not establish network scheduling, source-cache recovery, lock behavior, Windows lifecycle or live Bungie timing.

CI installs dependencies, runs overlay tests, builds through Tauri/Vite, runs release Rust tests and uploads the MSI. There is no separate TypeScript/Svelte type check; the PR's earlier local type-check claim is historical. Toolchain details belong to BUILD-GITHUB-DE.

Package 1 local validation: five frontend tests and Vite production build passed with bundled Node 24.19.0. The two existing profile-widget accessibility warnings remain. Temporary Yarn 1.22.22 installed declared project dependencies without a global tool installation; frozen-lockfile mode rejected the existing lock mismatch, so installation used pure-lockfile mode to leave tracked dependency files untouched. CI retains its original Node 20 / Yarn install reference path; Rust/MSI validation is performed there.

The takeover's in-memory check of the production notification function reproduced missed same-ID corrections and late older clears; this was not an installed-app test. Historical orbit/restart delays are not proved fixed or present in 1.1.4. PERFORMANCE retains those observations and every pending live check. A documentation-only cleanup adds no runtime validation.
