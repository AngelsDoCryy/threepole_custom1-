import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";

function compile(path) {
    const source = readFileSync(new URL(path, import.meta.url), "utf8");
    return ts.transpileModule(source, {
        compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 }
    }).outputText;
}

function loadModule(path, require) {
    const exports = {};
    vm.runInNewContext(compile(path), { exports, require });
    return exports;
}

const constants = loadModule("../src/core/consts.ts");
const util = loadModule("../src/core/util.ts", () => constants);
const source = compile("../src/overlay/overlay.ts").replace(/\ninit\(\);\s*$/, "");

async function harness(initial) {
    const elements = new Map();
    const intervals = new Map();
    const timeouts = new Map();
    const popups = [];
    let id = 0;
    const element = selector => {
        if (!elements.has(selector)) {
            const classes = new Set();
            elements.set(selector, {
                innerText: "", innerHTML: "",
                classList: { add: c => classes.add(c), remove: c => classes.delete(c), contains: c => classes.has(c) }
            });
        }
        return elements.get(selector);
    };
    const setTimeout = fn => { const next = ++id; timeouts.set(next, fn); return next; };
    const appWindow = { async show() {}, async hide() {} };
    const h = vm.runInNewContext(source + `
        ;({ refresh, applyPreferences, async show() { desiredShown = true; await updateVisibility(); } })
    `, {
        exports: {}, console, document: { querySelector: element },
        window: { setTimeout }, setTimeout,
        setInterval(fn) { const next = ++id; intervals.set(next, fn); return next; },
        clearTimeout(key) { intervals.delete(key); timeouts.delete(key); },
        requestAnimationFrame(fn) { fn(); },
        require(name) {
            if (name.endsWith(".css") || name.endsWith("/ipc")) return {};
            if (name === "@tauri-apps/api/window") return { appWindow };
            if (name === "./popups") return { createPopup: popup => popups.push(popup) };
            if (name === "../core/util") return util;
            throw new Error(`Unexpected import ${name}`);
        }
    });
    h.applyPreferences({ displayTimer: true, displayMilliseconds: true,
        displayActivityName: true, autoHideActivityName: false,
        displayDailyClears: true, displayClearNotifications: true });
    h.refresh(initial);
    await h.show();
    return { ...h, element, intervals, popups, timeouts };
}

function status(hash = 123, start = "2026-09-18T18:00:00Z", history = []) {
    return { error: null, lastUpdate: {
        currentActivity: { activityHash: hash, startDate: start,
            activityModes: hash ? [82] : [],
            activityInfo: hash ? { name: `Dungeon ${hash}`, activityModes: [82] } : null },
        activityHistory: history,
        profileInfo: { displayName: "Test", displayTag: "0001" }
    } };
}

const clear = { instanceId: "previous-run", activityHash: 123, completed: true,
    activityDuration: "1:00", activityDurationSeconds: 60, modes: [82],
    period: "2026-09-18T18:00:00Z" };

test("clear can precede orbit; the orbit event hides timer, name and clears immediately", async () => {
    const h = await harness(status());
    h.refresh(status(123, clear.period, [clear]));
    assert.equal(h.popups.length, 2, "startup plus clear notification");
    assert.equal(h.element("#timer").classList.contains("hidden"), false);
    h.refresh(status(0, clear.period, [clear]));
    for (const selector of ["#timer", "#activity-name", "#counter"]) {
        assert.equal(h.element(selector).classList.contains("hidden"), true, selector);
    }
    assert.equal(h.intervals.size, 0, "no timer keeps ticking in orbit");
    assert.equal(h.popups.length, 2, "orbit does not repeat the clear");
});

test("a late clear for the previous run does not stop a new run, even in the same dungeon", async () => {
    for (const hash of [123, 456]) {
        const h = await harness(status());
        h.refresh(status(hash, "2026-09-18T18:02:00Z"));
        h.refresh(status(hash, "2026-09-18T18:02:00Z", [clear]));
        assert.equal(h.element("#timer").classList.contains("hidden"), false);
        assert.equal(h.intervals.size, 1);
        assert.equal(h.element("#activity-name").innerText, `Dungeon ${hash}`);
        assert.equal(h.popups.length, 2);
    }
});

test("known 204 modes show timer and per-hash clears before the name arrives", async () => {
    const pending = status(123, clear.period, [clear]);
    pending.lastUpdate.currentActivity.activityInfo = null;
    const h = await harness(pending);
    assert.equal(h.element("#timer").classList.contains("hidden"), false);
    assert.equal(h.element("#counter").classList.contains("hidden"), false);
    assert.equal(h.element("#daily").innerText, "1");
    assert.equal(h.element("#activity-name").classList.contains("hidden"), true);
    h.refresh(status(123, clear.period, [clear]));
    assert.equal(h.element("#activity-name").innerText, "Dungeon 123");
    assert.equal(h.element("#activity-name").classList.contains("hidden"), false);
    assert.equal(h.intervals.size, 1);
    assert.equal(h.popups.length, 1, "metadata arrival does not repeat clears");
});

test("unknown classification hides previous activity until reliable modes arrive", async () => {
    const h = await harness(status());
    const pending = status(456);
    pending.lastUpdate.currentActivity.activityInfo = null;
    pending.lastUpdate.currentActivity.activityModes = [7];
    h.refresh(pending);
    for (const selector of ["#timer", "#counter", "#activity-name"]) {
        assert.equal(h.element(selector).classList.contains("hidden"), true);
    }
    assert.equal(h.intervals.size, 0);
    h.refresh(status(456));
    assert.equal(h.element("#activity-name").innerText, "Dungeon 456");
    assert.equal(h.intervals.size, 1);
});

test("name-less switches hide the old name and same-hash restarts retain timer", async () => {
    const h = await harness(status());
    for (const start of ["18:02:00", "18:04:00", "18:06:00"]) {
        const pending = status(456, `2026-09-18T${start}Z`);
        pending.lastUpdate.currentActivity.activityInfo = null;
        h.refresh(pending);
        assert.equal(h.element("#activity-name").classList.contains("hidden"), true);
        assert.equal(h.element("#timer").classList.contains("hidden"), false);
        h.refresh(status(456, pending.lastUpdate.currentActivity.startDate));
        assert.equal(h.element("#activity-name").innerText, "Dungeon 456");
        assert.equal(h.intervals.size, 1);
    }
    h.refresh(status(0));
    assert.equal(h.intervals.size, 0);
});

test("late metadata respects independently disabled display settings", async () => {
    const h = await harness(status());
    h.applyPreferences({ displayTimer: false, displayActivityName: false,
        displayDailyClears: false, displayClearNotifications: false });
    const pending = status(456);
    pending.lastUpdate.currentActivity.activityInfo = null;
    h.refresh(pending);
    h.refresh(status(456));
    for (const selector of ["#timer", "#counter", "#activity-name"]) {
        assert.equal(h.element(selector).classList.contains("hidden"), true);
    }
    assert.equal(h.intervals.size, 0);
});

test("late activity name starts auto-hide once and unchanged metadata does not reshow it", async () => {
    const pending = status();
    pending.lastUpdate.currentActivity.activityInfo = null;
    const h = await harness(pending);
    h.applyPreferences({ displayTimer: true, displayActivityName: true, autoHideActivityName: true,
        activityNameHideDelaySeconds: 10, displayDailyClears: true, displayClearNotifications: true });
    assert.equal(h.timeouts.size, 0);
    h.refresh(status());
    assert.equal(h.timeouts.size, 1);
    const hide = [...h.timeouts.values()][0];
    hide();
    h.refresh(status());
    assert.equal(h.element("#activity-name").classList.contains("hidden"), true);
    assert.equal(h.timeouts.size, 1, "same metadata must not restart auto-hide");
    h.refresh(status(123, "2026-09-18T19:00:00Z"));
    assert.equal(h.element("#activity-name").classList.contains("hidden"), false);
});
