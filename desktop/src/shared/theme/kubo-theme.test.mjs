import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { KUBO_DARK_VARS, KUBO_LIGHT_VARS } from "./kubo-theme.ts";

const THEME_CSS = fileURLToPath(
  new URL("../styles/globals/theme.css", import.meta.url),
);

/** Read the declarations of one top-level block in theme.css. */
function readBlock(css, selector) {
  const start = css.indexOf(`${selector} {`);
  assert.notEqual(start, -1, `theme.css has no ${selector} block`);
  const end = css.indexOf("\n  }", start);
  assert.notEqual(end, -1, `theme.css ${selector} block is unterminated`);
  return Object.fromEntries(
    [...css.slice(start, end).matchAll(/(--[\w-]+):\s*([^;]+);/g)].map(
      ([, name, value]) => [name, value.trim()],
    ),
  );
}

/**
 * The Kura themes are the app's default appearance made selectable, so every
 * token they emit must be byte-identical to the stylesheet. If this fails,
 * theme.css moved and kubo-theme.ts was not updated with it.
 */
test("the Kubo theme vars mirror theme.css", () => {
  const css = readFileSync(THEME_CSS, "utf8");

  for (const [selector, vars] of [
    [":root", KUBO_LIGHT_VARS],
    [".dark", KUBO_DARK_VARS],
  ]) {
    const declared = readBlock(css, selector);
    for (const [name, value] of Object.entries(vars)) {
      // Status and warning colors live only in JS — nothing to mirror.
      if (!(name in declared)) continue;
      assert.equal(
        value,
        declared[name],
        `${name} drifted from theme.css ${selector}`,
      );
    }
  }
});

test("the Kubo theme vars cover every token the derived themes emit", async () => {
  const { createThemeVars } = await import("./adaptive-theme.ts");
  const derived = createThemeVars("#ffffff", "#000000", "#666666");

  for (const name of Object.keys(derived.vars)) {
    assert.ok(
      name in KUBO_LIGHT_VARS,
      `${name} is emitted by derived themes but missing from KUBO_LIGHT_VARS`,
    );
    assert.ok(
      name in KUBO_DARK_VARS,
      `${name} is emitted by derived themes but missing from KUBO_DARK_VARS`,
    );
  }
});
