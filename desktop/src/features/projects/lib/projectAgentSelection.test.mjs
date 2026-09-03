import assert from "node:assert/strict";
import test from "node:test";

import { pickDefaultProjectsAgent } from "./projectAgentSelection.ts";

test("prefers Hayate over the first running agent", () => {
  const implementationPartner = {
    name: "Implementation Partner",
    personaId: "custom:implementation",
  };
  const hayate = { name: "Hayate", personaId: "builtin:hayate" };
  assert.equal(
    pickDefaultProjectsAgent([implementationPartner, hayate]),
    hayate,
  );
});

test("ignores an unmanaged agent using the Hayate display name", () => {
  const managed = { name: "Builder", personaId: "custom:builder" };
  const spoofedHayate = { name: "Hayate" };
  assert.equal(pickDefaultProjectsAgent([managed, spoofedHayate]), managed);
  assert.equal(pickDefaultProjectsAgent([managed]), managed);
  assert.equal(pickDefaultProjectsAgent([]), null);
});
