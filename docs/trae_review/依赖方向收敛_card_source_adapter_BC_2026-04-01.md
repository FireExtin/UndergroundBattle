# 依赖方向收敛：Card Source Lookup Adapter（B + C，2026-04-01）

## 1. 输入目标

- 用户要求：继续做依赖方向收敛，把 card fixture lookup 从 `engine` 主体抽离；完成 B 后自动完成 C。
- 约束：保持规则语义不变，优先做结构收敛。

## 2. 落地范围

- 新增：
  - `server/pkg/rules/card_source_adapter.go`
  - `server/pkg/rules/card_source_adapter_test.go`
- 修改：
  - `server/pkg/rules/engine.go`
  - `server/pkg/rules/submit_pipeline.go`

## 3. B（轻量 adapter）完成内容

- 将 fixture lookup 与 fixture->source 转换逻辑从 `engine.go` 抽离到 `card_source_adapter.go`。
- 建立规则层 adapter 边界：
  - `cardOperationSourceLookup` 接口
  - `cardOperationSourceLookupFunc` 适配器
  - `defaultCardOperationSourceLookup` 默认实现（fixture catalog）
- `lookupCardOperationSource(cardID)` 仍保持原调用签名，行为不变。

## 4. C（pipeline 注入）完成内容

- `submitInternalOptions` 新增 `cardSourceLookup`。
- `submitLegalityPhase` 改为调用 `checkLegalityWithLookup(...)`。
- `submitBuildAndExecutePhase` 改为调用 `buildOperationWithLookup(...)`。
- `CheckLegality/BuildOperation` 保持原公开 API，内部通过默认 lookup 调用新内部函数，兼容现有调用方。

## 5. TDD 记录

- Red 1（B）：
  - `TestLookupCardOperationSourceUsesConfiguredAdapter`
  - 先失败：缺少 `defaultCardOperationSourceLookup` / `cardOperationSourceLookupFunc`
- Green 1（B）：
  - 新增 adapter 层后通过
- Red 2（C）：
  - `TestSubmitActionInternalUsesInjectedCardSourceLookup`
  - 先失败：`submitInternalOptions` 无 `cardSourceLookup`
- Green 2（C）：
  - 注入字段和调用链接入后通过

## 6. 验证

- `go test ./server/pkg/rules -run "TestLookupCardOperationSourceUsesConfiguredAdapter|TestSubmitActionInternalUsesInjectedCardSourceLookup"` ✅
- `go test ./server/pkg/rules` ✅
- `go test ./server/...` ✅

## 7. 风险边界

- 本次只做依赖方向收敛与可注入能力建设，不引入新规则语义。
- 仍保留默认 fixture catalog 作为生产 lookup；后续可按同接口切换缓存或远端源。

