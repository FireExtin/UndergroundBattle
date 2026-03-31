// Purpose: Covers key validation failures so fixture format drift is caught before it reaches Go-side parsing.

import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { normalizeFixture, validateFixture, validateFixtureSources } from "./contracts.js";
import type { ContractFixture } from "./types.js";

describe("validateFixture", () => {
  it("rejects a mismatched schema version", () => {
    const issues = validateFixture(
      {
        ...fixture(),
        schemaVersion: "9.9.9"
      },
      "0.1.0"
    );

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SCHEMA_VERSION_MISMATCH",
          path: "DSL001.schemaVersion"
        })
      ])
    );
  });

  it("rejects an invalid effect kind", () => {
    const issues = validateFixture(
      {
        ...fixture(),
        input: {
          logic: {
            ...fixture().input.logic,
            effects: [
              {
                kind: "unknown",
                targetRef: "selected",
                amount: 1
              } as never
            ]
          }
        }
      },
      "0.1.0"
    );

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "EFFECT_KIND_INVALID"
        })
      ])
    );
  });

  it("rejects an empty basic type", () => {
    const issues = validateFixture(
      {
        ...fixture(),
        card: {
          ...fixture().card,
          basicType: "   "
        }
      },
      "0.1.0"
    );

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "BASIC_TYPE_INVALID",
          path: "DSL001.card.basicType"
        })
      ])
    );
  });

  it("marks scripted fixtures as requiring scripts during normalization", () => {
    const normalized = normalizeFixture({
      ...fixture(),
      cardId: "BQ013",
      card: {
        name: "召现雷霆",
        sourcePath: "organized_content/cards/事/cards.json",
        basicType: "事务"
      },
      input: {
        logic: {
          ...fixture().input.logic,
          id: "cards.bq013.call-lightning",
          scriptId: "scripts.bq013.call-lightning"
        }
      },
      expectations: {
        ...fixture().expectations,
        scriptId: "scripts.bq013.call-lightning"
      }
    });

    expect(normalized.cardName).toBe("召现雷霆");
    expect(normalized.sourcePath).toBe("organized_content/cards/事/cards.json");
    expect(normalized.requiresScript).toBe(true);
    expect(normalized.pureDSLExecutable).toBe(false);
  });

  it("rejects a fixture whose source metadata does not match organized content", async () => {
    const repoRoot = await mkdtemp(path.join(os.tmpdir(), "undergroundbattle-fixtures-"));
    const sourceDir = path.join(repoRoot, "organized_content/cards/事");
    await mkdir(sourceDir, { recursive: true });
    await writeFile(
      path.join(sourceDir, "cards.json"),
      `${JSON.stringify({ DSL001: { id: "DSL001", name: "错误名称", "basic-type": "事务" } }, null, 2)}\n`,
      "utf-8"
    );

    await expect(validateFixtureSources([fixture()], repoRoot)).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SOURCE_NAME_MISMATCH",
          path: "DSL001.card.name"
        })
      ])
    );
  });

  it("rejects a fixture whose source basic type does not match organized content", async () => {
    const repoRoot = await mkdtemp(path.join(os.tmpdir(), "undergroundbattle-fixtures-"));
    const sourceDir = path.join(repoRoot, "organized_content/cards/事");
    await mkdir(sourceDir, { recursive: true });
    await writeFile(
      path.join(sourceDir, "cards.json"),
      `${JSON.stringify({ DSL001: { id: "DSL001", name: "示例卡", "basic-type": "角色" } }, null, 2)}\n`,
      "utf-8"
    );

    await expect(validateFixtureSources([fixture()], repoRoot)).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SOURCE_BASIC_TYPE_MISMATCH",
          path: "DSL001.card.basicType"
        })
      ])
    );
  });
});

function fixture(): ContractFixture {
  return {
    cardId: "DSL001",
    schemaVersion: "0.1.0",
    card: {
      name: "示例卡",
      sourcePath: "organized_content/cards/事/cards.json",
      basicType: "事务"
    },
    input: {
      logic: {
        id: "cards.dsl001.example",
        schemaVersion: "0.1.0",
        speed: "slow",
        targetKinds: ["player"],
        requiresStack: false,
        durationKind: "none",
        scriptId: null,
        effects: [
          {
            kind: "inspectHand",
            targetRef: "selected"
          },
          {
            kind: "drawCards",
            targetRef: "controller",
            amount: 1
          }
        ]
      }
    },
    expectations: {
      parseOk: true,
      speed: "slow",
      targetKinds: ["player"],
      requiresStack: false,
      durationKind: "none",
      scriptId: null
    }
  };
}
