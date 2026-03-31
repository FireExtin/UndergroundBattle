// Purpose: Verifies that validation rejects malformed normalized records with structured, stable error codes.

import { describe, expect, it } from "vitest";

import type { CardPrint, RuleDocMeta, TokenMeta } from "./types.js";
import { validateCardPrint, validateCardPrints, validateRuleDocMeta, validateTokenMeta } from "./validators.js";

describe("validators", () => {
  it("accepts valid normalized records", () => {
    expect(validateCardPrint(validCardPrint())).toEqual([]);
    expect(validateRuleDocMeta(validRuleDocMeta())).toEqual([]);
    expect(validateTokenMeta(validTokenMeta())).toEqual([]);
  });

  it("rejects invalid category, keyword, and cost formats", () => {
    const issues = validateCardPrint({
      ...validCardPrint(),
      category: "bad-category",
      keywords: ["", " 隐秘"],
      cost: {
        raw: "X",
        value: null
      }
    } as unknown as CardPrint);

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ code: "CATEGORY_INVALID", field: "category" }),
        expect.objectContaining({ code: "KEYWORD_INVALID", field: "keywords" }),
        expect.objectContaining({ code: "COST_FORMAT_INVALID", field: "cost.raw" })
      ])
    );
  });

  it("rejects duplicate ids across a normalized card dataset", () => {
    const issues = validateCardPrints([
      validCardPrint(),
      {
        ...validCardPrint(),
        sourcePath: "organized_content/cards/附/cards.json"
      }
    ]);

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "ID_DUPLICATE",
          recordId: "TEST001"
        })
      ])
    );
  });

  it("rejects missing required fields", () => {
    const issues = validateCardPrint({
      ...validCardPrint(),
      name: ""
    });

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "FIELD_REQUIRED",
          field: "name",
          recordId: "TEST001"
        })
      ])
    );
  });

  it("returns SCHEMA_VERSION_MISSING when schemaVersion is absent", () => {
    const issues = validateCardPrint({
      ...validCardPrint(),
      schemaVersion: ""
    });

    expect(issues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SCHEMA_VERSION_MISSING",
          field: "schemaVersion",
          recordId: "TEST001"
        })
      ])
    );
  });
});

function validCardPrint(): CardPrint {
  return {
    id: "TEST001",
    schemaVersion: "0.1.0",
    name: "测试角色",
    category: "character",
    basicType: "角色",
    typeLine: "角色-测试",
    traits: ["测试"],
    sourcePath: "organized_content/cards/角/cards.json",
    sourceMarkdownPath: "organized_content/cards/角/cards.md",
    artworkPath: null,
    sourceSet: "测试集",
    sourceSetNumber: "001",
    rawText: "持续：测试能力。",
    text: ["持续：测试能力。"],
    keywords: ["隐秘"],
    relatedIds: [],
    cost: {
      raw: "1",
      value: 1
    },
    defense: {
      raw: "1",
      value: 1
    },
    loyalty: {
      raw: "黄色",
      symbols: ["黄色"]
    },
    abilityIcons: ["［白眼］"],
    color: "黄",
    magicDomain: "心灵",
    society: "测试秘社",
    searchText: "TEST001 | 测试角色",
    flags: {
      isToken: false,
      isDeckCard: true
    }
  };
}

function validRuleDocMeta(): RuleDocMeta {
  return {
    id: "隐秘世界规则手册",
    schemaVersion: "0.1.0",
    title: "隐秘世界规则手册",
    description: "规则手册描述",
    sourcePath: "organized_content/rules/隐秘世界规则手册.md",
    sourcePdfPath: "resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf",
    rawText: "# 隐秘世界规则手册",
    headings: ["隐秘世界规则手册"],
    lineCount: 1,
    characterCount: 11,
    contentHash: "abc"
  };
}

function validTokenMeta(): TokenMeta {
  return {
    id: "TK001",
    schemaVersion: "0.1.0",
    name: "测试指示物",
    category: "token",
    basicType: "指示物角色",
    typeLine: "指示物角色-测试",
    traits: ["测试"],
    sourcePath: "organized_content/tokens/tokens.json",
    sourceMarkdownPath: "organized_content/tokens/tokens.md",
    rawText: "公开",
    text: ["公开"],
    keywords: ["公开"],
    cost: {
      raw: "-",
      value: null
    },
    defense: {
      raw: "1",
      value: 1
    },
    abilityIcons: ["［黑刀］"],
    color: "灰",
    magicDomain: null,
    society: "测试机构",
    linkedCardPrintId: "TK001"
  };
}
