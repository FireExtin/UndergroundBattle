# 状态迁移入口继续收口（draw/random/resolved，2026-04-01）

## 1. 输入来源

- 用户指令：继续推进降复杂度改造。
- 本轮目标：把以下三类状态写入从业务逻辑中抽离到 transition helper：
  - `drawCards` 生成手牌
  - `RandomResults` 记录
  - `Board.Resolved` 记录

## 2. 输出落点

- 修改：
  - `server/pkg/rules/state_transitions.go`
  - `server/pkg/rules/dsl.go`
  - `server/pkg/rules/engine.go`
  - `server/pkg/rules/card_effect_resolution.go`
  - `server/pkg/rules/state_transitions_test.go`

## 3. 具体改动

### A. 新增 transition helpers

- `appendGeneratedDrawCard(state, operationID, ownerID, sequence)`
- `appendRandomResult(state, result)`
- `appendResolvedOperation(state, operation)`（内部使用 `cloneOperation` 防止调用方后续突变影响已提交状态）

### B. 替换业务层直接 append

- `applyDrawCardsEffect` 改为调用 `appendGeneratedDrawCard`
- `executeRollRandom` 改为调用 `appendRandomResult`
- `finalizeResolvedOperation` 改为调用 `appendResolvedOperation`

## 4. 测试与验证

新增/增强测试：
- `TestAppendGeneratedDrawCardTransition`
- `TestAppendGeneratedDrawCardTransitionRejectsInvalidInput`
- `TestAppendRandomResultTransition`
- `TestAppendResolvedOperationTransitionClonesOperation`

执行结果：
- `go test ./server/pkg/rules -run "TestAppendGeneratedDrawCardTransition|TestAppendGeneratedDrawCardTransitionRejectsInvalidInput|TestAppendRandomResultTransition|TestAppendResolvedOperationTransitionClonesOperation|TestDirectDSLCardAppliesInspectAndDrawEffects|TestSubmitActionDeterministicReplayWithSeededRandom"` ✅
- `go test ./server/pkg/rules` ✅
- `go test ./server/...` ✅

## 5. 风险边界与后续建议

- 本轮为结构性收口，未引入新规则语义。
- 仍建议继续把剩余“append 风格状态写入”逐步迁移到 transition API（尤其是跨模块共享写入点），并在 helper 层维护 invariant 约束，降低未来规则扩展时的散写回潮。

