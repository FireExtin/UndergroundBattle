// Purpose: Defines the shared TypeScript-side contract model for CardLogic DSL fixtures.

export const SPEED_VALUES = ["slow", "fast", "reaction"] as const;
export const TARGET_KIND_VALUES = ["character", "player", "region", "secretSociety", "asset", "attachment"] as const;
export const DURATION_KIND_VALUES = ["none", "turn", "permanent"] as const;
export const EFFECT_KIND_VALUES = [
  "addKeyword",
  "dealDamage",
  "drawCards",
  "modifyStat",
  "placeInfluence",
  "inspectHand",
  "exhaust"
] as const;

export type Speed = (typeof SPEED_VALUES)[number];
export type TargetKind = (typeof TARGET_KIND_VALUES)[number];
export type DurationKind = (typeof DURATION_KIND_VALUES)[number];
export type EffectKind = (typeof EFFECT_KIND_VALUES)[number];

export type DrawCardsEffect = {
  kind: "drawCards";
  targetRef: "controller";
  amount: number;
};

export type PlaceInfluenceEffect = {
  kind: "placeInfluence";
  targetRef: "selected" | "controller";
  amount: number;
};

export type DealDamageEffect = {
  kind: "dealDamage";
  targetRef: "selected";
  amount: number;
};

export type ModifyStatEffect = {
  kind: "modifyStat";
  targetRef: "selected";
  stat: "combat" | "defense" | "influence" | "investigation";
  amount: number;
};

export type AddKeywordEffect = {
  kind: "addKeyword";
  targetRef: "selected";
  keyword: string;
};

export type InspectHandEffect = {
  kind: "inspectHand";
  targetRef: "selected";
};

export type ExhaustEffect = {
  kind: "exhaust";
  targetRef: "selected";
};

export type BasicEffect =
  | DrawCardsEffect
  | PlaceInfluenceEffect
  | DealDamageEffect
  | ModifyStatEffect
  | AddKeywordEffect
  | InspectHandEffect
  | ExhaustEffect;

export type CardLogic = {
  id: string;
  schemaVersion: string;
  speed: Speed;
  targetKinds: TargetKind[];
  requiresStack: boolean;
  durationKind: DurationKind;
  scriptId: string | null;
  effects: BasicEffect[];
};

export type FixtureExpectations = {
  parseOk: boolean;
  speed: Speed;
  targetKinds: TargetKind[];
  requiresStack: boolean;
  durationKind: DurationKind;
  scriptId: string | null;
};

export type FixtureCard = {
  name: string;
  sourcePath: string;
};

export type ContractFixture = {
  $schema?: string;
  cardId: string;
  schemaVersion: string;
  card: FixtureCard;
  input: {
    logic: CardLogic;
  };
  expectations: FixtureExpectations;
};

export type NormalizedContractRecord = {
  cardId: string;
  cardName: string;
  sourcePath: string;
  schemaVersion: string;
  logicId: string;
  speed: Speed;
  targetKinds: TargetKind[];
  requiresStack: boolean;
  durationKind: DurationKind;
  scriptId: string | null;
  requiresScript: boolean;
  pureDSLExecutable: boolean;
  effectKinds: EffectKind[];
};

export type NormalizedContractEnvelope = {
  schemaVersion: string;
  generatedAt: string;
  recordType: "CardLogicContract";
  records: NormalizedContractRecord[];
};

export type ValidationIssue = {
  code: string;
  message: string;
  path: string;
};
