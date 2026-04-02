import { describe, expect, it } from "vitest";

import { deriveBattleState } from "./model";
import type { CardView, DebuggerProtocolEnvelope, PlayerViewState } from "../debugger/protocol";

describe("deriveBattleState", () => {
  it("groups cards into local/opponent zones and contest regions", () => {
    const state = deriveBattleState(buildMessages(), "P1");

    expect(state.local.hand).toHaveLength(1);
    expect(state.local.deck).toHaveLength(1);
    expect(state.local.discard).toHaveLength(1);
    expect(state.local.score).toHaveLength(1);
    expect(state.opponent.handCount).toBe(1);

    expect(state.contest.regions).toHaveLength(3);
    expect(state.contest.regions[0]?.regionCard?.cardId).toBe("region-1");
    expect(state.contest.regions[0]?.localCards.map((card) => card.cardId)).toEqual(["p1-table-1"]);
    expect(state.contest.regions[1]?.opponentCards.map((card) => card.cardId)).toEqual(["p2-table-1"]);
    expect(state.contest.unassignedTableCards.map((card) => card.cardId)).toEqual(["p1-table-free"]);
  });

  it("keeps opponent hand cards hidden for local player", () => {
    const state = deriveBattleState(buildMessages(), "P1");

    expect(state.opponent.handPreview).toHaveLength(1);
    expect(state.opponent.handPreview[0]?.visibility).toBe("hidden");
    expect(state.opponent.handPreview[0]?.cardId).toBeUndefined();
  });

  it("reads opponent markers from latest opponent patch", () => {
    const state = deriveBattleState(buildMessages(), "P1");

    expect(state.local.markers.secret_society).toBe(1);
    expect(state.opponent.markers.secret_society).toBe(2);
  });

  it("returns empty-safe defaults when no state patch exists", () => {
    const state = deriveBattleState([], "P1");

    expect(state.match.status).toBe("active");
    expect(state.local.hand).toHaveLength(0);
    expect(state.opponent.handCount).toBe(0);
    expect(state.contest.regions).toHaveLength(3);
  });
});

function buildMessages(): DebuggerProtocolEnvelope[] {
  const p1Cards: CardView[] = [
    card({ cardId: "p1-hand-1", ownerId: "P1", zone: "hand", visibility: "visible", name: "P1 Hand", kind: "character" }),
    card({ ownerId: "P2", zone: "hand", visibility: "hidden" }),
    card({ ownerId: "P1", zone: "deck", visibility: "hidden" }),
    card({ cardId: "p1-discard-1", ownerId: "P1", zone: "discard", visibility: "visible", name: "P1 Discard" }),
    card({ cardId: "p1-score-1", ownerId: "P1", zone: "score", visibility: "visible", name: "P1 Score" }),
    card({ cardId: "region-1", ownerId: "P1", zone: "table", visibility: "visible", name: "Region 1", kind: "region", regionOrder: 1 }),
    card({ cardId: "region-2", ownerId: "P2", zone: "table", visibility: "visible", name: "Region 2", kind: "region", regionOrder: 2 }),
    card({ cardId: "p1-table-1", ownerId: "P1", zone: "table", visibility: "visible", name: "P1 Unit", kind: "character", regionCardId: "region-1" }),
    card({ cardId: "p2-table-1", ownerId: "P2", zone: "table", visibility: "visible", name: "P2 Unit", kind: "character", regionCardId: "region-2" }),
    card({ cardId: "p1-table-free", ownerId: "P1", zone: "table", visibility: "visible", name: "P1 Free", kind: "asset" })
  ];

  const p2Cards: CardView[] = [
    card({ ownerId: "P1", zone: "hand", visibility: "hidden" }),
    card({ cardId: "p2-hand-1", ownerId: "P2", zone: "hand", visibility: "visible", name: "P2 Hand", kind: "character" })
  ];

  return [
    statePatched("P1", p1Cards, { secret_society: 1 }),
    statePatched("P2", p2Cards, { secret_society: 2 })
  ];
}

function statePatched(
  audienceId: "P1" | "P2",
  cards: CardView[],
  markers: Record<string, number>
): DebuggerProtocolEnvelope {
  const playerView: PlayerViewState = {
    gameId: "game-battle-test",
    viewerPlayerId: audienceId,
    revision: { number: 1 },
    match: { status: "active" },
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
        name: "main",
        step: "action",
        allowsStack: true,
        stepEnded: false
      }
    },
    score: {
      byPlayer: {
        P1: 0,
        P2: 0
      },
      victoryThreshold: 2
    },
    markers,
    board: {
      stack: [],
      resolved: [],
      randomResults: [],
      cards
    }
  };

  return {
    version: "0.1.0",
    kind: "view",
    messageId: `msg-${audienceId}`,
    name: "StatePatched",
    revision: 1,
    payload: {
      type: "StatePatched",
      audienceKind: "player",
      audienceId,
      revision: { number: 1 },
      event: {
        id: `evt-${audienceId}`,
        actionId: `act-${audienceId}`,
        operationId: `op-${audienceId}`,
        kind: "operation_resolved",
        revisionNumber: 1,
        phase: "main",
        step: "action",
        priorityPlayerId: "P1",
        priorityWindow: "action",
        passCount: 0,
        stackDepth: 0
      },
      playerView
    }
  };
}

function card(partial: Partial<CardView>): CardView {
  return {
    ownerId: "P1",
    zone: "hand",
    visibility: "hidden",
    revealed: false,
    exhausted: false,
    destroyed: false,
    stats: {
      combat: 0,
      defense: 0,
      influence: 0,
      investigation: 0
    },
    counters: {
      damage: 0,
      influence: 0
    },
    ...partial
  };
}
