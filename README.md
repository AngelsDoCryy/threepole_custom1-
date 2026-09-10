# Threepole Custom

Custom build of Threepole for Destiny 2.

This project is based on the original Threepole application by des-sh and adds several custom overlay and tracking features.

---

## Original Threepole features

The following functionality comes from the original Threepole project:

- Destiny 2 activity timer
- Overlay mode
- Daily activity clears
- Activity clear notifications
- Timer milliseconds
- Bungie.net profile integration
- Multiple profile/account support
- Background tray operation
- Current activity detection
- Activity history display

Original project:

https://github.com/des-sh/threepole

---

## Custom features

The following features were added in this custom build:

### Activity name display

- Displays the current activity name above the timer
- Automatically updates when the activity changes
- Can be enabled or disabled independently

### Configurable activity name auto-hide

- Activity name can automatically disappear after a configurable amount of time
- Auto-hide can be enabled or disabled independently
- Hide delay can be changed at any time

### Per-activity daily clear counter

Instead of combining all activity clears into one counter, clears are tracked separately for each activity.

Example:

- Vault of Glass: 3 clears
- Spire of the Watcher: 2 clears
- Sundered Doctrine: 5 clears

When switching activities, the displayed clear count automatically changes to the count for that activity.

Daily reset behavior is preserved. The original Threepole calendar icon is used next to the clear counter.

### Additional Raid / Dungeon detection

Improved activity detection for Raid and Dungeon activities that may not expose the normal activity mode information.

A small local activity-mode cache is used so history processing does not need extra manifest requests. New Raid / Dungeon activities are still classified dynamically from Bungie manifest activity data when they are encountered.

### Bungie API affinity handling

- Reuses one HTTP client for the full app session
- Preserves Bungie affinity cookies between API requests
- Keeps the existing Current Activity polling interval unchanged
- Does not add extra Bungie API polling

This is intended to reduce cases where different Bungie backend servers return temporarily inconsistent current-activity state.

### Custom application icon

The application uses a custom Eager Edge icon for:

- application icon
- taskbar icon
- tray icon
- Windows installer icon

### Custom updater behavior

The original Threepole automatic updater is disabled in this build to prevent official releases from overwriting the custom version.

---

## Safety / Destiny interaction

This custom version does not add invasive Destiny 2 interaction.

The custom functionality does not use:

- Destiny 2 memory reading
- Destiny 2 memory writing
- DLL injection
- code injection
- gameplay automation
- input automation
- modification of Destiny 2 game files

Activity information is obtained through the existing Threepole logic and Bungie API / manifest data.

---

## Original project

Threepole was originally created by des-sh.

https://github.com/des-sh/threepole

This custom version is based on that project and retains the original project license.

---

## License

GPL-3.0
