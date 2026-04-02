import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { BattleShell } from "./BattleShell";
import type { CardView, DebuggerProtocolEnvelope, PlayerViewState } from "../debugger/protocol";

describe("BattleShell", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("loads live messages and renders battle zones", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(buildMessages()));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    await screen.findByText("对方玩家区域");
    expect(screen.getByText("争夺区")).toBeInTheDocument();
    expect(screen.getByText("本方玩家区域")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith("/api/debugger/messages", undefined);
  });

  it("submits declare_attack action from composer", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(buildMessages()))
      .mockResolvedValueOnce(createJSONResponse(buildMessages()));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("Action Kind"), {
      target: { value: "declare_attack" }
    });
    fireEvent.change(screen.getByLabelText("Source Card"), {
      target: { value: "p1-table-1" }
    });
    fireEvent.change(screen.getByLabelText("Target Card"), {
      target: { value: "p2-table-1" }
    });
    fireEvent.click(screen.getByRole("button", { name: "提交动作" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/actions");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "POST"
    });

    const body = JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body));
    expect(body).toMatchObject({
      actorId: "P1",
      kind: "declare_attack",
      cardId: "p1-table-1",
      targetCardId: "p2-table-1"
    });
  });

  it("disables action submit when match is finished", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(buildMessages({ finished: true })));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("Game over. Winner: P1")).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "提交动作" })).toBeDisabled();
  });

  it("resets the sandbox from battle controls", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(createJSONResponse(buildMessages({ finished: true })))
      .mockResolvedValueOnce(createJSONResponse(buildMessages()));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("Game over. Winner: P1")).length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "重开对局" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/reset");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "POST"
    });
  });

  it("falls back to offline message when live server is unavailable", async () => {
    const fetchMock = vi.fn().mockRejectedValue(new Error("offline"));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("Live server unavailable. Showing mock protocol data.")).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "提交动作" })).toBeDisabled();
  });

  it("auto-fills source and target from table card clicks", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(buildMessages()));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("Action Kind"), {
      target: { value: "declare_attack" }
    });

    fireEvent.click(screen.getAllByRole("button", { name: /P1 Unit/ })[0]!);
    expect(screen.getByLabelText("Source Card")).toHaveValue("p1-table-1");

    fireEvent.click(screen.getAllByRole("button", { name: /P2 Unit/ })[0]!);
    expect(screen.getByLabelText("Target Card")).toHaveValue("p2-table-1");
  });

  it("keeps manual source selection when auto-fill would otherwise override it", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(createJSONResponse(buildMessages({ includeExtraLocalTable: true })));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.click(screen.getAllByRole("button", { name: /P1 Unit/ })[0]!);
    expect(screen.getByLabelText("Source Card")).toHaveValue("p1-table-1");

    fireEvent.change(screen.getByLabelText("Source Card"), {
      target: { value: "p1-table-2" }
    });
    expect(screen.getByLabelText("Source Card")).toHaveValue("p1-table-2");

    fireEvent.click(screen.getAllByRole("button", { name: /P1 Unit/ })[0]!);
    expect(screen.getByLabelText("Source Card")).toHaveValue("p1-table-2");
  });

  it("shows action docs and supports info log filtering", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(createJSONResponse(buildMessages({ includeActionLogs: true })));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    expect(screen.getByText("动作说明")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "查看完整说明" })).toBeInTheDocument();

    expect(screen.getByText(/ACCEPTED/)).toBeInTheDocument();
    expect(screen.getByText(/REJECTED/)).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("日志过滤"), {
      target: { value: "rejected" }
    });
    expect(screen.getByText(/REJECTED/)).toBeInTheDocument();
    expect(screen.queryByText(/ACCEPTED/)).not.toBeInTheDocument();
  });

  it("clears stale source card when switching actor perspective", async () => {
    const fetchMock = vi.fn().mockResolvedValue(createJSONResponse(buildMessages()));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("Action Kind"), {
      target: { value: "declare_attack" }
    });
    fireEvent.change(screen.getByLabelText("Source Card"), {
      target: { value: "p1-table-1" }
    });
    expect(screen.getByLabelText("Source Card")).toHaveValue("p1-table-1");

    fireEvent.click(screen.getByRole("button", { name: "P2 视角" }));
    expect(screen.getByLabelText("Source Card")).toHaveValue("");
  });
});

function buildMessages(options?: {
  finished?: boolean;
  includeActionLogs?: boolean;
  includeExtraLocalTable?: boolean;
}): DebuggerProtocolEnvelope[] {
  const finished = options?.finished === true;
  const includeActionLogs = options?.includeActionLogs === true;
  const includeExtraLocalTable = options?.includeExtraLocalTable === true;

  const envelopes: DebuggerProtocolEnvelope[] = [];
  if (includeActionLogs) {
    envelopes.push(
      accepted("P1", "declare_attack", "damage_applied", 1),
      rejected("P2", "declare_attack", "LEGALITY_FAILED_NOT_YOUR_PRIORITY")
    );
  }

  const p1Cards = [
    card({ cardId: "p1-hand-1", ownerId: "P1", zone: "hand", visibility: "visible", name: "P1 Hand", kind: "character" }),
    card({ ownerId: "P2", zone: "hand", visibility: "hidden" }),
    card({ cardId: "region-1", ownerId: "P1", zone: "table", visibility: "visible", name: "Region 1", kind: "region", regionOrder: 1 }),
    card({ cardId: "region-2", ownerId: "P2", zone: "table", visibility: "visible", name: "Region 2", kind: "region", regionOrder: 2 }),
    card({ cardId: "region-3", ownerId: "P1", zone: "table", visibility: "visible", name: "Region 3", kind: "region", regionOrder: 3 }),
    card({ cardId: "p1-table-1", ownerId: "P1", zone: "table", visibility: "visible", name: "P1 Unit", kind: "character", regionCardId: "region-1" }),
    card({ cardId: "p2-table-1", ownerId: "P2", zone: "table", visibility: "visible", name: "P2 Unit", kind: "character", regionCardId: "region-2" })
  ];
  if (includeExtraLocalTable) {
    p1Cards.push(
      card({
        cardId: "p1-table-2",
        ownerId: "P1",
        zone: "table",
        visibility: "visible",
        name: "P1 Backup",
        kind: "character",
        regionCardId: "region-3"
      })
    );
  }

  envelopes.push(
    statePatched("P1", [
      ...p1Cards
    ], { secret_society: 1 }, finished),
    statePatched("P2", [
      card({ ownerId: "P1", zone: "hand", visibility: "hidden" }),
      card({ cardId: "p2-hand-1", ownerId: "P2", zone: "hand", visibility: "visible", name: "P2 Hand", kind: "character" })
    ], { secret_society: 2 }, finished)
  );

  return envelopes;
}

function statePatched(
  audienceId: "P1" | "P2",
  cards: CardView[],
  markers: Record<string, number>,
  finished: boolean
): DebuggerProtocolEnvelope {
  const playerView: PlayerViewState = {
    gameId: "game-battle-shell-test",
    viewerPlayerId: audienceId,
    revision: { number: 1 },
    match: finished
      ? {
          status: "finished",
          winnerPlayerId: "P1",
          endReason: "victory_threshold",
          finishedAtRevision: 1
        }
      : { status: "active" },
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
        P1: finished ? 2 : 0,
        P2: 0
      },
      victoryThreshold: 2,
      winnerPlayerId: finished ? "P1" : undefined
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

function createJSONResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: {
      "Content-Type": "application/json"
    }
  });
}

function accepted(
  actorId: "P1" | "P2",
  actionKind: string,
  eventKind: string,
  revision: number
): DebuggerProtocolEnvelope {
  return {
    version: "0.1.0",
    kind: "event",
    messageId: `msg-accepted-${actorId}-${revision}`,
    name: "ActionAccepted",
    revision,
    payload: {
      type: "ActionAccepted",
      action: {
        id: `act-${actorId}-${revision}`,
        actorId,
        kind: actionKind
      },
      operation: {
        id: `op-${actorId}-${revision}`,
        actionId: `act-${actorId}-${revision}`,
        actorId,
        kind: actionKind,
        status: "resolved",
        requiresStack: false
      },
      event: {
        id: `evt-${actorId}-${revision}`,
        actionId: `act-${actorId}-${revision}`,
        operationId: `op-${actorId}-${revision}`,
        kind: eventKind,
        revisionNumber: revision,
        phase: "main",
        step: "action",
        priorityPlayerId: actorId,
        priorityWindow: "action",
        passCount: 0,
        stackDepth: 0
      },
      revision: {
        number: revision
      }
    }
  };
}

function rejected(
  actorId: "P1" | "P2",
  actionKind: string,
  reasonCode: string
): DebuggerProtocolEnvelope {
  return {
    version: "0.1.0",
    kind: "event",
    messageId: `msg-rejected-${actorId}`,
    name: "ActionRejected",
    payload: {
      type: "ActionRejected",
      action: {
        id: `act-rejected-${actorId}`,
        actorId,
        kind: actionKind
      },
      legality: {
        ok: false,
        reasonCode,
        messageKey: "rules.legality.not_your_priority"
      }
    }
  };
}
