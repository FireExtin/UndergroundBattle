import { describe, expect, it } from "vitest";

import {
  normalizeColor,
  parseLoyaltyRequirements,
  validateActionInput
} from "./actionPolicy";
import type { RulesMetadata } from "../debugger/protocol";

const metadata: RulesMetadata = {
  actionPolicies: [
    {
      actionKind: "play_card",
      actorConstraint: "priority_player",
      requiresPriority: true,
      requiresEmptyStack: false,
      fieldRules: [
        { field: "cardId", requirement: "required" },
        {
          field: "targetRegionCardId",
          requirement: "required",
          sourceKinds: ["character"]
        },
        {
          field: "targetCardId",
          requirement: "required",
          sourceKinds: ["asset"]
        }
      ]
    }
  ],
  loyalty: {
    colorAliases: [
      { canonical: "黄色", aliases: ["黄"] },
      { canonical: "蓝色", aliases: ["蓝"] }
    ]
  },
  projection: {
    hiddenCardPreserves: ["ownerId", "zone", "regionCardId", "faceDown"]
  }
};

describe("validateActionInput", () => {
  it("requires target region for character play_card from policy", () => {
    expect(
      validateActionInput(metadata, {
        actionKind: "play_card",
        sourceCardKind: "character",
        sourceCardId: "card-1",
        targetCardId: "",
        targetRegionCardId: "",
        targetPlayerId: "",
        markerType: "",
        markerAmount: "1",
        playMode: "face_up"
      })
    ).toBe("需要选择部署地区");
  });

  it("requires target card for asset play_card from policy", () => {
    expect(
      validateActionInput(metadata, {
        actionKind: "play_card",
        sourceCardKind: "asset",
        sourceCardId: "card-1",
        targetCardId: "",
        targetRegionCardId: "",
        targetPlayerId: "",
        markerType: "",
        markerAmount: "1",
        playMode: "face_up"
      })
    ).toBe("需要选择目标卡牌");
  });
});

describe("loyalty metadata helpers", () => {
  it("counts mixed canonical and alias loyalty tokens from metadata", () => {
    expect(parseLoyaltyRequirements("黄黄色", metadata)).toEqual({ 黄色: 2 });
  });

  it("normalizes alias colors through metadata", () => {
    expect(normalizeColor("黄", metadata)).toBe("黄色");
    expect(normalizeColor("中立", metadata)).toBe("");
  });
});
