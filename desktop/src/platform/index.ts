/**
 * Ponto único de acesso à plataforma. Hoje sempre resolve para
 * `tauriPlatform` — a troca por `platform/web` (Fase 3) é decidida aqui,
 * quando essa implementação existir (provavelmente por uma env/flag de
 * build, já que o bundle web não inclui `@tauri-apps/*` de jeito nenhum).
 *
 * Nenhum call site importa daqui ainda nesta fatia — ver o comentário de
 * topo de `./types`.
 */
import { tauriPlatform } from "./tauri";
import type { Platform } from "./types";

export const platform: Platform = tauriPlatform;

export type {
  NotifyOptions,
  Platform,
  PlatformCapabilities,
  PlatformChannel,
  PlatformClipboard,
  PlatformWindow,
  UnlistenFn,
} from "./types";
