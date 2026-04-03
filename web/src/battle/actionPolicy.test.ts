import { describe, it, expect } from "vitest";
import { validateActionInput, parseLoyaltyRequirements } from "./actionPolicy";
import type { RulesMetadata } from "../debugger/protocol";

describe("actionPolicy", () => {
  const mockMetadata: RulesMetadata = {
    actionPolicies: [
      {
        actionKind: "play_card",
        actorConstraint: "priority_player",
        requiresPriority: true,
        requiresEmptyStack: true,
        requiresActionWindow: true,
        fieldRules: [
          { field: "cardId", requirement: "required" },
          { field: "targetRegionCardId", requirement: "required", sourceKinds: ["character"] }
        ]
      }
    ],
    loyalty: {
      colorAliases: [
        { canonical: "方碑序列", aliases: ["方碑"] },
        { canonical: "帷幕守望", aliases: ["帷幕"] }
      ]
    },
    projection: { hiddenCardPreserves: ["regionCardId"] }
  };

  describe("validateActionInput", () => {
    it("should return error when required field is missing", () => {
      const result = validateActionInput(mockMetadata, {
        actionKind: "play_card",
        sourceCardId: "",
        targetCardId: "",
        targetRegionCardId: "",
        targetPlayerId: "",
        markerType: "",
        markerAmount: "0",
        playMode: "face_up",
        sourceCardKind: "character"
      });
      expect(result).toBe("需要选择来源卡牌");
    });

    it("should return error when kind-specific required field is missing", () => {
      const result = validateActionInput(mockMetadata, {
        actionKind: "play_card",
        sourceCardId: "c1",
        targetCardId: "",
        targetRegionCardId: "",
        targetPlayerId: "",
        markerType: "",
        markerAmount: "0",
        playMode: "face_up",
        sourceCardKind: "character"
      });
      expect(result).toBe("需要选择部署地区");
    });

    it("should pass when all required fields are present", () => {
      const result = validateActionInput(mockMetadata, {
        actionKind: "play_card",
        sourceCardId: "c1",
        targetCardId: "",
        targetRegionCardId: "r1",
        targetPlayerId: "",
        markerType: "",
        markerAmount: "0",
        playMode: "face_up",
        sourceCardKind: "character"
      });
      expect(result).toBe("");
    });
  });

  describe("parseLoyaltyRequirements", () => {
    it("should parse complex loyalty strings using aliases", () => {
      const result = parseLoyaltyRequirements("方碑方碑帷幕", mockMetadata);
      expect(result).toEqual({
        "方碑序列": 2,
        "帷幕守望": 1
      });
    });

    it("should handle mixed canonical and alias tokens", () => {
      const result = parseLoyaltyRequirements("方碑序列方碑", mockMetadata);
      expect(result).toEqual({
        "方碑序列": 2
      });
    });
  });
});
