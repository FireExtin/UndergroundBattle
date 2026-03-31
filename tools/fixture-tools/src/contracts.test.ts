// Purpose: Runs the TypeScript-side contract checks against the shared DSL fixtures and the committed normalized JSON.

import { readFile } from "node:fs/promises";

import { describe, expect, it } from "vitest";

import {
  loadFixtures,
  normalizeFixtures,
  readCurrentCardSchemaVersion,
  validateFixtures,
  validateFixtureSources
} from "./contracts.js";
import { defaultRepoRoot, normalizedOutputPath } from "./repo.js";
import type { NormalizedContractEnvelope } from "./types.js";

describe("CardLogic contract fixtures", () => {
  it("load, validate, and normalize the shared fixtures", async () => {
    const repoRoot = defaultRepoRoot();
    const currentVersion = await readCurrentCardSchemaVersion(repoRoot);
    const fixtures = await loadFixtures(repoRoot);

    expect(fixtures).toHaveLength(10);
    expect(validateFixtures(fixtures, currentVersion)).toEqual([]);
    await expect(validateFixtureSources(fixtures, repoRoot)).resolves.toEqual([]);

    const normalized = normalizeFixtures(fixtures, currentVersion, "2026-03-31T00:00:00.000Z");
    expect(normalized.records.map((record) => record.cardId)).toEqual([
      "BQ005",
      "BQ010",
      "BQ013",
      "BQ022",
      "BQ024",
      "JZ74",
      "WM088",
      "WM090",
      "XQ03",
      "XQ34"
    ]);

    const scripted = normalized.records.find((record) => record.cardId === "BQ013");
    expect(scripted).toMatchObject({
      cardName: "召现雷霆",
      sourcePath: "organized_content/cards/事/cards.json",
      scriptId: "scripts.bq013.call-lightning",
      requiresScript: true,
      pureDSLExecutable: false
    });
  });

  it("matches the committed normalized JSON artifact", async () => {
    const repoRoot = defaultRepoRoot();
    const currentVersion = await readCurrentCardSchemaVersion(repoRoot);
    const fixtures = await loadFixtures(repoRoot);

    const committed = JSON.parse(
      await readFile(normalizedOutputPath(repoRoot), "utf-8")
    ) as NormalizedContractEnvelope;
    const generated = normalizeFixtures(fixtures, currentVersion, committed.generatedAt);

    expect(committed).toEqual(generated);
  });
});
