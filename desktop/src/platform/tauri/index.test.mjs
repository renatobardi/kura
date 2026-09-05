import assert from "node:assert/strict";
import { after, test } from "node:test";

import { isRunningInTauri, tauriPlatform } from "./index.ts";

// Guards the wiring, not the underlying Tauri APIs (those are covered by
// existing tests on `invokeTauri`/etc. — this module is a thin delegation
// layer, see ../types.ts for why).

// `desktop/src` ships as one bundle used both inside the real Tauri shell
// and standalone (browser dev preview, Desktop Smoke E2E under
// Playwright/Chromium — `platform/web` does not exist yet, see the comment
// on `capabilities` in ./index.ts), so these flags must track the runtime
// `isTauri()` reports, not a build-time constant. `@tauri-apps/api/core`'s
// `isTauri()` reads `globalThis.isTauri`; toggling it here reproduces both
// runtimes without mocking `window.__TAURI_INTERNALS__`.
after(() => {
  delete globalThis.isTauri;
});

test("tauriPlatform.capabilities reports a Tauri shell when isTauri() is true", () => {
  globalThis.isTauri = true;
  assert.equal(tauriPlatform.capabilities.isTauri, true);
  assert.equal(tauriPlatform.capabilities.tray, true);
  assert.equal(tauriPlatform.capabilities.multiWindow, true);
});

test("tauriPlatform.capabilities reports no native shell when isTauri() is false (e2e/browser)", () => {
  delete globalThis.isTauri;
  assert.equal(tauriPlatform.capabilities.isTauri, false);
  assert.equal(tauriPlatform.capabilities.tray, false);
  assert.equal(tauriPlatform.capabilities.vibrancy, false);
  assert.equal(tauriPlatform.capabilities.nativeNotifications, false);
  assert.equal(tauriPlatform.capabilities.globalShortcuts, false);
  assert.equal(tauriPlatform.capabilities.multiWindow, false);
  assert.equal(tauriPlatform.capabilities.nativeFileDialogs, false);
});

test("isRunningInTauri is re-exported from @tauri-apps/api/core", () => {
  assert.equal(typeof isRunningInTauri, "function");
});

test("tauriPlatform exposes every Platform member", () => {
  assert.equal(typeof tauriPlatform.invoke, "function");
  assert.equal(typeof tauriPlatform.listen, "function");
  assert.equal(typeof tauriPlatform.emit, "function");
  assert.equal(typeof tauriPlatform.channel, "function");
  assert.equal(typeof tauriPlatform.openUrl, "function");
  assert.equal(typeof tauriPlatform.notify, "function");
  assert.equal(typeof tauriPlatform.clipboard.readText, "function");
  assert.equal(typeof tauriPlatform.clipboard.writeText, "function");
  assert.equal(typeof tauriPlatform.window.openHuddle, "function");
});
