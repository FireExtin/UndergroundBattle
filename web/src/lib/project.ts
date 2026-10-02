export type ProjectOverviewModel = {
  title: string;
  authority: string;
  typescriptResponsibilities: string[];
  testRules: string[];
};

// Purpose: Centralizes the repo invariants so the first UI and the first tests share one source.
export function getProjectOverview(): ProjectOverviewModel {
  return {
    title: "正式工程骨架",
    authority: "Go 是唯一规则语义权威；TypeScript 负责 Web 前端、调试器和 DSL 作者工具。",
    typescriptResponsibilities: [
      "Web 前端与最小调试界面",
      "协议与 schema 的静态消费层",
      "Fixture 管理和 DSL 作者辅助工具"
    ],
    testRules: [
      "测试是第一公民",
      "较重逻辑改动必须新增测试",
      "旧测试必须继续通过"
    ]
  };
}

