// Purpose: Performs deterministic runtime validation for normalized importer outputs without third-party schema libraries.

import { ERROR_CODES, type NormalizationError } from "./errors.js";
import { CARD_CATEGORY_VALUES, type CardCategory, type CardPrint, type RuleDocMeta, type TokenMeta } from "./types.js";

const CATEGORY_SET = new Set<CardCategory>(CARD_CATEGORY_VALUES);
const CARD_LIKE_ID_PATTERN = /^[A-Za-z0-9_-]+$/;
const SEMVER_PATTERN = /^\d+\.\d+\.\d+$/;
const COST_PATTERN = /^$|^-$|^\d+$/;

export function validateCardPrints(records: CardPrint[]): NormalizationError[] {
  return [
    ...validateUniqueIds("CardPrint", records.map((record) => ({ id: record.id, sourcePath: record.sourcePath }))),
    ...records.flatMap((record) => validateCardPrint(record))
  ];
}

export function validateCardPrint(record: CardPrint): NormalizationError[] {
  return [
    ...validateSchemaVersion("CardPrint", record.schemaVersion, record.id, record.sourcePath),
    ...validateBaseIdentity("CardPrint", record.id, record.name, record.sourcePath),
    ...validateCategory("CardPrint", record.category, record.id, record.sourcePath),
    ...validateSourcePath("CardPrint", record.sourcePath, record.id),
    ...validateRequiredString("CardPrint", "rawText", record.rawText, record.id, record.sourcePath),
    ...validateKeywords("CardPrint", record.keywords, record.id, record.sourcePath),
    ...validateCost("CardPrint", record.cost.raw, record.id, record.sourcePath)
  ];
}

export function validateRuleDocMetas(records: RuleDocMeta[]): NormalizationError[] {
  return [
    ...validateUniqueIds("RuleDocMeta", records.map((record) => ({ id: record.id, sourcePath: record.sourcePath }))),
    ...records.flatMap((record) => validateRuleDocMeta(record))
  ];
}

export function validateRuleDocMeta(record: RuleDocMeta): NormalizationError[] {
  return [
    ...validateSchemaVersion("RuleDocMeta", record.schemaVersion, record.id, record.sourcePath),
    ...validateNamedIdentity("RuleDocMeta", record.id, record.title, record.sourcePath),
    ...validateSourcePath("RuleDocMeta", record.sourcePath, record.id),
    ...validateRequiredString("RuleDocMeta", "description", record.description, record.id, record.sourcePath),
    ...validateRequiredString("RuleDocMeta", "rawText", record.rawText, record.id, record.sourcePath)
  ];
}

export function validateTokenMetas(records: TokenMeta[]): NormalizationError[] {
  return [
    ...validateUniqueIds("TokenMeta", records.map((record) => ({ id: record.id, sourcePath: record.sourcePath }))),
    ...records.flatMap((record) => validateTokenMeta(record))
  ];
}

export function validateTokenMeta(record: TokenMeta): NormalizationError[] {
  return [
    ...validateSchemaVersion("TokenMeta", record.schemaVersion, record.id, record.sourcePath),
    ...validateBaseIdentity("TokenMeta", record.id, record.name, record.sourcePath),
    ...validateCategory("TokenMeta", record.category, record.id, record.sourcePath),
    ...validateSourcePath("TokenMeta", record.sourcePath, record.id),
    ...validateRequiredString("TokenMeta", "rawText", record.rawText, record.id, record.sourcePath),
    ...validateKeywords("TokenMeta", record.keywords, record.id, record.sourcePath),
    ...validateCost("TokenMeta", record.cost.raw, record.id, record.sourcePath)
  ];
}

type IdentifiedSource = {
  id: string;
  sourcePath?: string;
};

function validateUniqueIds(recordType: string, records: IdentifiedSource[]): NormalizationError[] {
  const seen = new Map<string, string | undefined>();
  const issues: NormalizationError[] = [];

  for (const record of records) {
    const previous = seen.get(record.id);
    if (previous !== undefined) {
      const issue: NormalizationError = {
        code: ERROR_CODES.ID_DUPLICATE,
        message: `Duplicate ${recordType} id "${record.id}" detected.`,
        recordType,
        recordId: record.id,
        field: "id",
        details: {
          firstSourcePath: previous,
          duplicateSourcePath: record.sourcePath
        }
      };
      if (record.sourcePath !== undefined) {
        issue.sourcePath = record.sourcePath;
      }

      issues.push(issue);
      continue;
    }

    seen.set(record.id, record.sourcePath);
  }

  return issues;
}

function validateBaseIdentity(recordType: string, id: string, name: string, sourcePath: string): NormalizationError[] {
  return [
    ...validateRequiredString(recordType, "id", id, id, sourcePath),
    ...validateRequiredString(recordType, "name", name, id, sourcePath),
    ...(CARD_LIKE_ID_PATTERN.test(id)
      ? []
      : [
          {
            code: ERROR_CODES.FIELD_REQUIRED,
            message: `Invalid ${recordType} id "${id}".`,
            recordType,
            recordId: id,
            field: "id",
            sourcePath,
            details: {
              expectedPattern: CARD_LIKE_ID_PATTERN.source
            }
          }
        ])
  ];
}

function validateNamedIdentity(recordType: string, id: string, name: string, sourcePath: string): NormalizationError[] {
  return [
    ...validateRequiredString(recordType, "id", id, id, sourcePath),
    ...validateRequiredString(recordType, "name", name, id, sourcePath)
  ];
}

function validateSourcePath(recordType: string, sourcePath: string, recordId: string): NormalizationError[] {
  return validateRequiredString(recordType, "sourcePath", sourcePath, recordId, sourcePath);
}

function validateRequiredString(
  recordType: string,
  field: string,
  value: string,
  recordId: string,
  sourcePath: string
): NormalizationError[] {
  if (typeof value === "string" && value.trim().length > 0) {
    return [];
  }

  return [
    {
      code: field === "schemaVersion" ? ERROR_CODES.SCHEMA_VERSION_MISSING : ERROR_CODES.FIELD_REQUIRED,
      message: `${recordType} is missing required field "${field}".`,
      recordType,
      recordId,
      field,
      sourcePath
    }
  ];
}

function validateSchemaVersion(
  recordType: string,
  schemaVersion: string,
  recordId: string,
  sourcePath: string
): NormalizationError[] {
  if (typeof schemaVersion !== "string" || schemaVersion.trim().length === 0) {
    return [
      {
        code: ERROR_CODES.SCHEMA_VERSION_MISSING,
        message: `${recordType} is missing schemaVersion.`,
        recordType,
        recordId,
        field: "schemaVersion",
        sourcePath
      }
    ];
  }

  if (SEMVER_PATTERN.test(schemaVersion)) {
    return [];
  }

  return [
    {
      code: ERROR_CODES.FIELD_REQUIRED,
      message: `${recordType} schemaVersion "${schemaVersion}" is not valid semver.`,
      recordType,
      recordId,
      field: "schemaVersion",
      sourcePath,
      details: {
        expectedPattern: SEMVER_PATTERN.source
      }
    }
  ];
}

function validateCategory(
  recordType: string,
  category: string,
  recordId: string,
  sourcePath: string
): NormalizationError[] {
  if (CATEGORY_SET.has(category as CardCategory)) {
    return [];
  }

  return [
    {
      code: ERROR_CODES.CATEGORY_INVALID,
      message: `${recordType} category "${category}" is not supported.`,
      recordType,
      recordId,
      field: "category",
      sourcePath,
      details: {
        allowed: CARD_CATEGORY_VALUES
      }
    }
  ];
}

function validateKeywords(
  recordType: string,
  keywords: string[],
  recordId: string,
  sourcePath: string
): NormalizationError[] {
  const issues: NormalizationError[] = [];

  for (const keyword of keywords) {
    if (typeof keyword !== "string" || keyword.trim().length === 0 || keyword !== keyword.trim()) {
      issues.push({
        code: ERROR_CODES.KEYWORD_INVALID,
        message: `${recordType} keyword "${keyword}" is invalid.`,
        recordType,
        recordId,
        field: "keywords",
        sourcePath
      });
    }
  }

  return issues;
}

function validateCost(
  recordType: string,
  rawCost: string,
  recordId: string,
  sourcePath: string
): NormalizationError[] {
  if (typeof rawCost === "string" && COST_PATTERN.test(rawCost)) {
    return [];
  }

  return [
    {
      code: ERROR_CODES.COST_FORMAT_INVALID,
      message: `${recordType} cost "${rawCost}" is invalid.`,
      recordType,
      recordId,
      field: "cost.raw",
      sourcePath,
      details: {
        expectedPattern: COST_PATTERN.source
      }
    }
  ];
}
