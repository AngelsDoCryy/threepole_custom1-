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
    const setTimeout = () => ++id;
    const appWindow = { async show() {}, async hide() {} };
    const h = vm.runInNewContext(source + `
        ;({ refresh, applyPreferences, async show() { desiredShown = true; await updateVisibility(); } })
    `, {
        exports: {}, console, document: { querySelector: element },
        window: { setTimeout }, setTimeout,
        setInterval(fn) { const next = ++id; intervals.set(next, fn); return next; },
        clearTimeout(key) { intervals.delete(key); },
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
    return { ...h, element, intervals, popups };
}

function status(hash = 123, start = "2026-09-18T18:00:00Z", history = []) {
    return { error: null, lastUpdate: {
        currentActivity: { activityHash: hash, startDate: start,
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
