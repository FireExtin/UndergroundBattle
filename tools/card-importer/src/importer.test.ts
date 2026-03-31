// Purpose: Covers the importer pipeline with a small synthetic repository so structural regressions fail fast.

import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { ImporterError } from "./errors.js";
import { importRepositoryData } from "./importer.js";

const createdDirs: string[] = [];

afterEach(async () => {
  await Promise.all(createdDirs.splice(0).map((directory) => rm(directory, { recursive: true, force: true })));
});

describe("importRepositoryData", () => {
  it("builds normalized outputs from a minimal repository layout", async () => {
    const repoRoot = await createMinimalRepo();

    const outputs = await importRepositoryData({
      repoRoot,
      writeFiles: true,
      generatedAt: "2026-03-31T00:00:00.000Z"
    });

    expect(outputs.cardsRawIndex.records).toHaveLength(1);
    expect(outputs.cardsNormalized.records).toHaveLength(1);
    expect(outputs.rulesIndex.records).toHaveLength(1);
    expect(outputs.tokensIndex.records).toHaveLength(1);

    const card = outputs.cardsNormalized.records[0];
    expect(card).toBeDefined();
    if (!card) {
      throw new Error("expected a normalized card record");
    }

    expect(card.id).toBe("TEST001");
    expect(card.category).toBe("character");
    expect(card.cost.value).toBe(2);
    expect(card.artworkPath).toBe("resource/ymsj-fun.github.io/cards/TEST001 测试角色.jpg");

    const written = JSON.parse(
      await readFile(path.join(repoRoot, "data/normalized/cards.normalized.json"), "utf-8")
    ) as { records: Array<{ id: string }> };
    expect(written.records[0]?.id).toBe("TEST001");
  });

  it("fails with SCHEMA_VERSION_MISSING when schema versions are not configured", async () => {
    const repoRoot = await createMinimalRepo({
      schemaConfig: {
        cardsRawIndex: "0.1.0",
        cardPrint: "",
        ruleDocMeta: "0.1.0",
        tokenMeta: "0.1.0"
      }
    });

    await expect(
      importRepositoryData({
        repoRoot,
        writeFiles: false
      })
    ).rejects.toMatchObject({
      issues: expect.arrayContaining([
        expect.objectContaining({
          code: "SCHEMA_VERSION_MISSING",
          recordType: "CardPrint"
        })
      ])
    });
  });

  it("fails when duplicate card ids are produced", async () => {
    const repoRoot = await createMinimalRepo({
      extraCardRecords: {
        TEST001_DUP: {
          id: "TEST001",
          name: "重复角色",
          set: "测试集",
          "set-id": "002",
          type: "角色-测试",
          "basic-type": "角色",
          istoken: false,
          deckcard: true,
          keywords: [],
          related: [],
          color: "黄",
          cost: "3",
          lyl: "黄色",
          magic: "",
          text: "重复 ID",
          search: "",
          abl: "",
          dfc: "1",
          society: "测试秘社"
        }
      }
    });

    await expect(
      importRepositoryData({
        repoRoot,
        writeFiles: false
      })
    ).rejects.toMatchObject({
      issues: expect.arrayContaining([
        expect.objectContaining({
          code: "ID_DUPLICATE",
          recordType: "CardPrint",
          recordId: "TEST001"
        })
      ])
    });
  });
});

type MinimalRepoOptions = {
  extraCardRecords?: Record<string, unknown>;
  schemaConfig?: {
    cardsRawIndex?: string;
    cardPrint?: string;
    ruleDocMeta?: string;
    tokenMeta?: string;
  };
};

async function createMinimalRepo(options: MinimalRepoOptions = {}): Promise<string> {
  const repoRoot = await mkdtemp(path.join(os.tmpdir(), "undergroundbattle-importer-"));
  createdDirs.push(repoRoot);

  await writeRepoFile(
    repoRoot,
    "tools/card-importer/config/schema-versions.json",
    JSON.stringify(
      {
        cardsRawIndex: options.schemaConfig?.cardsRawIndex ?? "0.1.0",
        cardPrint: options.schemaConfig?.cardPrint ?? "0.1.0",
        ruleDocMeta: options.schemaConfig?.ruleDocMeta ?? "0.1.0",
        tokenMeta: options.schemaConfig?.tokenMeta ?? "0.1.0"
      },
      null,
      2
    )
  );

  await writeRepoFile(
    repoRoot,
    "organized_content/cards/角/cards.json",
    JSON.stringify(
      {
        TEST001: {
          id: "TEST001",
          name: "测试角色",
          set: "测试集",
          "set-id": "001",
          type: "角色-测试/学者",
          "basic-type": "角色",
          istoken: false,
          deckcard: true,
          keywords: ["隐秘"],
          related: [],
          color: "黄",
          cost: "2",
          lyl: "黄色",
          magic: "心灵",
          text: "持续：测试能力。",
          search: "TEST001 | 测试角色",
          abl: "［白眼］",
          dfc: "1",
          society: "测试秘社"
        },
        ...(options.extraCardRecords ?? {})
      },
      null,
      2
    )
  );
  await writeRepoFile(repoRoot, "organized_content/cards/角/cards.md", "# 测试角色\n");

  await writeRepoFile(
    repoRoot,
    "organized_content/rules/隐秘世界规则手册.md",
    "# 隐秘世界规则手册\n\n规则正文。\n"
  );

  await writeRepoFile(
    repoRoot,
    "organized_content/tokens/tokens.json",
    JSON.stringify(
      {
        TK001: {
          id: "TK001",
          name: "测试指示物",
          set: "TK",
          "set-id": "001",
          type: "指示物角色-测试",
          "basic-type": "指示物角色",
          istoken: true,
          deckcard: false,
          keywords: ["公开"],
          related: [],
          color: "灰",
          cost: "-",
          lyl: "",
          magic: "",
          text: "公开",
          search: "",
          abl: "［黑刀］",
          dfc: "1",
          society: "测试机构"
        }
      },
      null,
      2
    )
  );
  await writeRepoFile(repoRoot, "organized_content/tokens/tokens.md", "# 测试指示物\n");

  await writeRepoFile(repoRoot, "resource/ymsj-fun.github.io/cards/TEST001 测试角色.jpg", "image");
  await writeRepoFile(repoRoot, "resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf", "pdf");

  return repoRoot;
}

async function writeRepoFile(repoRoot: string, relativePath: string, content: string): Promise<void> {
  const filePath = path.join(repoRoot, relativePath);
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, content, { encoding: "utf-8", flag: "w" });
}
