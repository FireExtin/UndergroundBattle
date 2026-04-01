import { describe, expect, it } from "vitest";

import { buildActionFromCustomJSON, buildActionFromPreset, liveActionPresets } from "./live";

describe("liveActionPresets", () => {
  it("exposes marker and face-down presets", () => {
    const presetIds = liveActionPresets.map((preset) => preset.id);

    expect(presetIds).toEqual(
      expect.arrayContaining([
        "setSecretSocietyMarker",
        "removeSecretSocietyMarker",
        "setOwnTableFaceDown",
        "useFirstPlayerPrivilege"
      ])
    );
  });
});

describe("buildActionFromPreset", () => {
  it("builds set_marker action that targets acting player", () => {
    const action = buildActionFromPreset("P1", "setSecretSocietyMarker", 7);

    expect(action).toMatchObject({
      id: "act-web-p1-7",
      actorId: "P1",
      kind: "set_marker",
      targetPlayerId: "P1",
      markerType: "secret_society",
      markerAmount: 1
    });
  });

  it("builds remove_marker action that targets acting player", () => {
    const action = buildActionFromPreset("P2", "removeSecretSocietyMarker", 5);

    expect(action).toMatchObject({
      id: "act-web-p2-5",
      actorId: "P2",
      kind: "remove_marker",
      targetPlayerId: "P2",
      markerType: "secret_society",
      markerAmount: 1
    });
  });

  it("builds set_face_down action for acting player's table card", () => {
    const p1Action = buildActionFromPreset("P1", "setOwnTableFaceDown", 3);
    const p2Action = buildActionFromPreset("P2", "setOwnTableFaceDown", 4);

    expect(p1Action).toMatchObject({
      id: "act-web-p1-3",
      actorId: "P1",
      kind: "set_face_down",
      cardId: "P1-TABLE-1"
    });

    expect(p2Action).toMatchObject({
      id: "act-web-p2-4",
      actorId: "P2",
      kind: "set_face_down",
      cardId: "P2-TABLE-1"
    });
  });

  it("builds explicit use_first_player_privilege action", () => {
    const action = buildActionFromPreset("P1", "useFirstPlayerPrivilege", 9);

    expect(action).toMatchObject({
      id: "act-web-p1-9",
      actorId: "P1",
      kind: "use_first_player_privilege"
    });
  });
});

describe("buildActionFromCustomJSON", () => {
  it("fills id and actor defaults when custom JSON omits them", () => {
    const action = buildActionFromCustomJSON("P1", JSON.stringify({ kind: "pass_priority" }), 11);

    expect(action).toMatchObject({
      id: "act-web-p1-11",
      actorId: "P1",
      kind: "pass_priority"
    });
  });

  it("keeps explicit id and actor from custom JSON", () => {
    const action = buildActionFromCustomJSON(
      "P1",
      JSON.stringify({
        id: "act-explicit",
        actorId: "P2",
        kind: "advance_phase"
      }),
      3
    );

    expect(action).toMatchObject({
      id: "act-explicit",
      actorId: "P2",
      kind: "advance_phase"
    });
  });

  it("rejects invalid JSON input", () => {
    expect(() => buildActionFromCustomJSON("P1", "{ not-json", 1)).toThrow(
      "Custom action must be valid JSON."
    );
  });

  it("rejects non-object JSON input", () => {
    expect(() => buildActionFromCustomJSON("P1", JSON.stringify(["pass_priority"]), 1)).toThrow(
      "Custom action JSON must be an object."
    );
  });

  it("rejects object JSON without kind", () => {
    expect(() => buildActionFromCustomJSON("P1", JSON.stringify({ cardId: "P1-TABLE-1" }), 1)).toThrow(
      "Custom action JSON must include a non-empty kind."
    );
  });
});
