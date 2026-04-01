// Purpose: Mirrors the minimal Go-side protocol payloads used by the web debugger without inventing a new DTO layer.

export type ProtocolChannelKind = "command" | "event" | "view";
export type ViewerId = "P1" | "P2" | "spectator";
export type CardZone = "deck" | "hand" | "table" | "discard";

export type Action = {
  id: string;
  actorId: string;
  kind: string;
  cardId?: string;
  targetPlayerId?: string;
  targetCardId?: string;
  operationLabel?: string;
  randomMax?: number;
  markerType?: string;
  markerAmount?: number;
};

export type CardOperationSource = {
  cardId: string;
  cardName: string;
  sourcePath: string;
  logicId: string;
  speed: string;
  targetKinds: string[];
  requiresStack: boolean;
  executionKind: "dsl" | "script";
  durationKind: string;
  scriptId?: string | null;
  requiresScript: boolean;
  pureDSLExecutable: boolean;
  effectKinds: string[];
};

export type Operation = {
  id: string;
  actionId: string;
  actorId: string;
  kind: string;
  status: string;
  requiresStack: boolean;
  cardId?: string;
  targetPlayerId?: string;
  targetCardId?: string;
  label?: string;
  randomMax?: number;
  nextPhase?: string;
  markerType?: string;
  markerAmount?: number;
  source?: CardOperationSource;
};

export type Event = {
  id: string;
  actionId: string;
  operationId: string;
  kind: string;
  revisionNumber: number;
  phase?: string;
  step?: string;
  priorityPlayerId?: string;
  priorityWindow?: string;
  passCount?: number;
  resolvedTargetId?: string;
  stackDepth: number;
  randomValue?: number;
  stepEnded?: boolean;
  markerType?: string;
  markerAmount?: number;
  targetPlayerId?: string;
  sourceCardId?: string;
  targetCardId?: string;
  appliedAmount?: number;
};

export type Revision = {
  number: number;
  actionId?: string;
  operationId?: string;
  eventId?: string;
};

export type PriorityState = {
  currentPlayerId: string;
  passCount: number;
  lastPassedPlayerId?: string;
  windowKind: string;
};

export type PhaseState = {
  name: string;
  step: string;
  allowsStack: boolean;
  stepEnded: boolean;
};

export type TurnState = {
  turnNumber: number;
  activePlayerId: string;
  priorityPlayerId: string;
  firstPlayerPrivilegeUsed?: boolean;
  priority: PriorityState;
  phase: PhaseState;
};

export type ScoreState = {
  byPlayer: Record<string, number>;
  victoryThreshold: number;
  winnerPlayerId?: string;
};

export type MatchState = {
  status: "active" | "finished";
  endReason?: string;
  winnerPlayerId?: string;
  finishedAtRevision?: number;
};

export type CardNumericStats = {
  combat: number;
  defense: number;
  influence: number;
  investigation: number;
};

export type CardCounters = {
  damage: number;
  influence: number;
  shield?: number;
};

export type CardView = {
  cardId?: string;
  name?: string;
  ownerId: string;
  zone: CardZone;
  visibility: string;
  revealed: boolean;
  faceDown?: boolean;
  exhausted: boolean;
  destroyed: boolean;
  keywords?: string[];
  stats: CardNumericStats;
  counters: CardCounters;
};

export type ViewBoardState = {
  stack: Operation[];
  resolved: Operation[];
  randomResults: Array<{
    actionId: string;
    operationId: string;
    drawIndex: number;
    value: number;
  }>;
  cards: CardView[];
};

export type PlayerViewState = {
  gameId: string;
  viewerPlayerId: string;
  revision: Revision;
  match: MatchState;
  turn: TurnState;
  score: ScoreState;
  markers?: Record<string, number>;
  board: ViewBoardState;
};

export type SpectatorViewState = {
  gameId: string;
  revision: Revision;
  match: MatchState;
  turn: TurnState;
  score: ScoreState;
  markers?: Record<string, number>;
  board: ViewBoardState;
};

export type LegalityResult = {
  ok: boolean;
  reasonCode?: string;
  messageKey?: string;
  hook?: string;
  context?: Record<string, string>;
};

export type ActionAccepted = {
  type: "ActionAccepted";
  action: Action;
  operation: Operation;
  event: Event;
  revision: Revision;
};

export type ActionRejected = {
  type: "ActionRejected";
  action: Action;
  legality: LegalityResult;
};

export type StatePatched = {
  type: "StatePatched";
  audienceKind: "player" | "spectator";
  audienceId?: string;
  revision: Revision;
  event: Event;
  playerView?: PlayerViewState;
  spectatorView?: SpectatorViewState;
};

export type ProtocolEnvelope<
  Name extends string,
  Kind extends ProtocolChannelKind,
  Payload
> = {
  version: string;
  kind: Kind;
  messageId: string;
  name: Name;
  revision?: number;
  payload: Payload;
};

export type ActionAcceptedEnvelope = ProtocolEnvelope<"ActionAccepted", "event", ActionAccepted>;
export type ActionRejectedEnvelope = ProtocolEnvelope<"ActionRejected", "event", ActionRejected>;
export type StatePatchedEnvelope = ProtocolEnvelope<"StatePatched", "view", StatePatched>;

export type DebuggerProtocolEnvelope =
  | ActionAcceptedEnvelope
  | ActionRejectedEnvelope
  | StatePatchedEnvelope;

export type MockMessageSet = {
  id: string;
  label: string;
  messages: DebuggerProtocolEnvelope[];
};
