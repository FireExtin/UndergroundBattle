import { selectCurrentCards, selectCurrentPatch } from "../debugger/model";
import type {
  ActionAcceptedEnvelope,
  ActionRejectedEnvelope,
  CardView,
  DebuggerProtocolEnvelope,
  MatchState,
  RulesMetadata,
  ScoreState,
  TurnState,
  ViewerId
} from "../debugger/protocol";

// Purpose: Derives a board-game oriented battle-table view model from protocol envelopes without duplicating server rules.

export type BattlePlayerId = Exclude<ViewerId, "spectator">;

export type BattlePlayerArea = {
  hand: CardView[];
  deck: CardView[];
  assets: CardView[];
  discard: CardView[];
  score: CardView[];
  table: CardView[];
  markers: Record<string, number>;
};

export type BattleOpponentArea = {
  handCount: number;
  handPreview: CardView[];
  deckCount: number;
  assetCount: number;
  assets: CardView[];
  discardCount: number;
  scoreCount: number;
  table: CardView[];
  markers: Record<string, number>;
};

export type BattleRegionSlot = {
  slotId: string;
  order: number;
  regionCard?: CardView;
  localCards: CardView[];
  opponentCards: CardView[];
};

export type BattleContestState = {
  regions: BattleRegionSlot[];
  unassignedTableCards: CardView[];
};

export type BattleActionCandidates = {
  localTableCardIds: string[];
  opponentTableCardIds: string[];
  regionCardIds: string[];
  localHandCardIds: string[];
  visibleCardIds: string[];
};

export type BattleState = {
  localPlayerId: BattlePlayerId;
  opponentPlayerId: BattlePlayerId;
  match: MatchState;
  turn: TurnState;
  score: ScoreState;
  rulesMetadata: RulesMetadata;
  local: BattlePlayerArea;
  opponent: BattleOpponentArea;
  contest: BattleContestState;
  actionCandidates: BattleActionCandidates;
};

export type BattleInfoLogKind = "accepted" | "rejected" | "system";

export type BattleInfoLogEntry = {
  id: string;
  kind: BattleInfoLogKind;
  summary: string;
  detail: string;
  revision?: number;
  order: number;
};

const defaultMatch: MatchState = {
  status: "active"
};

const defaultTurn: TurnState = {
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
};

const defaultScore: ScoreState = {
  byPlayer: {
    P1: 0,
    P2: 0
  },
  victoryThreshold: 2
};

const defaultRulesMetadata: RulesMetadata = {
  actionPolicies: [],
  payment: {
    mode: "prototype"
  },
  loyalty: {
    colorAliases: []
  },
  projection: {
    hiddenCardPreserves: []
  }
};

export function deriveBattleState(
  messages: DebuggerProtocolEnvelope[],
  localPlayerId: BattlePlayerId
): BattleState {
  const opponentPlayerId: BattlePlayerId = localPlayerId === "P1" ? "P2" : "P1";

  const localPatch = selectCurrentPatch(messages, localPlayerId);
  const opponentPatch = selectCurrentPatch(messages, opponentPlayerId);
  const spectatorPatch = selectCurrentPatch(messages, "spectator");
  const referencePatch = localPatch ?? spectatorPatch;

  const cards = sortedCards(selectCurrentCards(referencePatch));
  const match = referencePatch?.payload.playerView?.match ?? referencePatch?.payload.spectatorView?.match ?? defaultMatch;
  const turn = referencePatch?.payload.playerView?.turn ?? referencePatch?.payload.spectatorView?.turn ?? defaultTurn;
  const score = referencePatch?.payload.playerView?.score ?? referencePatch?.payload.spectatorView?.score ?? defaultScore;
  const rulesMetadata =
    referencePatch?.payload.playerView?.rulesMetadata ??
    referencePatch?.payload.spectatorView?.rulesMetadata ??
    defaultRulesMetadata;

  const localCards = cardsByOwner(cards, localPlayerId);
  const opponentCards = cardsByOwner(cards, opponentPlayerId);

  const localArea: BattlePlayerArea = {
    hand: cardsInZone(localCards, "hand"),
    deck: cardsInZone(localCards, "deck"),
    assets: cardsInZone(localCards, "asset"),
    discard: cardsInZone(localCards, "discard"),
    score: cardsInZone(localCards, "score"),
    table: cardsInZone(localCards, "table"),
    markers: cloneMarkers(localPatch?.payload.playerView?.markers)
  };

  const opponentHand = cardsInZone(opponentCards, "hand");
  const opponentAssets = cardsInZone(opponentCards, "asset");
  const opponentArea: BattleOpponentArea = {
    handCount: opponentHand.length,
    handPreview: opponentHand,
    deckCount: cardsInZone(opponentCards, "deck").length,
    assetCount: opponentAssets.length,
    assets: opponentAssets,
    discardCount: cardsInZone(opponentCards, "discard").length,
    scoreCount: cardsInZone(opponentCards, "score").length,
    table: cardsInZone(opponentCards, "table"),
    markers: cloneMarkers(opponentPatch?.payload.playerView?.markers)
  };

  const contest = deriveContestState(cards, localPlayerId, opponentPlayerId);

  return {
    localPlayerId,
    opponentPlayerId,
    match,
    turn,
    score,
    rulesMetadata,
    local: localArea,
    opponent: opponentArea,
    contest,
    actionCandidates: {
      localTableCardIds: idsWithIdentity(localArea.table),
      opponentTableCardIds: idsWithIdentity(opponentArea.table),
      regionCardIds: contest.regions.map((slot) => slot.regionCard?.cardId).filter(isNonEmptyString),
      localHandCardIds: idsWithIdentity(localArea.hand),
      visibleCardIds: idsWithIdentity(cards.filter((card) => card.visibility === "visible"))
    }
  };
}

export function deriveBattleInfoLogs(messages: DebuggerProtocolEnvelope[]): BattleInfoLogEntry[] {
  const entries: BattleInfoLogEntry[] = [];

  for (let index = 0; index < messages.length; index++) {
    const message = messages[index];

    if (message.name === "ActionAccepted") {
      entries.push(acceptedLog(message, index));
      continue;
    }

    if (message.name === "ActionRejected") {
      entries.push(rejectedLog(message, index));
      continue;
    }

    if (message.name === "StatePatched") {
      const view = message.payload.playerView ?? message.payload.spectatorView;
      if (!view) {
        continue;
      }

      const revision = message.payload.revision.number;
      const scoreSummary = Object.keys(view.score.byPlayer)
        .sort((left, right) => left.localeCompare(right))
        .map((playerID) => `${playerID}:${view.score.byPlayer[playerID] ?? 0}`)
        .join(" ");

      const winner = view.match.winnerPlayerId ? ` winner=${view.match.winnerPlayerId}` : "";
      entries.push({
        id: `${message.messageId}:system`,
        kind: "system",
        summary: `系统 rev ${revision} ${view.turn.phase.name}/${view.turn.phase.step}`,
        detail: `优先权=${view.turn.priority.currentPlayerId} 分数=${scoreSummary} 状态=${view.match.status}${winner}`,
        revision,
        order: messageOrder(message.messageId, index)
      });
    }
  }

  entries.sort((left, right) => right.order - left.order);
  return entries;
}

function deriveContestState(
  cards: CardView[],
  localPlayerId: BattlePlayerId,
  opponentPlayerId: BattlePlayerId
): BattleContestState {
  const tableCards = cardsInZone(cards, "table");
  const regionCards = tableCards.filter((card) => card.kind === "region").sort(compareByRegionOrder);

  const regions: BattleRegionSlot[] = regionCards.map((regionCard, index) => ({
    slotId: regionCard.cardId ?? `region-slot-${index + 1}`,
    order: regionCard.regionOrder ?? index + 1,
    regionCard,
    localCards: [],
    opponentCards: []
  }));

  // Keep a stable 3-slot board even when backend has fewer exposed region cards.
  while (regions.length < 3) {
    regions.push({
      slotId: `region-slot-${regions.length + 1}`,
      order: regions.length + 1,
      localCards: [],
      opponentCards: []
    });
  }

  const slotsByRegionCardId = new Map<string, BattleRegionSlot>();
  for (const slot of regions) {
    if (slot.regionCard?.cardId) {
      slotsByRegionCardId.set(slot.regionCard.cardId, slot);
    }
  }

  const unassignedTableCards: CardView[] = [];
  for (const card of tableCards) {
    if (card.kind === "region") {
      continue;
    }

    const targetSlot = card.regionCardId ? slotsByRegionCardId.get(card.regionCardId) : undefined;
    if (!targetSlot) {
      unassignedTableCards.push(card);
      continue;
    }

    if (card.ownerId === localPlayerId) {
      targetSlot.localCards.push(card);
      continue;
    }
    if (card.ownerId === opponentPlayerId) {
      targetSlot.opponentCards.push(card);
      continue;
    }

    unassignedTableCards.push(card);
  }

  for (const slot of regions) {
    slot.localCards = sortedCards(slot.localCards);
    slot.opponentCards = sortedCards(slot.opponentCards);
  }

  return {
    regions: regions.sort((left, right) => left.order - right.order),
    unassignedTableCards: sortedCards(unassignedTableCards)
  };
}

function compareByRegionOrder(left: CardView, right: CardView) {
  const leftOrder = left.regionOrder ?? Number.MAX_SAFE_INTEGER;
  const rightOrder = right.regionOrder ?? Number.MAX_SAFE_INTEGER;
  if (leftOrder !== rightOrder) {
    return leftOrder - rightOrder;
  }

  return cardStableKey(left).localeCompare(cardStableKey(right));
}

function cardsByOwner(cards: CardView[], ownerId: string) {
  return cards.filter((card) => card.ownerId === ownerId);
}

function cardsInZone(cards: CardView[], zone: CardView["zone"]) {
  return cards.filter((card) => card.zone === zone);
}

function idsWithIdentity(cards: CardView[]) {
  return cards.map((card) => card.cardId).filter(isNonEmptyString);
}

function isNonEmptyString(value: string | undefined): value is string {
  return typeof value === "string" && value.trim() !== "";
}

function cloneMarkers(markers: Record<string, number> | undefined) {
  const result: Record<string, number> = {};
  for (const [markerType, amount] of Object.entries(markers ?? {})) {
    result[markerType] = amount;
  }
  return result;
}

function sortedCards(cards: CardView[]) {
  return [...cards].sort((left, right) => cardStableKey(left).localeCompare(cardStableKey(right)));
}

function cardStableKey(card: CardView) {
  return [
    card.zone,
    card.ownerId,
    card.regionOrder ?? 0,
    card.regionCardId ?? "",
    card.cardId ?? "",
    card.name ?? ""
  ].join("|");
}

function acceptedLog(message: ActionAcceptedEnvelope, fallbackOrder: number): BattleInfoLogEntry {
  const revision = message.payload.revision.number;
  return {
    id: `${message.messageId}:accepted`,
    kind: "accepted",
    summary: `已接受 ${message.payload.action.actorId} -> ${message.payload.action.kind}`,
    detail: `事件=${message.payload.event.kind} 阶段=${message.payload.event.phase ?? "-"} 优先权=${message.payload.event.priorityPlayerId ?? "-"} 栈深=${message.payload.event.stackDepth}`,
    revision,
    order: messageOrder(message.messageId, fallbackOrder)
  };
}

function rejectedLog(message: ActionRejectedEnvelope, fallbackOrder: number): BattleInfoLogEntry {
  return {
    id: `${message.messageId}:rejected`,
    kind: "rejected",
    summary: `已拒绝 ${message.payload.action.actorId} -> ${message.payload.action.kind}`,
    detail: `原因=${message.payload.legality.reasonCode ?? "-"} 提示=${message.payload.legality.messageKey ?? "-"}`,
    order: messageOrder(message.messageId, fallbackOrder)
  };
}

function messageOrder(messageId: string, fallback: number): number {
  const match = /(\d+)$/.exec(messageId);
  if (!match) {
    return fallback + 1;
  }

  const parsed = Number(match[1]);
  if (!Number.isFinite(parsed)) {
    return fallback + 1;
  }

  return parsed;
}
