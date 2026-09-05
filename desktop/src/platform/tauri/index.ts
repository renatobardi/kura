/**
 * Implementação `Platform` para o shell Tauri — delega para o que já existe
 * (`invokeTauri`, `@tauri-apps/api/event`, `@tauri-apps/plugin-*`). Zero
 * mudança de comportamento: esta fatia só nomeia o que já roda hoje atrás
 * do contrato de `../types`, sem migrar nenhum call site.
 *
 * Um `platform/web/index.ts` futuro (Fase 3) satisfaz o mesmo `Platform`
 * falando JSON-RPC/WS com o `kurad` em vez de `@tauri-apps/*`.
 */
import {
  Channel as TauriChannel,
  isTauri as tauriIsTauri,
} from "@tauri-apps/api/core";
import {
  emit as tauriEmit,
  listen as tauriListen,
} from "@tauri-apps/api/event";
import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";
import { sendNotification } from "@tauri-apps/plugin-notification";

import { invokeTauri } from "@/shared/api/tauri";

import type {
  NotifyOptions,
  Platform,
  PlatformCapabilities,
  PlatformChannel,
  UnlistenFn,
} from "../types";

// `desktop/src` ships as a single bundle used both inside the real Tauri
// shell and standalone (browser dev preview, Desktop Smoke E2E under
// Playwright/Chromium) — `platform/web` (Fase 3) does not exist yet, so
// `platform/index.ts` always resolves to `tauriPlatform`. That means these
// flags cannot be build-time constants: whether native capabilities exist
// depends on the runtime the same bundle happens to be executing in today,
// exactly like the `isTauri()` checks this replaces. Getters re-evaluate
// `tauriIsTauri()` on every access instead of caching a stale answer from
// module init (before `window.__TAURI_INTERNALS__` might be set) or a
// hardcoded `true` that would report a Tauri shell inside the e2e browser.
const capabilities: PlatformCapabilities = {
  get isTauri() {
    return tauriIsTauri();
  },
  get tray() {
    return tauriIsTauri();
  },
  get vibrancy() {
    return tauriIsTauri();
  },
  get nativeNotifications() {
    return tauriIsTauri();
  },
  get globalShortcuts() {
    return tauriIsTauri();
  },
  get multiWindow() {
    return tauriIsTauri();
  },
  get nativeFileDialogs() {
    return tauriIsTauri();
  },
};

async function readClipboardText(): Promise<string> {
  // O app hoje lê a área de transferência via `navigator.clipboard`
  // (Async Clipboard API), disponível também na webview do Tauri — sem
  // plugin dedicado. Mantido idêntico nesta fatia.
  return navigator.clipboard.readText();
}

async function writeClipboardText(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}

async function openHuddleWindow(): Promise<void> {
  await invokeTauri("open_huddle_window");
}

async function notify(options: NotifyOptions): Promise<void> {
  await sendNotification({ title: options.title, body: options.body });
}

export const tauriPlatform: Platform = {
  capabilities,

  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return invokeTauri<T>(command, args);
  },

  listen<T>(
    event: string,
    handler: (event: { payload: T }) => void,
  ): Promise<UnlistenFn> {
    return tauriListen<T>(event, handler);
  },

  emit(event: string, payload?: unknown): Promise<void> {
    return tauriEmit(event, payload);
  },

  channel<T>(): PlatformChannel<T> {
    // `TauriChannel` já tem exatamente essa forma (`onmessage` settável,
    // `id` opaco) — não precisa de adaptador.
    return new TauriChannel<T>() as unknown as PlatformChannel<T>;
  },

  openUrl(url: string): Promise<void> {
    return tauriOpenUrl(url);
  },

  notify,

  clipboard: {
    readText: readClipboardText,
    writeText: writeClipboardText,
  },

  window: {
    openHuddle: openHuddleWindow,
  },
};

/** Reexportado para quem só precisa do booleano cru sem ir atrás de
 * `platform.capabilities.isTauri` (paridade com o `isTauri()` de hoje). */
export const isRunningInTauri = tauriIsTauri;
