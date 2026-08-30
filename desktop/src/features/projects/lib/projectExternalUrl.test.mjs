import assert from "node:assert/strict";
import { test } from "node:test";

import { projectExternalRefUrl } from "./projectExternalUrl.ts";

test("opens the selected GitHub branch", () => {
  assert.equal(
    projectExternalRefUrl(
      "https://github.com/block/kura",
      "fix/agent-profile-about-preserve",
    ),
    "https://github.com/block/kura/tree/fix%2Fagent-profile-about-preserve",
  );
});

test("normalizes clone URLs before adding the selected ref", () => {
  assert.equal(
    projectExternalRefUrl("https://github.com/block/kura.git/", "main"),
    "https://github.com/block/kura/tree/main",
  );
});

test("keeps unsupported and unscoped URLs unchanged", () => {
  assert.equal(
    projectExternalRefUrl("https://gitlab.com/block/kura", "main"),
    "https://gitlab.com/block/kura",
  );
  assert.equal(
    projectExternalRefUrl("https://github.com/block/kura", null),
    "https://github.com/block/kura",
  );
  assert.equal(projectExternalRefUrl("not a URL", "main"), "not a URL");
});
