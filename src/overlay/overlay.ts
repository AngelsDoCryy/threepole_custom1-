import "../core/global.css"
import "./overlay.css"
import { appWindow } from "@tauri-apps/api/window";
import { createPopup as _createPopup, type Popup } from "./popups";
import type { TauriEvent, Preferences, CurrentActivity, PlayerDataStatus } from "../core/types";
import { countActivityClears, determineActivityType, formatMillis, formatTime } from "../core/util";
import { getPlayerdata, getPreferences } from "../core/ipc";

const loaderElem = document.querySelector<HTMLElement>("#widget-loader")!;
const errorElem = document.querySelector<HTMLElement>("#widget-error")!;
const widgetContentElem = document.querySelector<HTMLElement>("#widget-content")!;
const activityNameElem = document.querySelector<HTMLElement>("#activity-name")!;
const timerElem = document.querySelector<HTMLElement>("#timer")!;
const timeElem = document.querySelector<HTMLElement>("#time")!;
const msElem = document.querySelector<HTMLElement>("#ms")!;
const counterElem = document.querySelector<HTMLElement>("#counter")!;
const dailyElem = document.querySelector<HTMLElement>("#daily")!;

let currentActivity: CurrentActivity;
let currentActivityHistory: PlayerDataStatus["lastUpdate"]["activityHistory"] = [];
let lastRaidId;
let lastActivityInstanceKey: string | null = null;
let activityNameHideTimer: number | null = null;
let doneInitialRefresh = false;

let shown = false;
let desiredShown = false;
let updatingVisibility = false;
let prefs: Preferences;
let timerInterval;

async function init() {
    await appWindow.listen("show", () => {
        desiredShown = true;
        void updateVisibility();
    });

    await appWindow.listen("hide", () => {
        desiredShown = false;
        void updateVisibility();
    });

    let preferencesUpdated = false;
    let playerdataUpdated = false;
    let latestPlayerdata: PlayerDataStatus;
    await appWindow.listen("preferences_update", (p: TauriEvent<Preferences>) => {
        preferencesUpdated = true;
        applyPreferences(p.payload);
    });
    await appWindow.listen("playerdata_update", (e: TauriEvent<PlayerDataStatus>) => {
        playerdataUpdated = true;
        latestPlayerdata = e.payload;
        if (prefs) refresh(e.payload);
    });
    const initialPreferences = await getPreferences();
    if (!preferencesUpdated) applyPreferences(initialPreferences);
    if (playerdataUpdated) refresh(latestPlayerdata);
    const initialPlayerdata = await getPlayerdata();
    if (!playerdataUpdated) refresh(initialPlayerdata);
}

async function updateVisibility() {
    if (updatingVisibility) return;
    updatingVisibility = true;
    try {
        while (shown !== desiredShown) {
            const target = desiredShown;
            if (target) await appWindow.show();
            else await appWindow.hide();
            shown = target;
            checkTimerInterval();
        }
    } catch (error) {
        // Do not mark a failed show/hide as successful; the next event retries.
        console.warn("Overlay visibility update failed", error);
    } finally {
        updatingVisibility = false;
    }
}

function createPopup(popup: Popup) {
    _createPopup(popup, shown);
}

function checkTimerInterval() {
    if (!prefs || !prefs.displayTimer || !shown || !determineActivityType(currentActivity?.activityInfo?.activityModes)) {
        clearTimeout(timerInterval);
        timerInterval = null;
        timerElem.classList.add("hidden");
        return;
    }

    timerElem.classList.remove("hidden");

    if (!timerInterval) {
        timerInterval = setInterval(() => requestAnimationFrame(timerTick), 1000 / (prefs.displayMilliseconds ? 30 : 2));
    }
}

function refresh(playerDataStatus: PlayerDataStatus) {
    let playerData = playerDataStatus?.lastUpdate;

    if (!playerData) {
        widgetContentElem.classList.add("hidden");

        currentActivity = null;
        currentActivityHistory = [];
        checkTimerInterval();
        doneInitialRefresh = false;

        if (playerDataStatus?.error) {
            loaderElem.classList.add("hidden");
            errorElem.classList.remove("hidden");
            createPopup({ title: "Failed to fetch initial stats", subtext: playerDataStatus.error });
        } else {
            errorElem.classList.add("hidden");
            loaderElem.classList.remove("hidden");
        }

        return;
    }

    loaderElem.classList.add("hidden");
    errorElem.classList.add("hidden");
    widgetContentElem.classList.remove("hidden");

    currentActivity = playerData.currentActivity;
    currentActivityHistory = playerData.activityHistory;

    checkTimerInterval();
    updateActivityDisplay(playerData.activityHistory);

    let latestRaid = playerData.activityHistory[0];

    if (doneInitialRefresh && latestRaid?.completed && lastRaidId != latestRaid.instanceId && prefs.displayClearNotifications) {
        const type = determineActivityType(latestRaid.modes);

        if (type) {
            const typeFormatted = type.charAt(0).toUpperCase() + type.slice(1);
            createPopup({ title: `${typeFormatted} clear result`, subtext: `API Time: <strong>${latestRaid.activityDuration}</strong>` });
        }
    }

    lastRaidId = latestRaid?.instanceId;

    if (!doneInitialRefresh) {
        createPopup({ title: `${playerData.profileInfo.displayName}#${playerData.profileInfo.displayTag}`, subtext: "Threepole is active." });
    }

    doneInitialRefresh = true;
}

function updateActivityDisplay(activityHistory: PlayerDataStatus["lastUpdate"]["activityHistory"]) {
    const activity = currentActivity;
    const activityInfo = activity?.activityInfo;
    const type = determineActivityType(activityInfo?.activityModes);

    if (!activity || !activityInfo || !type) {
        clearTimeout(activityNameHideTimer ?? undefined);
        activityNameHideTimer = null;
        lastActivityInstanceKey = null;
        activityNameElem.classList.add("hidden");
        counterElem.classList.add("hidden");
        return;
    }

    const activityInstanceKey = `${activity.activityHash}:${activity.startDate}`;
    if (activityInstanceKey !== lastActivityInstanceKey) {
        lastActivityInstanceKey = activityInstanceKey;
        showActivityName();
    }

    if (prefs.displayDailyClears) {
        dailyElem.innerText = String(countActivityClears(activityHistory, activity.activityHash));
        counterElem.classList.remove("hidden");
    } else {
        counterElem.classList.add("hidden");
    }
}

function showActivityName() {
    clearTimeout(activityNameHideTimer ?? undefined);

    if (!prefs.displayActivityName || !currentActivity?.activityInfo?.name) {
        activityNameElem.classList.add("hidden");
        return;
    }

    activityNameElem.innerText = currentActivity.activityInfo.name;
    activityNameElem.classList.remove("hidden");

    if (prefs.autoHideActivityName) {
        const delay = Math.max(0, prefs.activityNameHideDelaySeconds) * 1000;
        activityNameHideTimer = window.setTimeout(() => {
            activityNameElem.classList.add("hidden");
            activityNameHideTimer = null;
        }, delay);
    }
}

function applyPreferences(p: Preferences) {
    prefs = p;

    if (!p.displayActivityName) {
        clearTimeout(activityNameHideTimer ?? undefined);
        activityNameElem.classList.add("hidden");
    } else if (currentActivity?.activityInfo) {
        showActivityName();
    }

    if (p.displayMilliseconds) {
        msElem.classList.remove("hidden");
    } else {
        msElem.classList.add("hidden");
    }

    clearTimeout(timerInterval);
    timerInterval = null;

    checkTimerInterval();
    updateActivityDisplay(currentActivityHistory);
}

function timerTick() {
    if (!currentActivity || !prefs?.displayTimer || !shown) return;
    let millis = Number(new Date()) - Number(new Date(currentActivity.startDate));
    timeElem.innerHTML = formatTime(millis);
    msElem.innerHTML = formatMillis(millis);
}

init();
