// Purpose: Centralizes the normalized data contracts emitted by the card importer.

export const CARD_CATEGORY_VALUES = [
  "attachment",
  "character",
  "character_attachment",
  "core_zone",
  "encounter_character",
  "event",
  "mission_region",
  "region",
  "secret_society",
  "token",
  "unknown"
] as const;

export type CardCategory = (typeof CARD_CATEGORY_VALUES)[number];

export type ValueField = {
  raw: string;
  value: number | null;
};

export type LoyaltyField = {
  raw: string;
  symbols: string[];
};

export type CardPrint = {
  id: string;
  schemaVersion: string;
  name: string;
  category: CardCategory;
  basicType: string;
  typeLine: string;
  traits: string[];
  sourcePath: string;
  sourceMarkdownPath: string | null;
  artworkPath: string | null;
  sourceSet: string | null;
  sourceSetNumber: string | null;
  rawText: string;
  text: string[];
  keywords: string[];
  relatedIds: string[];
  cost: ValueField;
  defense: ValueField;
  loyalty: LoyaltyField;
  abilityIcons: string[];
  color: string | null;
  magicDomain: string | null;
  society: string | null;
  searchText: string | null;
  flags: {
    isToken: boolean;
    isDeckCard: boolean;
  };
};

export type RuleDocMeta = {
  id: string;
  schemaVersion: string;
  title: string;
  description: string;
  sourcePath: string;
  sourcePdfPath: string | null;
  rawText: string;
  headings: string[];
  lineCount: number;
  characterCount: number;
  contentHash: string;
};

export type TokenMeta = {
  id: string;
  schemaVersion: string;
  name: string;
  category: CardCategory;
  basicType: string;
  typeLine: string;
  traits: string[];
  sourcePath: string;
  sourceMarkdownPath: string | null;
  rawText: string;
  text: string[];
  keywords: string[];
  cost: ValueField;
  defense: ValueField;
  abilityIcons: string[];
  color: string | null;
  magicDomain: string | null;
  society: string | null;
  linkedCardPrintId: string | null;
};

export type CardRawBucket = {
  id: string;
  schemaVersion: string;
  category: CardCategory;
  sourcePath: string;
  sourceMarkdownPath: string | null;
  recordCount: number;
  ids: string[];
};

export type DatasetEnvelope<T> = {
  schemaVersion: string;
  generatedAt: string;
  recordType: string;
  records: T[];
};

export type ImporterSchemaVersions = {
  cardsRawIndex?: string;
  cardPrint?: string;
  ruleDocMeta?: string;
  tokenMeta?: string;
};

export type ImportOutputs = {
  cardsRawIndex: DatasetEnvelope<CardRawBucket>;
  cardsNormalized: DatasetEnvelope<CardPrint>;
  rulesIndex: DatasetEnvelope<RuleDocMeta>;
  tokensIndex: DatasetEnvelope<TokenMeta>;
};

export type ImportOptions = {
  repoRoot: string;
  outputRoot?: string;
  schemaConfigPath?: string;
  generatedAt?: string;
  writeFiles?: boolean;
};

