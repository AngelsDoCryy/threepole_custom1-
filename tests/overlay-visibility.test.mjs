import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(new URL("../src/overlay/overlay.ts", import.meta.url), "utf8");
const start = source.indexOf("async function updateVisibility()");
const end = source.indexOf("\nfunction createPopup(", start);
assert.ok(start >= 0 && end > start, "Use the production visibility handler");

function harness(appWindow) {
    return vm.runInNewContext(`
        let shown = false;
        let desiredShown = false;
        let updatingVisibility = false;
        let timerChecks = 0;
        function checkTimerInterval() { timerChecks++; }
        ${source.slice(start, end)}
        ({
            signal(value) { desiredShown = value; return updateVisibility(); },
            state() { return { shown, desiredShown, updatingVisibility, timerChecks }; }
        })
    `, { appWindow, console: { warn() {} } });
}

test("failed show remains retryable on the next poll", async () => {
    let attempts = 0;
    const h = harness({
        async show() { if (++attempts === 1) throw new Error("temporary window error"); },
        async hide() {}
    });
    await h.signal(true);
    assert.equal(h.state().shown, false);
    assert.equal(h.state().updatingVisibility, false);
    await h.signal(true);
    assert.equal(h.state().shown, true);
    assert.equal(attempts, 2);
});

test("focus change during pending show finishes hidden", async () => {
    let completeShow;
    let hides = 0;
    const h = harness({
        show() { return new Promise(resolve => { completeShow = resolve; }); },
        async hide() { hides++; }
    });
    const pending = h.signal(true);
    await h.signal(false);
    completeShow();
    await pending;
    assert.equal(h.state().shown, false);
    assert.equal(hides, 1);
    assert.equal(h.state().updatingVisibility, false);
});

test("failed hide retries and repeated visibility events do no extra work", async () => {
    let shows = 0;
    let hides = 0;
    const h = harness({
        async show() { shows++; },
        async hide() { if (++hides === 1) throw new Error("temporary window error"); }
    });
    await h.signal(true);
    await h.signal(true);
    assert.equal(shows, 1);
    await h.signal(false);
    assert.equal(h.state().shown, true);
    await h.signal(false);
    assert.equal(h.state().shown, false);
    assert.equal(hides, 2);
});
