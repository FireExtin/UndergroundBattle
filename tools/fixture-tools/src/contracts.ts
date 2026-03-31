// Purpose: Loads shared DSL fixtures, validates them, and produces the normalized JSON artifact consumed by downstream tooling.

import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

import { cardSchemaPath, defaultRepoRoot, fixtureDirectory, normalizedOutputPath } from "./repo.js";
import type { ContractFixture, NormalizedContractEnvelope, NormalizedContractRecord, ValidationIssue } from "./types.js";
import { DURATION_KIND_VALUES, EFFECT_KIND_VALUES, SPEED_VALUES, TARGET_KIND_VALUES } from "./types.js";

const CARD_ID_PATTERN = /^[A-Z0-9_-]+$/;
const LOGIC_ID_PATTERN = /^[a-z0-9][a-z0-9._/-]*$/;
const SCRIPT_ID_PATTERN = /^[a-z0-9][a-z0-9._/-]*$/;
const SOURCE_PATH_PATTERN = /^organized_content\/.+\.json$/;

export async function readCurrentCardSchemaVersion(repoRoot = defaultRepoRoot()): Promise<string> {
  const payload = JSON.parse(await readFile(cardSchemaPath(repoRoot), "utf-8")) as {
    "x-currentSchemaVersion"?: string;
    properties?: {
      schemaVersion?: {
        const?: string;
      };
    };
  };

  const version = payload["x-currentSchemaVersion"] ?? payload.properties?.schemaVersion?.const;
  if (!version) {
    throw new Error("shared/schemas/card.schema.json must declare the current schema version");
  }

  return version;
}

export async function loadFixtures(repoRoot = defaultRepoRoot()): Promise<ContractFixture[]> {
  const fixturesRoot = fixtureDirectory(repoRoot);
  const files = (await readdir(fixturesRoot))
    .filter((name) => name.endsWith(".fixture.json"))
    .sort((left, right) => left.localeCompare(right));

  const fixtures = await Promise.all(
    files.map(async (name) => JSON.parse(await readFile(path.join(fixturesRoot, name), "utf-8")) as ContractFixture)
  );

  return fixtures;
}

export function validateFixtures(fixtures: ContractFixture[], currentVersion: string): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const seenCardIds = new Set<string>();
  const seenLogicIds = new Set<string>();

  for (const fixture of fixtures) {
    issues.push(...validateFixture(fixture, currentVersion));

    if (seenCardIds.has(fixture.cardId)) {
      issues.push({
        code: "CARD_ID_DUPLICATE",
        message: `Duplicate cardId "${fixture.cardId}" found in fixtures.`,
        path: `${fixture.cardId}.cardId`
      });
    }
    seenCardIds.add(fixture.cardId);

    if (seenLogicIds.has(fixture.input.logic.id)) {
      issues.push({
        code: "LOGIC_ID_DUPLICATE",
        message: `Duplicate logic id "${fixture.input.logic.id}" found in fixtures.`,
        path: `${fixture.cardId}.input.logic.id`
      });
    }
    seenLogicIds.add(fixture.input.logic.id);
  }

  return issues;
}

export async function validateFixtureSources(
  fixtures: ContractFixture[],
  repoRoot = defaultRepoRoot()
): Promise<ValidationIssue[]> {
  const issues: ValidationIssue[] = [];
  const sourceCache = new Map<string, Record<string, { id?: string; name?: string }>>();

  for (const fixture of fixtures) {
    issues.push(...(await validateFixtureSource(fixture, repoRoot, sourceCache)));
  }

  return issues;
}

export function validateFixture(fixture: ContractFixture, currentVersion: string): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  if (!CARD_ID_PATTERN.test(fixture.cardId)) {
    issues.push(issue("CARD_ID_INVALID", fixture.cardId, "cardId must match ^[A-Z0-9_-]+$"));
  }

  if (fixture.card.name.trim().length === 0) {
    issues.push(issue("CARD_NAME_INVALID", `${fixture.cardId}.card.name`, "card.name must not be empty"));
  }

  if (!SOURCE_PATH_PATTERN.test(fixture.card.sourcePath)) {
    issues.push(
      issue(
        "SOURCE_PATH_INVALID",
        `${fixture.cardId}.card.sourcePath`,
        "card.sourcePath must point at organized_content JSON"
      )
    );
  }

  if (fixture.card.basicType.trim().length === 0) {
    issues.push(issue("BASIC_TYPE_INVALID", `${fixture.cardId}.card.basicType`, "card.basicType must not be empty"));
  }

  if (fixture.schemaVersion !== currentVersion) {
    issues.push(issue("SCHEMA_VERSION_MISMATCH", `${fixture.cardId}.schemaVersion`, `expected ${currentVersion}`));
  }

  const logic = fixture.input.logic;
  if (!LOGIC_ID_PATTERN.test(logic.id)) {
    issues.push(issue("LOGIC_ID_INVALID", `${fixture.cardId}.input.logic.id`, "logic id must be lowercase and path-safe"));
  }

  if (logic.schemaVersion !== currentVersion) {
    issues.push(
      issue("SCHEMA_VERSION_MISMATCH", `${fixture.cardId}.input.logic.schemaVersion`, `expected ${currentVersion}`)
    );
  }

  if (!SPEED_VALUES.includes(logic.speed)) {
    issues.push(issue("SPEED_INVALID", `${fixture.cardId}.input.logic.speed`, `unsupported speed ${logic.speed}`));
  }

  if (!DURATION_KIND_VALUES.includes(logic.durationKind)) {
    issues.push(
      issue(
        "DURATION_KIND_INVALID",
        `${fixture.cardId}.input.logic.durationKind`,
        `unsupported durationKind ${logic.durationKind}`
      )
    );
  }

  for (const [index, targetKind] of logic.targetKinds.entries()) {
    if (!TARGET_KIND_VALUES.includes(targetKind)) {
      issues.push(
        issue(
          "TARGET_KIND_INVALID",
          `${fixture.cardId}.input.logic.targetKinds[${index}]`,
          `unsupported target kind ${targetKind}`
        )
      );
    }
  }

  if (logic.scriptId !== null && !SCRIPT_ID_PATTERN.test(logic.scriptId)) {
    issues.push(issue("SCRIPT_ID_INVALID", `${fixture.cardId}.input.logic.scriptId`, "invalid scriptId format"));
  }

  if (logic.effects.length === 0) {
    issues.push(issue("EFFECTS_EMPTY", `${fixture.cardId}.input.logic.effects`, "effects must not be empty"));
  }

  for (const [index, effect] of logic.effects.entries()) {
    if (!EFFECT_KIND_VALUES.includes(effect.kind)) {
      issues.push(
        issue("EFFECT_KIND_INVALID", `${fixture.cardId}.input.logic.effects[${index}].kind`, effect.kind)
      );
    }
  }

  if (fixture.expectations.speed !== logic.speed) {
    issues.push(issue("EXPECTATION_MISMATCH", `${fixture.cardId}.expectations.speed`, "speed must match input.logic"));
  }

  if (fixture.expectations.requiresStack !== logic.requiresStack) {
    issues.push(
      issue("EXPECTATION_MISMATCH", `${fixture.cardId}.expectations.requiresStack`, "requiresStack must match input.logic")
    );
  }

  if (fixture.expectations.durationKind !== logic.durationKind) {
    issues.push(
      issue(
        "EXPECTATION_MISMATCH",
        `${fixture.cardId}.expectations.durationKind`,
        "durationKind must match input.logic"
      )
    );
  }

  if (fixture.expectations.scriptId !== logic.scriptId) {
    issues.push(issue("EXPECTATION_MISMATCH", `${fixture.cardId}.expectations.scriptId`, "scriptId must match input.logic"));
  }

  if (fixture.expectations.targetKinds.length !== logic.targetKinds.length) {
    issues.push(
      issue("EXPECTATION_MISMATCH", `${fixture.cardId}.expectations.targetKinds`, "targetKinds length must match input.logic")
    );
  } else {
    for (const [index, targetKind] of fixture.expectations.targetKinds.entries()) {
      if (targetKind !== logic.targetKinds[index]) {
        issues.push(
          issue(
            "EXPECTATION_MISMATCH",
            `${fixture.cardId}.expectations.targetKinds[${index}]`,
            "targetKinds must match input.logic"
          )
        );
      }
    }
  }

  return issues;
}

export function normalizeFixture(fixture: ContractFixture): NormalizedContractRecord {
  const scriptId = fixture.input.logic.scriptId;
  return {
    cardId: fixture.cardId,
    cardName: fixture.card.name,
    sourcePath: fixture.card.sourcePath,
    basicType: fixture.card.basicType,
    schemaVersion: fixture.schemaVersion,
    logicId: fixture.input.logic.id,
    speed: fixture.input.logic.speed,
    targetKinds: [...fixture.input.logic.targetKinds],
    requiresStack: fixture.input.logic.requiresStack,
    durationKind: fixture.input.logic.durationKind,
    scriptId,
    requiresScript: scriptId !== null,
    pureDSLExecutable: scriptId === null,
    effectKinds: fixture.input.logic.effects.map((effect) => effect.kind)
  };
}

export function normalizeFixtures(
  fixtures: ContractFixture[],
  schemaVersion: string,
  generatedAt = new Date().toISOString()
): NormalizedContractEnvelope {
  return {
    schemaVersion,
    generatedAt,
    recordType: "CardLogicContract",
    records: fixtures.map(normalizeFixture).sort((left, right) => left.cardId.localeCompare(right.cardId))
  };
}

export async function generateNormalizedFixtures(repoRoot = defaultRepoRoot()): Promise<NormalizedContractEnvelope> {
  const currentVersion = await readCurrentCardSchemaVersion(repoRoot);
  const fixtures = await loadFixtures(repoRoot);
  const issues = [...validateFixtures(fixtures, currentVersion), ...(await validateFixtureSources(fixtures, repoRoot))];
  if (issues.length > 0) {
    throw new Error(JSON.stringify(issues, null, 2));
  }

  const normalized = normalizeFixtures(fixtures, currentVersion);
  await writeFile(normalizedOutputPath(repoRoot), `${JSON.stringify(normalized, null, 2)}\n`, "utf-8");
  return normalized;
}

function issue(code: string, path: string, message: string): ValidationIssue {
  return { code, message, path };
}

async function validateFixtureSource(
  fixture: ContractFixture,
  repoRoot: string,
  sourceCache: Map<string, Record<string, { id?: string; name?: string; "basic-type"?: string }>>
): Promise<ValidationIssue[]> {
  const issues: ValidationIssue[] = [];
  if (!SOURCE_PATH_PATTERN.test(fixture.card.sourcePath)) {
    return issues;
  }

  const sourcePath = path.join(repoRoot, fixture.card.sourcePath);
  let sourceIndex = sourceCache.get(sourcePath);
  if (!sourceIndex) {
    try {
      sourceIndex = JSON.parse(await readFile(sourcePath, "utf-8")) as Record<
        string,
        { id?: string; name?: string; "basic-type"?: string }
      >;
      sourceCache.set(sourcePath, sourceIndex);
    } catch {
      return [
        issue(
          "SOURCE_PATH_UNREADABLE",
          `${fixture.cardId}.card.sourcePath`,
          `unable to read ${fixture.card.sourcePath}`
        )
      ];
    }
  }

  const sourceRecord = sourceIndex[fixture.cardId];
  if (!sourceRecord) {
    issues.push(
      issue(
        "SOURCE_RECORD_MISSING",
        `${fixture.cardId}.card.sourcePath`,
        `organized content does not contain ${fixture.cardId}`
      )
    );
    return issues;
  }

  if (sourceRecord.id && sourceRecord.id !== fixture.cardId) {
    issues.push(
      issue(
        "SOURCE_RECORD_ID_MISMATCH",
        `${fixture.cardId}.card.sourcePath`,
        `organized content id ${sourceRecord.id} does not match fixture`
      )
    );
  }

  if (sourceRecord.name !== fixture.card.name) {
    issues.push(
      issue(
        "SOURCE_NAME_MISMATCH",
        `${fixture.cardId}.card.name`,
        `organized content name ${sourceRecord.name ?? "(missing)"} does not match fixture`
      )
    );
  }

  if (sourceRecord["basic-type"] !== fixture.card.basicType) {
    issues.push(
      issue(
        "SOURCE_BASIC_TYPE_MISMATCH",
        `${fixture.cardId}.card.basicType`,
        `organized content basic-type ${sourceRecord["basic-type"] ?? "(missing)"} does not match fixture`
      )
    );
  }

  return issues;
}
