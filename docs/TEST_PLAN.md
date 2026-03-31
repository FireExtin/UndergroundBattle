# TEST_PLAN

Purpose: records the hard testing rules that every future logic change must satisfy.

## Core Rules

- 测试是第一公民。
- 每次较重的逻辑改动都必须新增测试。
- 所有旧测试必须继续通过。
- fixture 是主卡池准入门槛。
- Go 统一使用原生 `testing`。
- TypeScript 统一使用 `Vitest`。
- 不允许删除旧测试或弱化断言来“修复”CI。

## Heavy Logic Change Definition

以下改动默认属于较重逻辑改动，必须伴随新增测试：

- legality、target、cost 判定
- stack、response、phase、step 流程
- 持续效果、replacement、prevention、trigger
- projection、replay、revision
- DSL 语义解释
- fixture 解释或 schema 消费

## Fixture Gate

- 新卡进入主卡池前，必须先补齐 fixture 与 expectation。
- fixture 必须同时通过 Go 与 TypeScript 测试。
- fixture 失败时，不允许合并对应主卡池改动。

## CI Discipline

- 任何逻辑改动都必须保证新增测试通过。
- 任何逻辑改动都必须保证旧测试继续通过。
- 如确有 breaking change，必须同时提交迁移说明、fixture 更新和断言更新理由。
- 禁止通过删除旧测试、跳过测试或弱化断言掩盖真实回归。

## Tooling Baseline

- Go 侧测试只使用标准库 `testing`，不引入第三方测试框架。
- TS 侧测试统一使用 `Vitest`，保持脚本、fixture、schema 测试入口一致。
- 共享 fixture 应优先放在 `shared/contracts`，供双端共同消费。

