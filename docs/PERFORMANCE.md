# Threepole Custom — Performance

This file tracks performance goals, known regressions, implemented mitigations and live-validation needs.

## Primary performance target

The custom build should preserve the simplicity, startup speed and responsiveness of original Threepole while adding optional features.

Critical user-visible paths:

- app startup
- first activity recognition
- switching activities
- returning to orbit
- stopping/updating the timer
- restarting the same activity shortly after completion
- overlay recovery/visibility

## Known user observations

### Orbit responsiveness

On an earlier custom build, original Threepole appeared to detect return to orbit sooner.

Observed symptoms:

- custom Threepole kept the previous activity/timer visible longer
- clear notification could appear before the overlay timer/state visibly updated

This motivated the 1.1.4 separation of live activity identity/status from optional timing data.

### Same-Dungeon restart

On an older build, the user observed approximately:

- first Dungeon recognition: ~30 seconds
- immediate restart of the same Dungeon: ~2–3 minutes before recognition

This has not yet been proven present or fixed on 1.1.4. Treat it as a live-validation gap, not as a confirmed current regression.

## Performance architecture in 1.1.4

Current design goals:

- CharacterActivities/component 204 remains on the current-activity path with the existing two-second pause
- Transitory/component 1000 is independent and uses a ten-second pause
- optional timing must not block activity identity/orbit state
- local activity classification cache avoids repeated manifest work
- shared HTTP client avoids needless session setup and preserves Bungie affinity/cookies
- history/clear notifications remain independent from current-activity state

## Request-volume guardrail

The Transitory path adds at most six requests per minute in steady state at the current ten-second pause.

Do not raise polling rates without first proving the delayed path and evaluating API impact.

## Low-overhead requirements

Custom UI features should be cheap presentation/state work.

In particular:

- timer milliseconds should not materially change CPU/network behavior
- activity-name rendering should not trigger additional API polling
- clear-counter display should reuse existing tracked state
- icons should use cached/local metadata where possible
- background work should stop with the app
- no new service or persistent helper process should be introduced

## Existing mitigation themes

Work already implemented or considered includes:

- local activity-mode cache
- independent live status and optional timing requests
- response-freshness handling that does not misuse `dateActivityStarted`
- clearing stale optional timing after confirmed orbit
- shared HTTP client
- Bungie affinity/cookie preservation
- overlay visibility/recovery regression fixes
- regression tests around orbit/re-entry and timing data

## Validation checklist

When comparing custom Threepole with original Threepole, record separately:

1. app launch to usable UI
2. launch to authenticated/loaded state where relevant
3. activity identity detection latency
4. activity start/timer update latency
5. orbit detection latency
6. timer stop/clear latency
7. same-activity restart latency
8. different-activity switch latency
9. notification timing relative to live overlay state
10. any visible UI freeze or overlay recovery issue

Run repeated trials rather than relying on one run, because Bungie API freshness can vary independently of local code.

## Diagnostic principle

Before changing code, determine where the delay occurs:

- Bungie server/API freshness
- CharacterActivities request/acceptance
- optional Transitory timing
- history/clear processing
- manifest lookup/classification
- local state propagation
- frontend rendering

A performance fix should target the identified layer and include a regression test when practical.

## Success criteria

A performance change is considered complete only when:

- relevant automated tests pass
- build succeeds where required
- no safety boundary is weakened
- request volume is understood
- live behavior is tested when the issue depends on Bungie/Destiny timing
- documentation states what is proven versus what remains uncertain