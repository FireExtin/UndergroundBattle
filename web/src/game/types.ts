export type Icons = { investigation: number; combat: number; influence: number };
export type CardDefinition = {
  subtitle?: string;
  id: string; name: string; kind: string; type?: string; cost: number;
  loyalty?: string[]; loyaltyText?: string; subtypes?: string[]; color?: string; magic?: string; text: string;
  icons?: { permanent: Icons; temporary: Icons };
  permanentIcons?: Icons; temporaryIcons?: Icons; defense?: number;
  threshold?: number; points?: number; keywords?: string[]; supported?: boolean;
  deckCopyLimit?: number | null;
  abilities?: { key: string; label: string; timing: 'standard' | 'fast' | 'actionFast'; triggered: boolean }[];
};
export type SocietyDefinition = CardDefinition & {
  kind: 'society';
  subtitle?: string;
  printedCost?: null;
  startingHand: number;
  deckConstraints: { kind: 'minimumColor'; color: string; count: number }[];
  unresolvedAbilities: Record<string, string>;
};
export type Card = {
  currentRenown?: boolean;
  currentBarrier?: boolean;
  currentSubtypes?: string[];
  instanceId: string; cardId?: string; name: string; owner: string; controller: string;
  kind: string; region?: number; exhausted: boolean; faceDown: boolean;
  cost?: number; effectiveCost?: number; text?: string; icons?: Icons; defense?: number; damage?: number;
  shield?: number; wounds?: number; color?: string; magic?: string;
  usedOncePerGame?: string[];
};
export type Attachment = Card & { hostId: string };
export type Action = {
  kind: string; cardId?: string; targetId?: string; region?: number; option?: string;
  abilityId?: string; costSelected?: string[];
  choiceId?: string; selected?: string[]; top?: string[]; bottom?: string[];
  allocations?: Record<string, number>;
  deckDraft?: import('./deckLibrary').DeckDraft;
  windowId?: string; intentId?: string; action?: Action;
};
export type LegalAction = Action & { id: string; label: string; description?: string; sourceZoneId?: string };
export type Choice = {
  id: string; kind: string; title: string; description: string; playerId: string;
  options: { id: string; label: string; card?: Card }[];
  min?: number; max?: number; amount?: number; allowDecline?: boolean;
};
export type Player = {
  id: string; seat: number; name: string; team: number; deckId: string;
  deckName?: string;
  ready: boolean; eliminated: boolean; handCount: number; deckCount: number; score: number;
};
export type Region = {
  id: string; index: number; cardId: string; name: string; threshold: number;
  points: number; influence: number[]; characters: Card[];
  iconsByTeam?: [Icons, Icons];
  skipConfrontation?: boolean;
};
export type StackEffect = {
  id: string; label: string; controller: string; cardId?: string; targetId?: string;
  abilityId?: string; targets?: string[];
  resolutionState?: 'awaitingResponses' | 'resolving';
  targetSummaries?: {
    instanceId: string; label: string; kind: string; owner?: string; controller?: string; region?: number;
    valid: boolean; status: 'valid' | 'missing' | 'changed' | 'guardAccepted'; invalidReason?: string;
  }[];
};
export type ResponseIntentWindow = {
  id: string; stackTopId: string; holderTeam: number;
  members: { playerId: string; status: 'undecided' | 'composing' | 'passed'; deadlineMs?: number }[];
  canBegin: boolean; myIntentId?: string;
};
export type View = {
  roomId: string; inviteCode: string; version: number; mode: 'duel' | 'teams';
  status: 'lobby' | 'playing' | 'finished'; you: string; players: Player[];
  firstTeam: number; activeTeam: number; priorityTeam: number; turn: number;
  phase: string; step: string; winScore: number; winnerTeam?: number;
  regions: Region[]; hand: Card[]; assets: Card[]; graveyard: Card[]; scoreCards: Card[];
  privateDeckTop?: Card;
  attachments?: Attachment[];
  societyZones?: { id: string; playerId: string; card: Card | null }[];
  stack: StackEffect[];
  pendingChoice: Choice | null;
  waitingChoice?: { playerId: string; title: string; kind: string } | null;
  legalActions: LegalAction[]; log: { version: number; text: string }[];
  versions: { rules: string; cardPool: string; engine: string };
  worldDeckCount?: number;
  yourDeck?: import('./deckLibrary').DeckDraft | null;
  serverNowMs?: number;
  responseWindow?: ResponseIntentWindow | null;
};
export type Deck = { id: string; name: string; description: string; cardCount: number; cards: { cardId: string; count: number }[] };
export type Catalog = { rulesVersion: string; cardPoolVersion: string; engineVersion: string; decks: Deck[]; cards: CardDefinition[];
  societies?: SocietyDefinition[];
  deckBuildRules?: { minimumCards: number; serviceCardCapacity: number; societySupported: boolean } };
export type Session = { roomId: string; inviteCode: string; token: string; seat: number; view: View };
export type SavedSession = Omit<Session, 'view'>;
