import { describe, expect, it } from "vitest";

import { getProjectOverview } from "./project";

// Purpose: Provides the minimal Vitest baseline expected for all future TypeScript logic changes.
describe("getProjectOverview", () => {
  it("returns the repository invariants used by the scaffold UI", () => {
    const overview = getProjectOverview();

    expect(overview.authority).toContain("Go");
    expect(overview.typescriptResponsibilities).toContain("Web 前端与最小调试界面");
    expect(overview.testRules).toContain("旧测试必须继续通过");
  });
});

