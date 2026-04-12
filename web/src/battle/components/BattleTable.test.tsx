import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { BattleState } from "../model";
import { BattleTable, formatPlayerValueSummary } from "./BattleTable";

describe("formatPlayerValueSummary", () => {
  it("renders preferred players first and appends unknown player ids dynamically", () => {
    const summary = formatPlayerValueSummary(
      {
        Alpha: 4,
        Beta: 1,
        Gamma: 2
      },
      ["Beta", "Alpha"]
    );

    expect(summary).toBe("Beta 1 · Alpha 4 · Gamma 2");
  });

  it("uses zero-like fallback for preferred players without explicit values", () => {
    const summary = formatPlayerValueSummary(
      {
        Alpha: 3
      },
      ["Beta", "Alpha"]
    );

    expect(summary).toBe("Beta 0 · Alpha 3");
  });
});

describe("BattleTable", () => {
  it("renders projected region controller and influence summary", () => {
    render(
      <BattleTable
        battle={makeBattleState()}
        localPlayerId="P1"
        onLocalPlayerChanged={vi.fn()}
        onCardPicked={vi.fn()}
      />
    );

    expect(screen.getByText("势力值 3 · 分值 2")).toBeVisible();
    expect(screen.getByText("当前控制：P1")).toBeVisible();
    expect(screen.getByText("地区势力：P1 2 · P2 1")).toBeVisible();
  });

  it("renders projected score and winner-facing summary without frontend rule guessing", () => {
    render(
      <BattleTable
        battle={makeBattleState({
          score: {
            byPlayer: { P1: 1, P2: 0 },
            victoryThreshold: 1,
            winnerPlayerId: "P1"
          },
          match: {
            status: "finished",
            winnerPlayerId: "P1"
          }
        })}
        localPlayerId="P1"
        onLocalPlayerChanged={vi.fn()}
        onCardPicked={vi.fn()}
      />
    );

    expect(screen.getByText("分数：P1 1 · P2 0 | 胜利阈值 1")).toBeVisible();
    expect(screen.getByText(/回合 1 \| 当前玩家 P1 \| 优先权 P1/)).toBeVisible();
  });
});

function makeBattleState(overrides?: Partial<BattleState>): BattleState {
  const battle: BattleState = {
    localPlayerId: "P1",
    opponentPlayerId: "P2",
    match: {
      status: "active"
    },
    turn: {
      turnNumber: 1,
      activePlayerId: "P1",
      priorityPlayerId: "P1",
      priority: {
        currentPlayerId: "P1",
        passCount: 0,
        windowKind: "action"
      },
      phase: {
        name: "conflict",
        step: "action",
        allowsStack: true,
        stepEnded: false
      },
      conflict: {
        stage: "pre_battle_fast",
        regionCardId: "region-1",
        priorityLeaderPlayerId: "P1"
      },
      pendingPrompt: null,
      resources: {
        P1: { current: 1, max: 1 },
        P2: { current: 1, max: 1 }
      }
    },
    score: {
      byPlayer: {
        P1: 0,
        P2: 0
      },
      victoryThreshold: 2
    },
    rulesMetadata: {
      actionPolicies: [],
      loyalty: { colorAliases: [] },
      projection: { hiddenCardPreserves: [] }
    },
    local: {
      hand: [],
      deck: [],
      assets: [],
      discard: [],
      score: [],
      table: [],
      markers: {}
    },
    opponent: {
      handCount: 0,
      handPreview: [],
      deckCount: 0,
      assetCount: 0,
      assets: [],
      discardCount: 0,
      scoreCount: 0,
      table: [],
      markers: {}
    },
    contest: {
      regions: [
        {
          slotId: "slot-1",
          order: 1,
          regionCard: {
            cardId: "region-1",
            ownerId: "TABLE",
            zone: "table",
            kind: "region",
            name: "Region One",
            visibility: "visible",
            revealed: true,
            exhausted: false,
            destroyed: false,
            regionOrder: 1,
            regionScore: 2,
            controllerId: "P1",
            influenceByPlayer: { P1: 2, P2: 1 },
            stats: { influence: 3, combat: 0, defense: 0, investigation: 0 },
            counters: { damage: 0, influence: 3, shield: 0 }
          },
          localCards: [],
          opponentCards: []
        }
      ],
      unassignedTableCards: []
    },
    actionCandidates: {
      localTableCardIds: [],
      opponentTableCardIds: [],
      regionCardIds: ["region-1"],
      localHandCardIds: [],
      visibleCardIds: ["region-1"]
    }
  };

  return {
    ...battle,
    ...overrides,
    turn: {
      ...battle.turn,
      ...overrides?.turn
    },
    score: {
      ...battle.score,
      ...overrides?.score
    },
    match: {
      ...battle.match,
      ...overrides?.match
    }
  };
}
