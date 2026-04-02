import { selectCurrentCards, selectCurrentPatch } from "../debugger/model";
import type {
  CardView,
  DebuggerProtocolEnvelope,
  MatchState,
  ScoreState,
  TurnState,
  ViewerId
} from "../debugger/protocol";

// Purpose: Derives a board-game oriented battle-table view model from protocol envelopes without duplicating server rules.

export type BattlePlayerId = Exclude<ViewerId, "spectator">;

export type BattlePlayerArea = {
  hand: CardView[];
  deck: CardView[];
  discard: CardView[];
  score: CardView[];
  table: CardView[];
  markers: Record<string, number>;
};

export type BattleOpponentArea = {
  handCount: number;
  handPreview: CardView[];
  deckCount: number;
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
  local: BattlePlayerArea;
  opponent: BattleOpponentArea;
  contest: BattleContestState;
  actionCandidates: BattleActionCandidates;
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

  const localCards = cardsByOwner(cards, localPlayerId);
  const opponentCards = cardsByOwner(cards, opponentPlayerId);

  const localArea: BattlePlayerArea = {
    hand: cardsInZone(localCards, "hand"),
    deck: cardsInZone(localCards, "deck"),
    discard: cardsInZone(localCards, "discard"),
    score: cardsInZone(localCards, "score"),
    table: cardsInZone(localCards, "table"),
    markers: cloneMarkers(localPatch?.payload.playerView?.markers)
  };

  const opponentHand = cardsInZone(opponentCards, "hand");
  const opponentArea: BattleOpponentArea = {
    handCount: opponentHand.length,
    handPreview: opponentHand,
    deckCount: cardsInZone(opponentCards, "deck").length,
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
