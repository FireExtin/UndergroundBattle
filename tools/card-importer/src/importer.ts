// Purpose: Scans organized resources, normalizes stable metadata, validates the result, and writes reusable JSON datasets.

import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { ERROR_CODES, ImporterError, type NormalizationError } from "./errors.js";
import type {
  CardCategory,
  CardPrint,
  CardRawBucket,
  DatasetEnvelope,
  ImportOptions,
  ImportOutputs,
  ImporterSchemaVersions,
  RuleDocMeta,
  TokenMeta,
  ValueField
} from "./types.js";
import { validateCardPrints, validateRuleDocMetas, validateTokenMetas } from "./validators.js";

type RawRecord = Record<string, unknown>;

type RuleDescription = {
  description: string;
  pdfFilename: string | null;
};

type ComparableEntry = {
  name: string;
};

const CARD_CATEGORY_BY_BASIC_TYPE: Record<string, CardCategory> = {
  "事务": "event",
  "任务/地区": "mission_region",
  "地区": "region",
  "指示物": "token",
  "指示物角色": "token",
  "指示物附属": "token",
  "核心区": "core_zone",
  "秘社": "secret_society",
  "角色": "character",
  "角色/附属": "character_attachment",
  "遭遇角色": "encounter_character",
  "附属": "attachment"
};

const RULE_DOC_DESCRIPTIONS: Record<string, RuleDescription> = {
  "隐秘世界勘误及释疑.md": {
    description: "收录已发布内容的勘误、释疑与裁定补充，作为规则手册之外的修订依据。",
    pdfFilename: "隐秘世界勘误及释疑.pdf"
  },
  "隐秘世界玩家指南.md": {
    description: "面向入门玩家的基础规则说明与教学材料，优先解释核心流程而非完整裁定细节。",
    pdfFilename: "隐秘世界玩家指南.pdf"
  },
  "隐秘世界规则手册.md": {
    description: "提供术语表、回合流程和细节裁定，是后续规则引擎对照的人类参考资料。",
    pdfFilename: "隐秘世界规则手册.pdf"
  },
  "霸权说明书.md": {
    description: "霸权相关卡池与扩展说明文档，用于补充主规则之外的内容边界。",
    pdfFilename: "霸权说明书.pdf"
  }
};

const LOYALTY_PATTERN = /(黄色|蓝色|绿色|红色|紫色|灰色|黑色|中立)/g;
const ABILITY_ICON_PATTERN = /［[^］]+］/g;

export async function importRepositoryData(options: ImportOptions): Promise<ImportOutputs> {
  const repoRoot = path.resolve(options.repoRoot);
  const outputRoot = path.resolve(options.outputRoot ?? path.join(repoRoot, "data/normalized"));
  const schemaConfigPath = path.resolve(
    options.schemaConfigPath ?? path.join(repoRoot, "tools/card-importer/config/schema-versions.json")
  );
  const generatedAt = options.generatedAt ?? new Date().toISOString();

  const schemaVersions = await loadSchemaVersions(schemaConfigPath);
  const issues: NormalizationError[] = [];

  const cardBuckets = await loadCardBuckets(repoRoot, schemaVersions, issues);
  const cardPrints = await loadCardPrints(repoRoot, schemaVersions, issues);
  const ruleDocs = await loadRuleDocs(repoRoot, schemaVersions, issues);
  const tokenMetas = await loadTokenMetas(repoRoot, schemaVersions, issues, new Set(cardPrints.map((card) => card.id)));

  issues.push(...validateCardPrints(cardPrints));
  issues.push(...validateRuleDocMetas(ruleDocs));
  issues.push(...validateTokenMetas(tokenMetas));

  if (issues.length > 0) {
    throw new ImporterError("Importer validation failed.", issues);
  }

  const outputs: ImportOutputs = {
    cardsRawIndex: createDatasetEnvelope<CardRawBucket>(
      requireSchemaVersion("cardsRawIndex", schemaVersions, schemaConfigPath),
      "CardRawBucket",
      cardBuckets,
      generatedAt
    ),
    cardsNormalized: createDatasetEnvelope<CardPrint>(
      requireSchemaVersion("cardPrint", schemaVersions, schemaConfigPath),
      "CardPrint",
      cardPrints,
      generatedAt
    ),
    rulesIndex: createDatasetEnvelope<RuleDocMeta>(
      requireSchemaVersion("ruleDocMeta", schemaVersions, schemaConfigPath),
      "RuleDocMeta",
      ruleDocs,
      generatedAt
    ),
    tokensIndex: createDatasetEnvelope<TokenMeta>(
      requireSchemaVersion("tokenMeta", schemaVersions, schemaConfigPath),
      "TokenMeta",
      tokenMetas,
      generatedAt
    )
  };

  if (options.writeFiles !== false) {
    await writeImportOutputs(outputRoot, outputs);
  }

  return outputs;
}

export async function writeImportOutputs(outputRoot: string, outputs: ImportOutputs): Promise<void> {
  await mkdir(outputRoot, { recursive: true });

  await Promise.all([
    writeJson(path.join(outputRoot, "cards.raw.index.json"), outputs.cardsRawIndex),
    writeJson(path.join(outputRoot, "cards.normalized.json"), outputs.cardsNormalized),
    writeJson(path.join(outputRoot, "rules.index.json"), outputs.rulesIndex),
    writeJson(path.join(outputRoot, "tokens.index.json"), outputs.tokensIndex)
  ]);
}

async function loadCardBuckets(
  repoRoot: string,
  schemaVersions: ImporterSchemaVersions,
  issues: NormalizationError[]
): Promise<CardRawBucket[]> {
  const schemaVersion = resolveSchemaVersion("cardsRawIndex", schemaVersions, issues, "CardRawBucket");
  const cardsRoot = path.join(repoRoot, "organized_content/cards");
  const entries = await readdir(cardsRoot, { withFileTypes: true });
  const buckets: CardRawBucket[] = [];

  for (const entry of sortNamedEntries(entries)) {
    if (!entry.isDirectory()) {
      continue;
    }

    const jsonPath = path.join(cardsRoot, entry.name, "cards.json");
    const markdownPath = path.join(cardsRoot, entry.name, "cards.md");
    const raw = await readJson(jsonPath);
    const ids = Object.values(raw).flatMap((value) => {
      const record = value as RawRecord;
      return typeof record.id === "string" ? [record.id] : [];
    });
    const firstRecord = Object.values(raw)[0] as RawRecord | undefined;
    const basicType = asOptionalString(firstRecord?.["basic-type"]) ?? "";

    buckets.push({
      id: `cards/${entry.name}`,
      schemaVersion,
      category: inferCardCategory(basicType),
      sourcePath: relativeToRepo(repoRoot, jsonPath),
      sourceMarkdownPath: await fileExists(markdownPath) ? relativeToRepo(repoRoot, markdownPath) : null,
      recordCount: ids.length,
      ids: sortStrings(ids)
    });
  }

  return buckets;
}

async function loadCardPrints(
  repoRoot: string,
  schemaVersions: ImporterSchemaVersions,
  issues: NormalizationError[]
): Promise<CardPrint[]> {
  const schemaVersion = resolveSchemaVersion("cardPrint", schemaVersions, issues, "CardPrint");
  const cardsRoot = path.join(repoRoot, "organized_content/cards");
  const artworkIndex = await loadArtworkIndex(repoRoot);
  const entries = await readdir(cardsRoot, { withFileTypes: true });
  const cards: CardPrint[] = [];

  for (const entry of sortNamedEntries(entries)) {
    if (!entry.isDirectory()) {
      continue;
    }

    const jsonPath = path.join(cardsRoot, entry.name, "cards.json");
    const markdownPath = path.join(cardsRoot, entry.name, "cards.md");
    const raw = await readJson(jsonPath);

    for (const [recordId, value] of sortObjectEntries(raw)) {
      const record = value as RawRecord;
      const id = asOptionalString(record.id) ?? recordId;
      const basicType = asOptionalString(record["basic-type"]) ?? "";

      cards.push({
        id,
        schemaVersion,
        name: asOptionalString(record.name) ?? "",
        category: inferCardCategory(basicType),
        basicType,
        typeLine: asOptionalString(record.type) ?? "",
        traits: parseTraits(asOptionalString(record.type) ?? ""),
        sourcePath: relativeToRepo(repoRoot, jsonPath),
        sourceMarkdownPath: await fileExists(markdownPath) ? relativeToRepo(repoRoot, markdownPath) : null,
        artworkPath: artworkIndex.get(id) ?? null,
        sourceSet: asOptionalString(record.set) ?? null,
        sourceSetNumber: asOptionalString(record["set-id"]) ?? null,
        rawText: asOptionalString(record.text) ?? "",
        text: splitTextLines(asOptionalString(record.text) ?? ""),
        keywords: parseStringArray(record.keywords),
        relatedIds: parseStringArray(record.related),
        cost: normalizeNumericField(asOptionalString(record.cost) ?? ""),
        defense: normalizeNumericField(asOptionalString(record.dfc) ?? ""),
        loyalty: {
          raw: asOptionalString(record.lyl) ?? "",
          symbols: parseLoyaltySymbols(asOptionalString(record.lyl) ?? "")
        },
        abilityIcons: parseAbilityIcons(asOptionalString(record.abl) ?? ""),
        color: emptyToNull(asOptionalString(record.color)),
        magicDomain: emptyToNull(asOptionalString(record.magic)),
        society: emptyToNull(asOptionalString(record.society)),
        searchText: emptyToNull(asOptionalString(record.search)),
        flags: {
          isToken: Boolean(record.istoken),
          isDeckCard: Boolean(record.deckcard)
        }
      });
    }
  }

  return cards;
}

async function loadRuleDocs(
  repoRoot: string,
  schemaVersions: ImporterSchemaVersions,
  issues: NormalizationError[]
): Promise<RuleDocMeta[]> {
  const schemaVersion = resolveSchemaVersion("ruleDocMeta", schemaVersions, issues, "RuleDocMeta");
  const rulesRoot = path.join(repoRoot, "organized_content/rules");
  const entries = await readdir(rulesRoot, { withFileTypes: true });
  const docs: RuleDocMeta[] = [];

  for (const entry of sortNamedEntries(entries)) {
    if (!entry.isFile() || path.extname(entry.name) !== ".md") {
      continue;
    }

    const markdownPath = path.join(rulesRoot, entry.name);
    const rawText = await readFile(markdownPath, "utf-8");
    const headings = extractMarkdownHeadings(rawText);
    const title = headings[0] ?? path.basename(entry.name, ".md");
    const descriptionMeta = RULE_DOC_DESCRIPTIONS[entry.name] ?? {
      description: buildFallbackRuleDescription(title),
      pdfFilename: null
    };
    const pdfPath = descriptionMeta.pdfFilename
      ? path.join(repoRoot, "resource/ymsj-fun.github.io/public/docs", descriptionMeta.pdfFilename)
      : null;

    docs.push({
      id: path.basename(entry.name, ".md"),
      schemaVersion,
      title,
      description: descriptionMeta.description,
      sourcePath: relativeToRepo(repoRoot, markdownPath),
      sourcePdfPath: (pdfPath && (await fileExists(pdfPath))) ? relativeToRepo(repoRoot, pdfPath) : null,
      rawText,
      headings,
      lineCount: rawText.split(/\r?\n/u).length,
      characterCount: rawText.length,
      contentHash: sha256(rawText)
    });
  }

  return docs;
}

async function loadTokenMetas(
  repoRoot: string,
  schemaVersions: ImporterSchemaVersions,
  issues: NormalizationError[],
  cardIds: Set<string>
): Promise<TokenMeta[]> {
  const schemaVersion = resolveSchemaVersion("tokenMeta", schemaVersions, issues, "TokenMeta");
  const sourcePath = path.join(repoRoot, "organized_content/tokens/tokens.json");
  const markdownPath = path.join(repoRoot, "organized_content/tokens/tokens.md");
  const raw = await readJson(sourcePath);
  const tokens: TokenMeta[] = [];

  for (const [recordId, value] of sortObjectEntries(raw)) {
    const record = value as RawRecord;
    const id = asOptionalString(record.id) ?? recordId;
    const basicType = asOptionalString(record["basic-type"]) ?? "";

    tokens.push({
      id,
      schemaVersion,
      name: asOptionalString(record.name) ?? "",
      category: inferCardCategory(basicType),
      basicType,
      typeLine: asOptionalString(record.type) ?? "",
      traits: parseTraits(asOptionalString(record.type) ?? ""),
      sourcePath: relativeToRepo(repoRoot, sourcePath),
      sourceMarkdownPath: await fileExists(markdownPath) ? relativeToRepo(repoRoot, markdownPath) : null,
      rawText: asOptionalString(record.text) ?? "",
      text: splitTextLines(asOptionalString(record.text) ?? ""),
      keywords: parseStringArray(record.keywords),
      cost: normalizeNumericField(asOptionalString(record.cost) ?? ""),
      defense: normalizeNumericField(asOptionalString(record.dfc) ?? ""),
      abilityIcons: parseAbilityIcons(asOptionalString(record.abl) ?? ""),
      color: emptyToNull(asOptionalString(record.color)),
      magicDomain: emptyToNull(asOptionalString(record.magic)),
      society: emptyToNull(asOptionalString(record.society)),
      linkedCardPrintId: cardIds.has(id) ? id : null
    });
  }

  return tokens;
}

async function loadArtworkIndex(repoRoot: string): Promise<Map<string, string>> {
  const cardsDir = path.join(repoRoot, "resource/ymsj-fun.github.io/cards");
  const index = new Map<string, string>();

  if (!(await fileExists(cardsDir))) {
    return index;
  }

  const entries = await readdir(cardsDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isFile()) {
      continue;
    }

    const match = entry.name.match(/^([A-Za-z0-9_-]+)/u);
    if (!match) {
      continue;
    }

    const cardID = match[1];
    if (typeof cardID !== "string" || cardID.length === 0) {
      continue;
    }

    index.set(cardID, relativeToRepo(repoRoot, path.join(cardsDir, entry.name)));
  }

  return index;
}

async function loadSchemaVersions(schemaConfigPath: string): Promise<ImporterSchemaVersions> {
  const raw = await readJson(schemaConfigPath);
  const schemaVersions: ImporterSchemaVersions = {};
  const cardsRawIndex = asOptionalString(raw.cardsRawIndex);
  const cardPrint = asOptionalString(raw.cardPrint);
  const ruleDocMeta = asOptionalString(raw.ruleDocMeta);
  const tokenMeta = asOptionalString(raw.tokenMeta);

  if (cardsRawIndex !== undefined) {
    schemaVersions.cardsRawIndex = cardsRawIndex;
  }

  if (cardPrint !== undefined) {
    schemaVersions.cardPrint = cardPrint;
  }

  if (ruleDocMeta !== undefined) {
    schemaVersions.ruleDocMeta = ruleDocMeta;
  }

  if (tokenMeta !== undefined) {
    schemaVersions.tokenMeta = tokenMeta;
  }

  return schemaVersions;
}

function resolveSchemaVersion(
  key: keyof ImporterSchemaVersions,
  schemaVersions: ImporterSchemaVersions,
  issues: NormalizationError[],
  recordType: string
): string {
  const value = schemaVersions[key];
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }

  issues.push({
    code: ERROR_CODES.SCHEMA_VERSION_MISSING,
    message: `${recordType} schemaVersion is not configured.`,
    recordType,
    field: "schemaVersion",
    details: {
      configKey: key
    }
  });

  return "";
}

function requireSchemaVersion(
  key: keyof ImporterSchemaVersions,
  schemaVersions: ImporterSchemaVersions,
  sourcePath: string
): string {
  const value = schemaVersions[key];
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }

  throw new ImporterError("Importer validation failed.", [
    {
      code: ERROR_CODES.SCHEMA_VERSION_MISSING,
      message: `Missing schemaVersion configuration for "${key}".`,
      recordType: "ImporterSchemaVersions",
      field: key,
      sourcePath
    }
  ]);
}

function createDatasetEnvelope<T>(
  schemaVersion: string,
  recordType: string,
  records: T[],
  generatedAt: string
): DatasetEnvelope<T> {
  return {
    schemaVersion,
    generatedAt,
    recordType,
    records
  };
}

function inferCardCategory(basicType: string): CardCategory {
  return CARD_CATEGORY_BY_BASIC_TYPE[basicType] ?? "unknown";
}

function parseTraits(typeLine: string): string[] {
  const [, traitText = ""] = typeLine.split("-", 2);
  return traitText
    .split("/")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function parseStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .map((item) => (typeof item === "string" ? item.trim() : ""))
    .filter((item) => item.length > 0);
}

function normalizeNumericField(raw: string): ValueField {
  const trimmed = raw.trim();
  return {
    raw: trimmed,
    value: /^\d+$/u.test(trimmed) ? Number(trimmed) : null
  };
}

function splitTextLines(text: string): string[] {
  return text
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function parseLoyaltySymbols(raw: string): string[] {
  const matches = raw.match(LOYALTY_PATTERN) ?? [];
  return matches;
}

function parseAbilityIcons(raw: string): string[] {
  return raw.match(ABILITY_ICON_PATTERN) ?? [];
}

function emptyToNull(value: string | undefined): string | null {
  if (typeof value !== "string") {
    return null;
  }

  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function extractMarkdownHeadings(markdown: string): string[] {
  return markdown
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line.startsWith("#"))
    .map((line) => line.replace(/^#+\s*/u, "").trim())
    .filter((line) => line.length > 0);
}

function buildFallbackRuleDescription(title: string): string {
  return `${title} 的原始规则文档元数据；若 Markdown 转写不可靠，应回看对应 PDF 进行人工校对。`;
}

function sha256(input: string): string {
  return createHash("sha256").update(input).digest("hex");
}

function asOptionalString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

async function readJson(filePath: string): Promise<Record<string, unknown>> {
  const content = await readFile(filePath, "utf-8");
  return JSON.parse(content) as Record<string, unknown>;
}

async function writeJson(filePath: string, value: unknown): Promise<void> {
  await writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf-8");
}

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await stat(filePath);
    return true;
  } catch {
    return false;
  }
}

function relativeToRepo(repoRoot: string, filePath: string): string {
  return path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
}

function sortNamedEntries<T extends ComparableEntry>(entries: readonly T[]): T[] {
  return [...entries].sort((left, right) => left.name.localeCompare(right.name, "zh-Hans-CN"));
}

function sortStrings(values: readonly string[]): string[] {
  return [...values].sort((left, right) => left.localeCompare(right));
}

function sortObjectEntries(record: Record<string, unknown>): Array<[string, unknown]> {
  return Object.entries(record).sort(([left], [right]) => left.localeCompare(right));
}
