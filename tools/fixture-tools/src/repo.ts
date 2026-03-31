// Purpose: Locates the shared fixture and schema files from the fixture-tools package.

import path from "node:path";
import { fileURLToPath } from "node:url";

export function defaultRepoRoot(): string {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
}

export function fixtureDirectory(repoRoot = defaultRepoRoot()): string {
  return path.join(repoRoot, "shared/contracts/fixtures");
}

export function normalizedOutputPath(repoRoot = defaultRepoRoot()): string {
  return path.join(repoRoot, "shared/contracts/normalized/card-logic.contracts.normalized.json");
}

export function cardSchemaPath(repoRoot = defaultRepoRoot()): string {
  return path.join(repoRoot, "shared/schemas/card.schema.json");
}
