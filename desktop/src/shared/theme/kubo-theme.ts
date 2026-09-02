/**
 * Kubo tokens for the first-party Kura themes.
 *
 * Every other entry in the theme picker is a Shiki syntax theme whose chrome
 * is *derived* from its editor colors by {@link createThemeVars}. `kura` and
 * `kura-dark` are not: they are the app's own identity, so they emit the exact
 * Kubo tokens that `shared/styles/globals/theme.css` declares for `:root` and
 * `.dark` instead of borrowing GitHub Light / GitHub Dark.
 *
 * The values below MIRROR that stylesheet on purpose — the picker preview
 * paints its tiles from this map without a document to inherit from, so the
 * tokens have to exist in JS too. `kubo-theme.test.mjs` parses theme.css and
 * fails if the two ever drift.
 *
 * Shiki still supplies the syntax highlighting and the terminal ANSI ramp for
 * these themes (see `resolveShikiThemeName`); only the app chrome is Kubo.
 */

/** Kubo light — the `:root` block of theme.css. */
export const KUBO_LIGHT_VARS: Readonly<Record<string, string>> = {
  "--background": "0 0% 100%",
  "--card": "0 0% 100%",
  "--popover": "0 0% 100%",
  "--muted": "60 4.8% 95.9%",
  "--accent": "60 4.8% 95.9%",
  "--secondary": "240 4.8% 95.9%",
  "--huddle-drawer-surface": "24 9.8% 10%",
  "--huddle-control-surface": "12 6.5% 15.1%",
  "--huddle-control-hover-surface": "24 6% 19%",
  "--huddle-control-chevron-surface": "24 9.8% 10%",
  "--huddle-control-chevron-hover-surface": "12 6.5% 15.1%",
  "--huddle-control-foreground": "60 9.1% 97.8%",
  "--huddle-popover-surface": "24 9.8% 10%",
  "--huddle-popover-border": "24 6% 19%",
  "--huddle-tooltip-surface": "12 6.5% 15.1%",
  "--huddle-tooltip-foreground": "60 9.1% 97.8%",
  "--foreground": "20 14.3% 4.1%",
  "--card-foreground": "20 14.3% 4.1%",
  "--popover-foreground": "20 14.3% 4.1%",
  "--muted-foreground": "25 5.3% 44.7%",
  "--accent-foreground": "24 9.8% 10%",
  "--secondary-foreground": "240 5.9% 10%",
  "--destructive": "0 72.2% 50.6%",
  "--destructive-foreground": "60 9.1% 97.8%",
  "--border": "20 5.9% 90%",
  "--input": "20 5.9% 90%",
  "--ring": "24 5.7% 63.1%",
  "--sidebar-background": "60 9.1% 97.8%",
  "--sidebar-foreground": "20 14.3% 4.1%",
  "--sidebar-accent": "60 4.8% 95.9%",
  "--sidebar-accent-foreground": "24 9.8% 10%",
  "--sidebar-border": "20 5.9% 90%",
  "--sidebar-ring": "24 5.7% 63.1%",
  // Git status + warning colors are NOT part of theme.css: they are only ever
  // set from here, so they must be emitted even though every other Kubo token
  // mirrors the stylesheet. Keep them semantic — Kubo reserves color for
  // meaning, and diff decoration is meaning.
  "--status-added": "#1a7f37",
  "--status-deleted": "#cf222e",
  "--status-modified": "#9a6700",
  "--ui-warning": "#9a6700",
  "--ui-warning-bg": "rgba(154, 103, 0, 0.08)",
};

/** Kubo dark — the `.dark` block of theme.css. */
export const KUBO_DARK_VARS: Readonly<Record<string, string>> = {
  "--background": "20 14.3% 4.1%",
  "--card": "24 9.8% 10%",
  "--popover": "24 9.8% 10%",
  "--muted": "12 6.5% 15.1%",
  "--accent": "12 6.5% 15.1%",
  "--secondary": "240 3.7% 15.9%",
  "--huddle-drawer-surface": "24 9.8% 10%",
  "--huddle-control-surface": "12 6.5% 15.1%",
  "--huddle-control-hover-surface": "24 6% 19%",
  "--huddle-control-chevron-surface": "24 9.8% 10%",
  "--huddle-control-chevron-hover-surface": "12 6.5% 15.1%",
  "--huddle-control-foreground": "60 9.1% 97.8%",
  "--huddle-popover-surface": "24 9.8% 10%",
  "--huddle-popover-border": "24 6% 19%",
  "--huddle-tooltip-surface": "12 6.5% 15.1%",
  "--huddle-tooltip-foreground": "60 9.1% 97.8%",
  "--foreground": "60 9.1% 97.8%",
  "--card-foreground": "60 9.1% 97.8%",
  "--popover-foreground": "60 9.1% 97.8%",
  "--muted-foreground": "24 5.7% 63.1%",
  "--accent-foreground": "60 9.1% 97.8%",
  "--secondary-foreground": "60 9.1% 97.8%",
  "--destructive": "0 90.6% 70.8%",
  "--destructive-foreground": "20 14.3% 4.1%",
  "--border": "0 0% 100% / 10%",
  "--input": "0 0% 100% / 15%",
  "--ring": "25 5.3% 44.7%",
  "--sidebar-background": "24 9.8% 10%",
  "--sidebar-foreground": "60 9.1% 97.8%",
  "--sidebar-accent": "12 6.5% 15.1%",
  "--sidebar-accent-foreground": "60 9.1% 97.8%",
  "--sidebar-border": "0 0% 100% / 10%",
  "--sidebar-ring": "25 5.3% 44.7%",
  "--status-added": "#3fb950",
  "--status-deleted": "#f85149",
  "--status-modified": "#d29922",
  "--ui-warning": "#d29922",
  "--ui-warning-bg": "rgba(210, 153, 34, 0.1)",
};

/** Theme names that paint Kubo tokens rather than a derived Shiki palette. */
export const KUBO_THEME_VARS: ReadonlyMap<
  string,
  { isDark: boolean; vars: Readonly<Record<string, string>> }
> = new Map([
  ["kura", { isDark: false, vars: KUBO_LIGHT_VARS }],
  ["kura-dark", { isDark: true, vars: KUBO_DARK_VARS }],
]);

/**
 * The Kubo var map for a theme name, or null when the theme derives its
 * chrome from Shiki colors like every other entry in the picker.
 */
export function getKuboThemeVars(
  name: string,
): { isDark: boolean; vars: Readonly<Record<string, string>> } | null {
  return KUBO_THEME_VARS.get(name) ?? null;
}
