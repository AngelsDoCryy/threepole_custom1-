# Threepole Custom — Features

This file tracks custom behavior separately from original Threepole functionality.

## Activity name display

Target behavior:

- Show the current activity name above the timer.
- Update automatically when the activity changes.
- Allow the feature to be enabled or disabled independently.
- Support optional auto-hide after a configurable delay.
- Auto-hide itself must be independently controllable.

The initial preferred hide delay was 10 seconds, but the delay must remain configurable.

## Per-activity daily clear counter

Target behavior:

- Track daily clears separately for each activity.
- Switching activities changes the displayed counter to the selected activity.
- Returning to an activity restores that activity's current daily count.
- Preserve normal daily reset behavior.
- Use the original Threepole calendar/clear icon beside the counter.

Example behavior:

- Vault of Glass: 3
- Spire of the Watcher: 2
- Sundered Doctrine: 5

## Raid / Dungeon classification

Target behavior:

- Support activities that do not expose normal activity-mode data cleanly.
- Use Bungie manifest data when needed.
- Cache activity-hash -> classification data locally.
- Do not perform repeated manifest work for every history entry.
- Unknown or future activities should degrade gracefully.

## Independent settings

Custom features should not unnecessarily depend on each other.

Keep separate controls for:

- activity-name visibility
- activity-name auto-hide
- clear counter
- clear notifications
- timer visibility
- timer milliseconds
- activity icon where applicable

## Activity / clear icon behavior

The clear counter should retain the original Threepole calendar/clear icon.

A Raid/Dungeon activity icon may be shown as an optional UI feature if classification data is available. Failure to resolve an activity icon must not break activity tracking or clear counting.

## Application icon

Current 1.1.4-era code/documentation still includes the custom Eager Edge application/taskbar/tray/installer icon.

Latest requested direction:

- restore the standard Threepole application/taskbar/tray/installer icon
- keep the original Threepole calendar/clear icon for clears

Treat the application-icon revert as open until code/build verification confirms it.

## Notifications

Clear notifications remain optional and should stay logically separate from live activity tracking.

A notification arriving before the live overlay updates is not by itself a bug. History/notification data must never force orbit or stop a later active run.

## Timer milliseconds

Timer milliseconds are an optional presentation feature.

Requirement:

- enabling milliseconds should not introduce material performance cost or change activity/timer state logic

## Updater

The original automatic updater is disabled in the custom build so an upstream Threepole release cannot overwrite the custom installation.

## Deferred / future ideas

Possible future work should remain separate from core stability work. Examples previously discussed include additional UI presentation such as class icons.

Do not prioritize cosmetic additions above:

1. orbit/activity responsiveness
2. repeated-run reliability
3. API-only safety
4. low overhead
5. stable clear/timer state