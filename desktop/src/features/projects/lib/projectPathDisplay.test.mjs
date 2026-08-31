import assert from "node:assert/strict";
import test from "node:test";

import { shortenProjectPath } from "./projectPathDisplay.ts";

test("keeps short repository paths intact", () => {
  assert.equal(shortenProjectPath("repos/kura"), "repos/kura");
});

test("shortens long repository paths to their trailing segments", () => {
  assert.equal(
    shortenProjectPath("/Users/thomasp/sprout/projects/kura"),
    "…/sprout/projects/kura",
  );
});

test("normalizes Windows separators for display", () => {
  assert.equal(
    shortenProjectPath("C:\\Users\\thomasp\\repos\\kura"),
    "…/thomasp/repos/kura",
  );
});
