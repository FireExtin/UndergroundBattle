import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { BattleShell } from "./BattleShell";
import type { CardView, DebuggerProtocolEnvelope, PlayerViewState } from "../debugger/protocol";

describe("BattleShell", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("loads setup and then renders battle zones", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    await screen.findByText("对方玩家区域");
    expect(screen.getByText("争夺区")).toBeInTheDocument();
    expect(screen.getByText("本方玩家区域")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith("/api/battle/setup/state", undefined);
    expect(fetchMock).toHaveBeenCalledWith("/api/debugger/messages", undefined);
  });

  it("submits declare_attack action from composer", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "declare_attack" }
    });
    fireEvent.change(screen.getByLabelText("来源卡牌"), {
      target: { value: "p1-table-1" }
    });
    fireEvent.change(screen.getByLabelText("目标卡牌"), {
      target: { value: "p2-table-1" }
    });
    fireEvent.click(screen.getByRole("button", { name: "提交动作" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/debugger/actions",
        expect.objectContaining({
          method: "POST"
        })
      );
    });

    const call = fetchMock.mock.calls.find((args) => args[0] === "/api/debugger/actions");
    const body = JSON.parse(String(call?.[1]?.body));
    expect(body).toMatchObject({
      actorId: "P1",
      kind: "declare_attack",
      cardId: "p1-table-1",
      targetCardId: "p2-table-1"
    });
  });

  it("disables action submit when match is finished", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages({ finished: true })
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("对局结束，胜者：P1")).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "提交动作" })).toBeDisabled();
  });

  it("resets to setup wizard from battle controls", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState(), inactiveSetupState()],
      messages: buildMessages({ finished: true })
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("对局结束，胜者：P1")).length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "重置并返回开局设置" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/debugger/reset",
        expect.objectContaining({ method: "POST" })
      );
    });

    expect(await screen.findByText("隐秘世界 开局设置")).toBeInTheDocument();
  });

  it("shows setup error when live server is unavailable", async () => {
    const fetchMock = vi.fn().mockRejectedValue(new Error("offline"));
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);

    expect((await screen.findAllByText("无法连接服务端，已切换离线协议演示。")).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "开始开局设置" })).toBeInTheDocument();
  });

  it("auto-fills source and target from table card clicks", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "declare_attack" }
    });

    fireEvent.click(screen.getAllByRole("button", { name: /P1 Unit/ })[0]!);
    expect(screen.getByLabelText("来源卡牌")).toHaveValue("p1-table-1");

    fireEvent.click(screen.getAllByRole("button", { name: /P2 Unit/ })[0]!);
    expect(screen.getByLabelText("目标卡牌")).toHaveValue("p2-table-1");
  });

  it("uses region click to auto-fill play_card target region", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "play_card" }
    });
    fireEvent.click(screen.getAllByRole("button", { name: /P1 Hand/ })[0]!);
    expect(screen.getByLabelText("来源卡牌")).toHaveValue("p1-hand-1");

    fireEvent.click(screen.getAllByRole("button", { name: /Region 1/ })[0]!);
    expect(screen.getByLabelText("部署地区")).toHaveValue("region-1");
  });

  it("shows action docs and supports info log filtering", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages({ includeActionLogs: true })
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    expect(screen.getByText("动作说明")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "查看完整说明" })).toBeInTheDocument();

    expect(screen.getByText(/已接受 P1/)).toBeInTheDocument();
    expect(screen.getByText(/已拒绝 P2/)).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("日志过滤"), {
      target: { value: "rejected" }
    });
    expect(screen.getByText(/已拒绝 P2/)).toBeInTheDocument();
    expect(screen.queryByText(/已接受 P1/)).not.toBeInTheDocument();
  });

  it("clears stale source card when switching actor perspective", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "declare_attack" }
    });
    fireEvent.change(screen.getByLabelText("来源卡牌"), {
      target: { value: "p1-table-1" }
    });
    expect(screen.getByLabelText("来源卡牌")).toHaveValue("p1-table-1");

    fireEvent.click(screen.getByRole("button", { name: "P2 视角" }));
    expect(screen.getByLabelText("来源卡牌")).toHaveValue("");
  });

  it("auto-submits pass then advance when end action closes the step", async () => {
    const actionResponses = [
      [
        accepted("P1", "pass_priority", "step_ended", 2),
        statePatched("P1", buildMessagesCards(), { secret_society: 1 }, false, {
          priorityPlayerId: "P1",
          phaseStep: "ended"
        })
      ],
      [
        accepted("P1", "advance_phase", "phase_advanced", 3),
        statePatched("P1", buildMessagesCards(), { secret_society: 1 }, false, {
          priorityPlayerId: "P2",
          phaseName: "end",
          phaseStep: "action"
        })
      ]
    ];
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages(),
      actionResponses
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.click(screen.getByRole("button", { name: "结束行动" }));

    await waitFor(() => {
      const calls = fetchMock.mock.calls.filter((args) => args[0] === "/api/debugger/actions");
      expect(calls).toHaveLength(2);
    });
  });

  it("renders dedicated asset zones for both players", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages({ includeAssets: true })
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    expect(screen.getAllByText("对方资产区").length).toBeGreaterThan(0);
    expect(screen.getAllByText("本方资产区").length).toBeGreaterThan(0);
  });

  it("submits establish asset shortcut as build_asset without extra target", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages({ includeAssets: true })
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "build_asset" }
    });
    fireEvent.change(screen.getByLabelText("来源卡牌"), {
      target: { value: "p1-hand-asset-1" }
    });
    fireEvent.change(screen.getByLabelText("目标卡牌"), {
      target: { value: "p1-table-1" }
    });
    fireEvent.click(screen.getByRole("button", { name: "提交动作" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/debugger/actions",
        expect.objectContaining({
          method: "POST"
        })
      );
    });

    const call = fetchMock.mock.calls.find((args) => args[0] === "/api/debugger/actions");
    const body = JSON.parse(String(call?.[1]?.body));
    expect(body).toMatchObject({
      actorId: "P1",
      kind: "build_asset",
      cardId: "p1-hand-asset-1"
    });
    expect(body.targetCardId).toBeUndefined();
    expect(body.targetRegionCardId).toBeUndefined();
  });

  it("allows establish asset shortcut for character-style deploy flow", async () => {
    const fetchMock = createBattleFetchMock({
      setupStates: [completedSetupState()],
      messages: buildMessages()
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<BattleShell fallbackMessageSets={[]} />);
    await screen.findByText("对方玩家区域");

    fireEvent.change(screen.getByLabelText("动作类型"), {
      target: { value: "build_asset" }
    });
    fireEvent.change(screen.getByLabelText("来源卡牌"), {
      target: { value: "p1-hand-1" }
    });
    fireEvent.change(screen.getByLabelText("部署地区"), {
      target: { value: "region-1" }
    });
    fireEvent.click(screen.getByRole("button", { name: "提交动作" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        "/api/debugger/actions",
        expect.objectContaining({
          method: "POST"
        })
      );
    });

    const call = fetchMock.mock.calls.find((args) => args[0] === "/api/debugger/actions");
    const body = JSON.parse(String(call?.[1]?.body));
    expect(body).toMatchObject({
      actorId: "P1",
      kind: "build_asset",
      cardId: "p1-hand-1"
    });
    expect(body.targetCardId).toBeUndefined();
    expect(body.targetRegionCardId).toBeUndefined();
  });
});

function createBattleFetchMock(options?: {
  setupStates?: Array<Record<string, unknown>>;
  messages?: DebuggerProtocolEnvelope[];
  actionMessages?: DebuggerProtocolEnvelope[];
  actionResponses?: DebuggerProtocolEnvelope[][];
}) {
  const setupStates = options?.setupStates ?? [completedSetupState()];
  const messages = options?.messages ?? buildMessages();
  const actionMessages = options?.actionMessages ?? messages;
  const actionResponses = options?.actionResponses ?? [actionMessages];
  let setupIndex = 0;
  let actionIndex = 0;

  return vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url === "/api/battle/setup/state") {
      const payload = setupStates[Math.min(setupIndex, setupStates.length - 1)] ?? inactiveSetupState();
      setupIndex += 1;
      return createJSONResponse(payload);
    }
    if (url === "/api/debugger/messages") {
      return createJSONResponse(messages);
    }
    if (url === "/api/debugger/actions") {
      const payload = actionResponses[Math.min(actionIndex, actionResponses.length - 1)] ?? actionMessages;
      actionIndex += 1;
      return createJSONResponse(payload);
    }
    if (url === "/api/debugger/reset") {
      return createJSONResponse([]);
    }
    if (url === "/api/battle/setup/start") {
      return createJSONResponse(completedSetupState());
    }
    if (url === "/api/battle/setup/advance") {
      return createJSONResponse(completedSetupState());
    }

    return Promise.reject(new Error(`unexpected fetch: ${url} ${init?.method ?? "GET"}`));
  });
}

function completedSetupState() {
  return {
    active: true,
    completed: true,
    currentStep: 7,
    seed: 20260402,
    steps: [
      { step: 1, title: "玩家选择牌组", completed: true },
      { step: 2, title: "设置世界牌库", completed: true },
      { step: 3, title: "整理标志", completed: true },
      { step: 4, title: "设置玩家牌库", completed: true },
      { step: 5, title: "翻开地区牌", completed: true },
      { step: 6, title: "抓取起始手牌", completed: true },
      { step: 7, title: "确定先手玩家", completed: true }
    ],
    p1Societies: ["方碑序列", "帷幕守望"],
    p2Societies: ["王座会", "国家机构"],
    markerPoolReady: true,
    worldDeckCount: 7,
    playerDeckCount: { P1: 44, P2: 44 },
    playerHandCount: { P1: 6, P2: 6 },
    mulliganUsed: { P1: false, P2: false },
    startingPlayerId: "P1"
  };
}

function inactiveSetupState() {
  return {
    active: false,
    completed: false,
    currentStep: 1,
    seed: 20260402,
    steps: [],
    markerPoolReady: false,
    worldDeckCount: 0,
    playerDeckCount: { P1: 0, P2: 0 },
    playerHandCount: { P1: 0, P2: 0 },
    mulliganUsed: { P1: false, P2: false }
  };
}

function buildMessages(options?: {
  finished?: boolean;
  includeActionLogs?: boolean;
  includeExtraLocalTable?: boolean;
  includeAssets?: boolean;
}): DebuggerProtocolEnvelope[] {
  const finished = options?.finished === true;
  const includeActionLogs = options?.includeActionLogs === true;
  const includeExtraLocalTable = options?.includeExtraLocalTable === true;
  const includeAssets = options?.includeAssets === true;

  const envelopes: DebuggerProtocolEnvelope[] = [];
  if (includeActionLogs) {
    envelopes.push(
      accepted("P1", "declare_attack", "damage_applied", 1),
      rejected("P2", "declare_attack", "LEGALITY_FAILED_NOT_YOUR_PRIORITY")
    );
  }

  const p1Cards = buildMessagesCards(includeExtraLocalTable, includeAssets);

  envelopes.push(
    statePatched("P1", [...p1Cards], { secret_society: 1 }, finished),
    statePatched(
      "P2",
      [
        card({ ownerId: "P1", zone: "hand", visibility: "hidden" }),
        card({
          cardId: "p2-hand-1",
          ownerId: "P2",
          zone: "hand",
          visibility: "visible",
          name: "P2 Hand",
          kind: "character"
        })
      ],
      { secret_society: 2 },
      finished
    )
  );

  return envelopes;
}

function statePatched(
  audienceId: "P1" | "P2",
  cards: CardView[],
  markers: Record<string, number>,
  finished: boolean,
  options?: {
    priorityPlayerId?: "P1" | "P2";
    phaseName?: "main" | "end";
    phaseStep?: "action" | "ended";
  }
): DebuggerProtocolEnvelope {
  const priorityPlayerId = options?.priorityPlayerId ?? "P1";
  const phaseName = options?.phaseName ?? "main";
  const phaseStep = options?.phaseStep ?? "action";

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
      priorityPlayerId,
      priority: {
        currentPlayerId: priorityPlayerId,
        passCount: 0,
        windowKind: phaseStep === "ended" ? "closed" : "action"
      },
      phase: {
        name: phaseName,
        step: phaseStep,
        allowsStack: phaseName === "main",
        stepEnded: phaseStep === "ended"
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
        phase: phaseName,
        step: phaseStep,
        priorityPlayerId,
        priorityWindow: phaseStep === "ended" ? "closed" : "action",
        passCount: 0,
        stackDepth: 0
      },
      playerView
    }
  };
}

function buildMessagesCards(includeExtraLocalTable = false, includeAssets = false): CardView[] {
  const cards = [
    card({
      cardId: "p1-hand-1",
      ownerId: "P1",
      zone: "hand",
      visibility: "visible",
      name: "P1 Hand",
      kind: "character"
    }),
    card({ ownerId: "P2", zone: "hand", visibility: "hidden" }),
    card({
      cardId: "region-1",
      ownerId: "P1",
      zone: "table",
      visibility: "visible",
      name: "Region 1",
      kind: "region",
      regionOrder: 1
    }),
    card({
      cardId: "region-2",
      ownerId: "P2",
      zone: "table",
      visibility: "visible",
      name: "Region 2",
      kind: "region",
      regionOrder: 2
    }),
    card({
      cardId: "region-3",
      ownerId: "P1",
      zone: "table",
      visibility: "visible",
      name: "Region 3",
      kind: "region",
      regionOrder: 3
    }),
    card({
      cardId: "p1-table-1",
      ownerId: "P1",
      zone: "table",
      visibility: "visible",
      name: "P1 Unit",
      kind: "character",
      regionCardId: "region-1"
    }),
    card({
      cardId: "p2-table-1",
      ownerId: "P2",
      zone: "table",
      visibility: "visible",
      name: "P2 Unit",
      kind: "character",
      regionCardId: "region-2"
    })
  ];

  if (includeAssets) {
    cards.push(
      card({
        cardId: "p1-hand-asset-1",
        ownerId: "P1",
        zone: "hand",
        visibility: "visible",
        name: "P1 资产手牌",
        kind: "asset"
      }),
      card({
        cardId: "p1-asset-1",
        ownerId: "P1",
        zone: "table",
        visibility: "visible",
        name: "P1 资产",
        kind: "asset",
        regionCardId: "region-1"
      }),
      card({
        cardId: "p2-asset-1",
        ownerId: "P2",
        zone: "table",
        visibility: "visible",
        name: "P2 资产",
        kind: "asset",
        regionCardId: "region-2"
      })
    );
  }

  if (includeExtraLocalTable) {
    cards.push(
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

  return cards;
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
