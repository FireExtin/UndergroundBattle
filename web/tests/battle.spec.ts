import { expect, test, type APIRequestContext, type Page } from "@playwright/test";

import {
  normalizeColor,
  parseLoyaltyRequirements
} from "../src/battle/actionPolicy";

// Purpose: Exercises setup wizard + battle actions against the authoritative live sandbox.

type PlayerID = "P1" | "P2";

type CardSnapshot = {
  cardId?: string;
  ownerId?: string;
  zone?: string;
  kind?: string;
  color?: string;
  loyalty?: string;
  cost?: number;
  revealed?: boolean;
  faceDown?: boolean;
  destroyed?: boolean;
  regionOrder?: number;
  stats?: {
    investigation?: number;
  };
};

type PlayerViewSnapshot = {
  match: {
    status: "active" | "finished";
    winnerPlayerId?: string;
  };
  turn: {
    priority: {
      currentPlayerId: string;
    };
    resources?: Record<string, { current: number; max: number }>;
  };
  board: {
    cards: CardSnapshot[];
  };
  rulesMetadata?: {
    actionPolicies: Array<unknown>;
    loyalty: {
      colorAliases: Array<{
        canonical: string;
        aliases?: string[];
      }>;
    };
    projection: {
      hiddenCardPreserves: string[];
    };
  };
};

let actionSequence = 1;

test("battle table combo actions: investigation + move + marker + end", async ({ page, request }) => {
  await resetSandboxSession(request);
  await completeSetupViaUI(page);

  const regionIDs = await listRegionCardIDs(request, "P1");
  expect(regionIDs.length).toBeGreaterThanOrEqual(3);

  const p1RegionID = regionIDs[0]!;
  const p2RegionID = regionIDs[1]!;
  const p1CharacterID = await playFirstCharacterFromHand(request, "P1", p1RegionID);

  await ensurePriority(request, "P1");
  await postActionExpectAccepted(request, {
    id: nextActionID("act-e2e-investigate"),
    actorId: "P1",
    kind: "declare_investigation",
    cardId: p1CharacterID,
    targetCardId: p1RegionID
  });

  await postActionExpectAccepted(request, {
    id: nextActionID("act-e2e-move"),
    actorId: "P1",
    kind: "move_card",
    cardId: p1CharacterID,
    targetCardId: p2RegionID
  });

  await page.getByRole("button", { name: "刷新状态" }).click();
  await expect(page.getByText("对局信息日志")).toBeVisible();
  await expect(page.locator(".battle-info-logs__item--accepted").first()).toBeVisible();

  await page.getByLabel("动作类型").selectOption("set_marker");
  await page.getByLabel("目标玩家").selectOption("P1");
  await page.getByLabel("标记类型").fill("secret_society");
  await page.getByLabel("标记数量").fill("1");
  await page.getByRole("button", { name: "提交动作" }).click();
  await page.getByRole("button", { name: "结束行动" }).click();

  await expect(page.getByText("本方玩家区域")).toBeVisible();
  await expect(page.locator(".battle-info-logs__item--accepted").first()).toBeVisible();
});

test("setup wizard rejects duplicate society selections before start", async ({ page, request }) => {
  await resetSandboxSession(request);

  await page.goto("/");
  await expect(page.getByRole("heading", { name: "隐秘世界 开局设置" })).toBeVisible();

  await page.getByLabel("P1 派系 B").selectOption("方碑序列");
  await page.getByRole("button", { name: "开始开局设置" }).click();

  await expect(page.getByText("每位玩家必须选择两个不同派系。")).toBeVisible();
  await expect(page.getByRole("heading", { name: "隐秘世界 开局设置" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "开局设置进行中" })).not.toBeVisible();
});

test("finished match disables actions and latest report endpoint is readable", async ({ page, request }) => {
  await completeSetupViaAPI(request);

  const regionIDs = await listRegionCardIDs(request, "P1");
  expect(regionIDs.length).toBeGreaterThanOrEqual(3);

  await ensurePriority(request, "P1");
  const p1Investigators = await deployInvestigatorsForP1(request, regionIDs.slice(0, 2));
  expect(p1Investigators.length).toBeGreaterThan(0);

  for (const investigator of p1Investigators) {
    await ensurePriority(request, "P1");
    await postActionExpectAccepted(request, {
      id: nextActionID("act-e2e-investigate"),
      actorId: "P1",
      kind: "declare_investigation",
      cardId: investigator.cardId,
      targetCardId: investigator.regionCardId
    });
  }

  await advanceUntilFinished(request, 16);

  await page.goto("/");
  await page.getByRole("button", { name: "刷新状态" }).click();

  await expect(page.getByText("对局结束，胜者：P1").first()).toBeVisible();
  await expect(page.getByRole("button", { name: "提交动作" })).toBeDisabled();

  const reportResponse = await request.get("/api/debugger/reports/latest");
  expect(reportResponse.ok()).toBeTruthy();
  const report = (await reportResponse.json()) as {
    winnerPlayerId?: string;
    content?: string;
  };
  expect(report.winnerPlayerId).toBe("P1");
  expect(report.content ?? "").toContain("# Match Report");
});

async function completeSetupViaUI(page: Page) {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "隐秘世界 开局设置" })).toBeVisible();
  await expect(page.getByText("准备开始《隐秘世界》卡牌游戏，请按规则顺序完成初始设置。")).toBeVisible();

  await page.getByRole("button", { name: "开始开局设置" }).click();
  await expect(page.getByRole("heading", { name: "开局设置进行中" })).toBeVisible();

  for (let step = 1; step <= 7; step += 1) {
    await expect(page.getByText(new RegExp(`当前步骤：第\\s*${step}\\s*/\\s*7\\s*步`))).toBeVisible();
    await page.getByRole("button", { name: "执行下一步" }).click();
  }

  await expect(page.getByRole("heading", { name: "对方玩家区域" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "争夺区" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "本方玩家区域" })).toBeVisible();
  await expect(page.getByText("数据源：实时沙盒")).toBeVisible();
}

async function completeSetupViaAPI(request: APIRequestContext) {
  await resetSandboxSession(request);

  const startResponse = await request.post("/api/battle/setup/start", {
    data: {
      seed: 20260402,
      p1Societies: ["方碑序列", "帷幕守望"],
      p2Societies: ["王座会", "国家机构"]
    }
  });
  expect(startResponse.ok()).toBeTruthy();

  for (let step = 1; step <= 7; step += 1) {
    const payload =
      step === 7
        ? { startingPlayerId: "P1", usePreviousLoserChoice: false }
        : step === 6
          ? { mulliganBottomCount: { P1: 0, P2: 0 } }
          : {};
    const response = await request.post("/api/battle/setup/advance", { data: payload });
    expect(response.ok()).toBeTruthy();
  }
}

async function resetSandboxSession(request: APIRequestContext) {
  const response = await request.post("/api/debugger/reset");
  expect(response.ok()).toBeTruthy();
}

function nextActionID(prefix: string) {
  const id = `${prefix}-${actionSequence}`;
  actionSequence += 1;
  return id;
}

async function listRegionCardIDs(request: APIRequestContext, playerID: PlayerID): Promise<string[]> {
  const view = await latestPlayerView(request, playerID);
  return view.board.cards
    .filter((card) => card.zone === "table" && card.kind === "region" && typeof card.cardId === "string")
    .sort((left, right) => (left.regionOrder ?? 99) - (right.regionOrder ?? 99))
    .map((card) => String(card.cardId));
}

async function ensurePriority(request: APIRequestContext, actorID: PlayerID) {
  for (let index = 0; index < 10; index += 1) {
    const view = await latestPlayerView(request, actorID);
    const currentPriority = toPlayerID(view.turn.priority.currentPlayerId);
    if (currentPriority === actorID) {
      return;
    }

    await postActionExpectAccepted(request, {
      id: nextActionID("act-e2e-pass"),
      actorId: currentPriority,
      kind: "pass_priority"
    });
  }

  throw new Error(`unable to transfer priority to ${actorID}`);
}

async function playFirstCharacterFromHand(
  request: APIRequestContext,
  actorID: PlayerID,
  targetRegionCardID: string,
  options?: { allowFailure?: boolean }
) {
  const cardID = await findAffordableCharacterInHand(request, actorID, options);
  if (!cardID) {
    return "";
  }
  await postActionExpectAccepted(request, {
    id: nextActionID("act-e2e-play"),
    actorId: actorID,
    kind: "play_card",
    cardId: cardID,
    targetRegionCardId: targetRegionCardID,
    playMode: "face_up"
  });
  return cardID;
}

async function deployInvestigatorsForP1(
  request: APIRequestContext,
  regionIDs: string[]
): Promise<Array<{ cardId: string; regionCardId: string }>> {
  const deployed: Array<{ cardId: string; regionCardId: string }> = [];
  for (let index = 0; index < regionIDs.length; index += 1) {
    const cardID = await findAffordableCharacterInHand(request, "P1", { allowFailure: true });
    if (!cardID) {
      break;
    }
    const regionCardID = regionIDs[index]!;
    await postActionExpectAccepted(request, {
      id: nextActionID("act-e2e-play-investigator"),
      actorId: "P1",
      kind: "play_card",
      cardId: cardID,
      targetRegionCardId: regionCardID,
      playMode: "face_up"
    });
    deployed.push({ cardId: cardID, regionCardId: regionCardID });
  }

  return deployed;
}

async function findAffordableCharacterInHand(
  request: APIRequestContext,
  actorID: PlayerID,
  options?: { allowFailure?: boolean }
) {
  for (let step = 0; step < 16; step += 1) {
    await ensurePriority(request, actorID);
    const view = await latestPlayerView(request, actorID);
    const pool = view.turn.resources?.[actorID];
    const currentResource = pool?.current ?? 0;
    const affordable = selectAffordableCharacter(view, actorID, currentResource);
    if (affordable?.cardId) {
      return String(affordable.cardId);
    }

    const priorityActor = toPlayerID(view.turn.priority.currentPlayerId);
    await postActionExpectAccepted(request, {
      id: nextActionID("act-e2e-advance-resource"),
      actorId: priorityActor,
      kind: "advance_phase"
    });
  }

  if (options?.allowFailure) {
    return "";
  }
  throw new Error(`unable to find affordable character for ${actorID}`);
}

function selectAffordableCharacter(
  view: PlayerViewSnapshot,
  actorID: PlayerID,
  currentResource: number
) {
  const availableColors = countAvailableColors(view.board.cards, actorID, view.rulesMetadata);
  const candidates = view.board.cards.filter(
    (card) =>
      card.ownerId === actorID &&
      card.zone === "hand" &&
      card.kind === "character" &&
      typeof card.cardId === "string"
  );
  candidates.sort((left, right) => {
    const leftCost = left.cost ?? 0;
    const rightCost = right.cost ?? 0;
    if (leftCost !== rightCost) {
      return leftCost - rightCost;
    }
    return (right.stats?.investigation ?? 0) - (left.stats?.investigation ?? 0);
  });

  return candidates.find((card) => {
    const cost = card.cost ?? 0;
    if (cost > currentResource) {
      return false;
    }
    return loyaltySatisfied(card.loyalty ?? "", availableColors, view.rulesMetadata);
  });
}

function countAvailableColors(cards: CardSnapshot[], actorID: PlayerID, rulesMetadata?: PlayerViewSnapshot["rulesMetadata"]) {
  const counts: Record<string, number> = {};
  for (const card of cards) {
    if (card.ownerId !== actorID) {
      continue;
    }
    if (card.zone !== "table" || card.destroyed || card.faceDown || !card.revealed) {
      continue;
    }
    if (card.kind !== "character" && card.kind !== "asset") {
      continue;
    }
    const color = normalizeColor(card.color ?? "", rulesMetadata);
    if (!color) {
      continue;
    }
    counts[color] = (counts[color] ?? 0) + 1;
  }
  return counts;
}

function loyaltySatisfied(
  loyalty: string,
  availableColors: Record<string, number>,
  rulesMetadata?: PlayerViewSnapshot["rulesMetadata"]
) {
  const requirements = parseLoyaltyRequirements(loyalty, rulesMetadata);
  for (const [color, amount] of Object.entries(requirements)) {
    if ((availableColors[color] ?? 0) < amount) {
      return false;
    }
  }
  return true;
}

async function advanceUntilFinished(request: APIRequestContext, maxAdvanceActions: number) {
  for (let index = 0; index < maxAdvanceActions; index += 1) {
    const view = await latestPlayerView(request, "P1");
    if (view.match.status === "finished") {
      return;
    }

    await postActionExpectAccepted(request, {
      id: nextActionID("act-e2e-advance"),
      actorId: toPlayerID(view.turn.priority.currentPlayerId),
      kind: "advance_phase"
    });
  }

  const finalView = await latestPlayerView(request, "P1");
  expect(finalView.match.status).toBe("finished");
}

async function latestPlayerView(request: APIRequestContext, playerID: PlayerID): Promise<PlayerViewSnapshot> {
  const response = await request.get("/api/debugger/messages");
  expect(response.ok()).toBeTruthy();
  const messages = (await response.json()) as Array<{
    name?: string;
    payload?: {
      audienceKind?: string;
      audienceId?: string;
      playerView?: PlayerViewSnapshot & { viewerPlayerId?: string };
    };
  }>;

  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message?.name !== "StatePatched") {
      continue;
    }
    const payload = message.payload;
    if (!payload?.playerView) {
      continue;
    }
    if (payload.audienceKind === "player" && payload.audienceId === playerID) {
      return payload.playerView;
    }
    if (payload.playerView.viewerPlayerId === playerID) {
      return payload.playerView;
    }
  }

  throw new Error(`missing player view for ${playerID}`);
}

function toPlayerID(raw: string): PlayerID {
  return raw === "P2" ? "P2" : "P1";
}

async function postActionExpectAccepted(
  request: APIRequestContext,
  action: {
    id: string;
    actorId: PlayerID;
    kind: string;
    cardId?: string;
    targetCardId?: string;
    targetPlayerId?: string;
    targetRegionCardId?: string;
    playMode?: string;
  }
) {
  const response = await request.post("/api/debugger/actions", {
    data: action
  });
  expect(response.ok()).toBeTruthy();

  const messages = (await response.json()) as Array<{
    name?: string;
    payload?: {
      action?: { id?: string };
      legality?: { reasonCode?: string; messageKey?: string };
    };
  }>;
  const accepted = messages.find((message) => message.name === "ActionAccepted");
  if (accepted) {
    return;
  }

  const rejected = messages.find((message) => message.name === "ActionRejected");
  throw new Error(
    `action rejected: ${action.kind} ${rejected?.payload?.legality?.reasonCode ?? "unknown"} ${rejected?.payload?.legality?.messageKey ?? ""}`
  );
}
