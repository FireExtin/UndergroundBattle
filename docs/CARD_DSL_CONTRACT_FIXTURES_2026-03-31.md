# CARD_DSL_CONTRACT_FIXTURES_2026-03-31

Purpose: explains the repository changes that introduced minimal CardLogic DSL fixtures and dual-runtime contract tests.

## Scope

- 新增最小 `CardLogic` DSL schema。
- 新增 5 个共享 fixture，覆盖单目标地区、无目标即时效果、快速入堆叠效果、本回合修正和 `scriptId` 特殊牌。
- 新增 TypeScript 侧 fixture loader、validator、normalizer 和 `Vitest` 契约测试。
- 新增 Go 侧 fixture 解析与原生 `testing` 契约测试。

## Important Decisions

- fixture 改为自包含结构，`expectations` 直接内联在 fixture JSON 中。
- `scriptId` 非空时，不允许把该卡当作纯 DSL 卡处理。
- `shared/schemas/card.schema.json` 现在显式声明当前版本为 `0.1.0`，fixture 与 DSL logic 都必须对齐这个版本。

## Output

- fixture 源文件位于 `shared/contracts/fixtures/`。
- TypeScript 归一化产物位于 `shared/contracts/normalized/card-logic.contracts.normalized.json`。
- 规则说明位于 `docs/CARD_DSL.md`。

## Current Limit

- 当前 DSL schema 只覆盖最小基础效果，目标是双端契约稳定，不是完整规则引擎。
- 复杂牌仍可通过 `scriptId` 进入 Go 的脚本解释路径。

