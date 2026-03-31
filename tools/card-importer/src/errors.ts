// Purpose: Defines structured importer and validation errors shared by the CLI, tests, and pipeline.

export const ERROR_CODES = {
  CATEGORY_INVALID: "CATEGORY_INVALID",
  COST_FORMAT_INVALID: "COST_FORMAT_INVALID",
  FIELD_REQUIRED: "FIELD_REQUIRED",
  ID_DUPLICATE: "ID_DUPLICATE",
  KEYWORD_INVALID: "KEYWORD_INVALID",
  SCHEMA_VERSION_MISSING: "SCHEMA_VERSION_MISSING"
} as const;

export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

export type NormalizationError = {
  code: ErrorCode;
  message: string;
  recordType: string;
  recordId?: string;
  field?: string;
  sourcePath?: string;
  details?: Record<string, unknown>;
};

export class ImporterError extends Error {
  readonly issues: NormalizationError[];

  constructor(message: string, issues: NormalizationError[]) {
    super(message);
    this.name = "ImporterError";
    this.issues = issues;
  }
}

