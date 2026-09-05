/**
 * Platform abstraction — Fase 1 do plano web+daemon
 * (`claude/kura-plano-web-daemon.md` §3, item "Fase 1 — A costura de
 * plataforma no frontend").
 *
 * `desktop/src` fala com o backend hoje só através de três primitivas do
 * Tauri: `invoke`, `emit`/`listen` e `Channel`. Esta interface nomeia essas
 * primitivas (mais os recursos de plataforma que a UI já checa via
 * `isTauri()`/imports diretos de `@tauri-apps/*`) atrás de um contrato único,
 * para que uma segunda implementação (`platform/web`, JSON-RPC sobre
 * WebSocket contra o `kurad`, Fase 3) possa satisfazer o mesmo contrato sem
 * a UI perceber a diferença.
 *
 * Esta é a PRIMEIRA fatia do refactor: só a interface + a implementação
 * `platform/tauri/` (que delega para o que já existe, comportamento
 * idêntico). Nenhum call site foi migrado ainda — os ~108 `invoke` crus, 49
 * `listen` e os usos de `isTauri()` continuam chamando `@tauri-apps/*`
 * diretamente. A migração call-site-por-call-site é o passo 2 do plano,
 * feita em fatias seguintes.
 */

/** Função de cancelamento retornada por `listen`, igual ao `UnlistenFn` do Tauri. */
export type UnlistenFn = () => void;

/**
 * Espelha a forma de `Channel<T>` do Tauri (`onmessage` settável, `id`
 * opaco) — usada hoje só pelo terminal e pelo socket nativo do relay. A
 * implementação web (Fase 3) mapeia isso para um sub-stream binário
 * multiplexado no mesmo WebSocket (ver §2.1 do plano).
 */
export interface PlatformChannel<T> {
  onmessage: ((payload: T) => void) | null;
  readonly id: unknown;
}

/**
 * Substitui os 41 usos de `isTauri()` (15 arquivos) por flags nomeadas por
 * capacidade — o mesmo switch que hoje decide "existe tray?" via
 * `isTauri()` passa a decidir "existe tray?" via `capabilities.tray`, o que
 * também serve de documentação viva do que muda de forma no browser (tabela
 * §1 do plano). No shell Tauri todas são `true`; no `platform/web` (Fase 3)
 * cada uma reflete a tabela de tradeoffs decidida com o Bardi (D1-D12).
 */
export interface PlatformCapabilities {
  readonly isTauri: boolean;
  readonly tray: boolean;
  readonly vibrancy: boolean;
  readonly nativeNotifications: boolean;
  readonly globalShortcuts: boolean;
  readonly multiWindow: boolean;
  readonly nativeFileDialogs: boolean;
}

export interface PlatformClipboard {
  readText(): Promise<string>;
  writeText(text: string): Promise<void>;
}

export interface PlatformWindow {
  /** Abre a janela secundária do huddle (hoje o único caso real de
   * multi-janela: `invoke("open_huddle_window")` cria a `WebviewWindow` do
   * lado Rust). No shell Tauri isso é uma janela nativa; no browser (Fase 3)
   * uma aba, com `BroadcastChannel` sincronizando estado entre as duas. */
  openHuddle(): Promise<void>;
}

export interface NotifyOptions {
  title: string;
  body?: string;
}

export interface Platform {
  readonly capabilities: PlatformCapabilities;

  /** Substitui `invokeTauri`/`invoke` cru. Mesmos nomes de comando, mesmos
   * DTOs — nenhuma mudança de contrato nesta fatia. */
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;

  /** Substitui `listen` de `@tauri-apps/api/event`. */
  listen<T>(
    event: string,
    handler: (event: { payload: T }) => void,
  ): Promise<UnlistenFn>;

  /** Substitui `emit` de `@tauri-apps/api/event`. */
  emit(event: string, payload?: unknown): Promise<void>;

  /** Cria um `PlatformChannel` — substitui `new Channel()`. */
  channel<T>(): PlatformChannel<T>;

  /** Abre uma URL no navegador/app padrão do SO. Substitui `openUrl` de
   * `@tauri-apps/plugin-opener`. */
  openUrl(url: string): Promise<void>;

  /** Notificação simples (equivalente a uma Web Notification). A máquina
   * completa de notificação nativa (badge, ações, ativação em segundo
   * plano — `features/notifications/lib/desktop.ts`) continua intocada
   * nesta fatia; entra numa fatia futura. */
  notify(options: NotifyOptions): Promise<void>;

  clipboard: PlatformClipboard;
  window: PlatformWindow;
}
