import assert from "node:assert/strict";
import test from "node:test";

import {
  PRESET_AVATARS,
  parsePresetAvatarDataUrl,
  presetAvatarDataUrl,
} from "./presetAvatars.ts";

test("the gallery ships twenty distinct presets", () => {
  assert.equal(PRESET_AVATARS.length, 20);
  assert.equal(new Set(PRESET_AVATARS.map((p) => p.id)).size, 20);
  assert.equal(new Set(PRESET_AVATARS.map((p) => p.name)).size, 20);
  assert.equal(new Set(PRESET_AVATARS.map((p) => p.svg)).size, 20);

  for (const preset of PRESET_AVATARS) {
    assert.match(preset.id, /^[a-z][a-z-]*$/, `${preset.id} is not a slug`);
    assert.match(preset.svg, /^<svg [^>]*viewBox="0 0 96 96"/);
    assert.match(preset.svg, /<\/svg>$/);
  }
});

/**
 * The persisted form has to stay the percent-encoded one: the base64 branch of
 * `resolveManagedAgentAvatarUrl` would try to upload the avatar to a relay,
 * and a preset must survive agent creation with no network at all.
 */
test("a preset persists as a percent-encoded SVG data URL", () => {
  for (const preset of PRESET_AVATARS) {
    const url = presetAvatarDataUrl(preset);
    assert.ok(url.startsWith("data:image/svg+xml,"));
    assert.ok(!url.includes(";base64"));
    assert.deepEqual(parsePresetAvatarDataUrl(url), preset);
  }
});

test("non-preset avatars do not parse as presets", () => {
  for (const url of [
    "",
    "https://example.test/avatar.png",
    "data:image/png;base64,AAAA",
    "data:image/svg+xml,%3Csvg%3E%3C%2Fsvg%3E",
    "data:image/svg+xml,%E0%A4%A", // malformed percent-encoding
  ]) {
    assert.equal(parsePresetAvatarDataUrl(url), null, url);
  }
});
