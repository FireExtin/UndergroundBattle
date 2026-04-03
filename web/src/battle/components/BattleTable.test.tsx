import { describe, expect, it } from "vitest";

import { formatPlayerValueSummary } from "./BattleTable";

describe("formatPlayerValueSummary", () => {
  it("renders preferred players first and appends unknown player ids dynamically", () => {
    const summary = formatPlayerValueSummary(
      {
        Alpha: 4,
        Beta: 1,
        Gamma: 2
      },
      ["Beta", "Alpha"]
    );

    expect(summary).toBe("Beta 1 · Alpha 4 · Gamma 2");
  });

  it("uses zero-like fallback for preferred players without explicit values", () => {
    const summary = formatPlayerValueSummary(
      {
        Alpha: 3
      },
      ["Beta", "Alpha"]
    );

    expect(summary).toBe("Beta 0 · Alpha 3");
  });
});
