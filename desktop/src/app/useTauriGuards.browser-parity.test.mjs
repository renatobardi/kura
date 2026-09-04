// Regression test for the S2 spike finding: useIsFullscreen and
// useWebviewZoomShortcuts called native Tauri window/webview APIs without an
// isTauri() guard, so mounting them outside a Tauri webview (e.g. a plain
// browser build of the console) threw synchronously and took the whole app
// down via the root error boundary.
//
// This suite mounts each hook in jsdom with NO `window.__TAURI_INTERNALS__`
// (the exact condition `isTauri()` from @tauri-apps/api/core checks) and
// asserts the hook does not throw and settles to a sane default. It is
// written to fail against the pre-fix source (verified manually: reverting
// the isTauri() guard in either hook reproduces
// `TypeError: Cannot read properties of undefined (reading 'metadata')`
// from useIsFullscreen, and the analogous crash from getCurrentWebview()
// in useWebviewZoomShortcuts).
import assert from "node:assert/strict";
import { after, before, test } from "node:test";

import { JSDOM } from "jsdom";

const dom = new JSDOM("<!doctype html><html><body></body></html>", {
  url: "http://localhost",
});

before(() => {
  Object.assign(globalThis, {
    window: dom.window,
    document: dom.window.document,
    HTMLElement: dom.window.HTMLElement,
    KeyboardEvent: dom.window.KeyboardEvent,
    StorageEvent: dom.window.StorageEvent,
    localStorage: dom.window.localStorage,
    IS_REACT_ACT_ENVIRONMENT: true,
  });
  // Deliberately absent: dom.window.__TAURI_INTERNALS__. This is what a
  // plain browser tab looks like — isTauri() from @tauri-apps/api/core
  // returns false whenever this is undefined.
  delete dom.window.__TAURI_INTERNALS__;
});

after(() => {
  dom.window.close();
});

test("useIsFullscreen does not throw outside Tauri and reports false", async () => {
  const React = await import("react");
  const { act, render, cleanup } = await import("@testing-library/react");
  const { useIsFullscreen } = await import(
    "./../shared/lib/useIsFullscreen.ts"
  );

  let observed;
  function Probe() {
    observed = useIsFullscreen();
    return null;
  }

  assert.doesNotThrow(() => {
    act(() => {
      render(React.createElement(Probe));
    });
  }, "useIsFullscreen crashed outside Tauri — getCurrentWindow() was called without an isTauri() guard");

  assert.equal(
    observed,
    false,
    "outside Tauri there is no native window, so isFullscreen must default to false",
  );
  cleanup();
});

test("useWebviewZoomShortcuts does not throw outside Tauri and keeps rem-zoom shortcuts working", async () => {
  const React = await import("react");
  const { act, render, cleanup, fireEvent } = await import(
    "@testing-library/react"
  );
  const { useWebviewZoomShortcuts } = await import(
    "./useWebviewZoomShortcuts.ts"
  );

  dom.window.localStorage.removeItem("kura:text-scale");
  dom.window.document.documentElement.style.fontSize = "";

  function Probe() {
    useWebviewZoomShortcuts();
    return null;
  }

  assert.doesNotThrow(() => {
    act(() => {
      render(React.createElement(Probe));
    });
  }, "useWebviewZoomShortcuts crashed outside Tauri — getCurrentWebview() was called without an isTauri() guard");

  // The rem-based zoom shortcut itself is not Tauri-specific and should
  // still work in a browser build once the native-only setZoom() call is
  // properly gated. jsdom's default navigator.platform is not mac-like, so
  // hasPrimaryShortcutModifier() expects the Ctrl modifier here (see
  // shared/lib/platform.ts) rather than Meta/Cmd.
  act(() => {
    fireEvent.keyDown(dom.window, { key: "+", ctrlKey: true });
  });
  assert.equal(
    dom.window.document.documentElement.style.fontSize,
    "17.6px",
    "Ctrl/Cmd+= should still bump the rem root font-size outside Tauri",
  );

  cleanup();
  dom.window.document.documentElement.style.fontSize = "";
  dom.window.localStorage.removeItem("kura:text-scale");
});
