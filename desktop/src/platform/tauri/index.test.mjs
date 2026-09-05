import assert from "node:assert/strict";
import { test } from "node:test";

import { isRunningInTauri, tauriPlatform } from "./index.ts";

// Guards the wiring, not the underlying Tauri APIs (those are covered by
// existing tests on `invokeTauri`/etc. — this module is a thin delegation
// layer, see ../types.ts for why).

test("tauriPlatform.capabilities reports a Tauri shell", () => {
  assert.equal(tauriPlatform.capabilities.isTauri, true);
  assert.equal(tauriPlatform.capabilities.tray, true);
  assert.equal(tauriPlatform.capabilities.multiWindow, true);
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
