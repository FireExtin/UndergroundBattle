import type { CardCounters, CardNumericStats, MockMessageSet } from "./protocol";

// Purpose: Provides one deterministic mock match transcript that obeys the shared envelope schema and Go payload field names.

const zeroStats: CardNumericStats = {
  combat: 0,
  defense: 0,
  influence: 0,
  investigation: 0
};

const zeroCounters: CardCounters = {
  damage: 0,
  influence: 0
};

const revision = {
  number: 7,
  actionId: "act-queue-1",
  operationId: "op:act-queue-1",
  eventId: "evt:act-queue-1"
} as const;

const turn = {
  turnNumber: 3,
  activePlayerId: "P1",
  priorityPlayerId: "P2",
  priority: {
    currentPlayerId: "P2",
    passCount: 1,
    lastPassedPlayerId: "P1",
    windowKind: "response"
  },
  phase: {
    name: "main",
    step: "action",
    allowsStack: true,
    stepEnded: false
  }
} as const;

const sharedStack = [
  {
    id: "op:opening",
    actionId: "act-opening",
    actorId: "P1",
    kind: "card_effect",
    status: "pending",
    requiresStack: true,
    cardId: "BQ100",
    label: "Opening Move"
  },
  {
    id: "op:dream-snare",
    actionId: "act-dream-snare",
    actorId: "P1",
    kind: "card_effect",
    status: "pending",
    requiresStack: true,
    cardId: "BQ101",
    label: "Dream Snare"
  },
  {
    id: "op:quick-barrier",
    actionId: "act-quick-barrier",
    actorId: "P2",
    kind: "card_effect",
    status: "pending",
    requiresStack: true,
    cardId: "BQ102",
    label: "Quick Barrier"
  }
] as const;

const queueEvent = {
  id: "evt:act-queue-1",
  actionId: "act-queue-1",
  operationId: "op:act-queue-1",
  kind: "operation_enqueued",
  revisionNumber: 7,
  phase: "main",
  step: "action",
  priorityPlayerId: "P2",
  priorityWindow: "response",
  passCount: 1,
  stackDepth: 3
} as const;

export const defaultMockMessageSets: MockMessageSet[] = [
  {
    id: "default-match",
    label: "Mock Match 01",
    messages: [
      {
        version: "0.1.0",
        kind: "event",
        messageId: "msg-action-accepted-1",
        name: "ActionAccepted",
        revision: 7,
        payload: {
          type: "ActionAccepted",
          action: {
            id: "act-queue-1",
            actorId: "P1",
            kind: "queue_operation",
            cardId: "BQ022",
            targetCardId: "P2-TABLE-1"
          },
          operation: {
            id: "op:act-queue-1",
            actionId: "act-queue-1",
            actorId: "P1",
            kind: "card_effect",
            status: "pending",
            requiresStack: true,
            cardId: "BQ022",
            label: "Dream Snare"
          },
          event: queueEvent,
          revision
        }
      },
      {
        version: "0.1.0",
        kind: "event",
        messageId: "msg-action-rejected-1",
        name: "ActionRejected",
        payload: {
          type: "ActionRejected",
          action: {
            id: "act-illegal-1",
            actorId: "P1",
            kind: "queue_operation",
            cardId: "BQ010"
          },
          legality: {
            ok: false,
            reasonCode: "LEGALITY_FAILED_NOT_YOUR_PRIORITY",
            messageKey: "rules.legality.not_your_priority",
            hook: "turn.priority",
            context: {
              actorId: "P1",
              priorityPlayerId: "P2"
            }
          }
        }
      },
      {
        version: "0.1.0",
        kind: "view",
        messageId: "msg-state-patched-p1",
        name: "StatePatched",
        revision: 7,
        payload: {
          type: "StatePatched",
          audienceKind: "player",
          audienceId: "P1",
          revision,
          event: queueEvent,
          playerView: {
            gameId: "game-web-debugger",
            viewerPlayerId: "P1",
            revision,
            turn,
            board: {
              stack: [...sharedStack],
              resolved: [],
              randomResults: [],
              cards: [
                {
                  cardId: "P1-HAND-SECRET",
                  name: "Secret Archive",
                  ownerId: "P1",
                  zone: "hand",
                  visibility: "visible",
                  revealed: false,
                  exhausted: false,
                  destroyed: false,
                  keywords: [],
                  stats: zeroStats,
                  counters: zeroCounters
                },
                {
                  cardId: "P2-TABLE-1",
                  name: "Frontline Adept",
                  ownerId: "P2",
                  zone: "table",
                  visibility: "visible",
                  revealed: true,
                  exhausted: false,
                  destroyed: false,
                  keywords: ["blackBlade"],
                  stats: {
                    combat: 2,
                    defense: 3,
                    influence: 0,
                    investigation: 1
                  },
                  counters: {
                    damage: 1,
                    influence: 0
                  }
                }
              ]
            }
          }
        }
      },
      {
        version: "0.1.0",
        kind: "view",
        messageId: "msg-state-patched-p2",
        name: "StatePatched",
        revision: 7,
        payload: {
          type: "StatePatched",
          audienceKind: "player",
          audienceId: "P2",
          revision,
          event: queueEvent,
          playerView: {
            gameId: "game-web-debugger",
            viewerPlayerId: "P2",
            revision,
            turn,
            board: {
              stack: [...sharedStack],
              resolved: [],
              randomResults: [],
              cards: [
                {
                  ownerId: "P1",
                  zone: "hand",
                  visibility: "hidden",
                  revealed: false,
                  exhausted: false,
                  destroyed: false,
                  stats: zeroStats,
                  counters: zeroCounters
                },
                {
                  cardId: "P2-TABLE-1",
                  name: "Frontline Adept",
                  ownerId: "P2",
                  zone: "table",
                  visibility: "visible",
                  revealed: true,
                  exhausted: false,
                  destroyed: false,
                  keywords: ["blackBlade"],
                  stats: {
                    combat: 2,
                    defense: 3,
                    influence: 0,
                    investigation: 1
                  },
                  counters: {
                    damage: 1,
                    influence: 0
                  }
                }
              ]
            }
          }
        }
      },
      {
        version: "0.1.0",
        kind: "view",
        messageId: "msg-state-patched-spectator",
        name: "StatePatched",
        revision: 7,
        payload: {
          type: "StatePatched",
          audienceKind: "spectator",
          revision,
          event: queueEvent,
          spectatorView: {
            gameId: "game-web-debugger",
            revision,
            turn,
            board: {
              stack: [...sharedStack],
              resolved: [],
              randomResults: [],
              cards: [
                {
                  ownerId: "P1",
                  zone: "hand",
                  visibility: "hidden",
                  revealed: false,
                  exhausted: false,
                  destroyed: false,
                  stats: zeroStats,
                  counters: zeroCounters
                },
                {
                  cardId: "P2-TABLE-1",
                  name: "Frontline Adept",
                  ownerId: "P2",
                  zone: "table",
                  visibility: "visible",
                  revealed: true,
                  exhausted: false,
                  destroyed: false,
                  keywords: ["blackBlade"],
                  stats: {
                    combat: 2,
                    defense: 3,
                    influence: 0,
                    investigation: 1
                  },
                  counters: {
                    damage: 1,
                    influence: 0
                  }
                }
              ]
            }
          }
        }
      }
    ]
  }
];
