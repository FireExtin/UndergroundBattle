# Legality Hardening Step1-4（2026-04-01）

## 背景与目标

- 目标：按既定顺序完成四件事，降低规则引擎复杂度并补齐 XQ01 前置能力。
  - Step1：关键状态写入加 guardrail，防止散写回潮。
  - Step2：`queue_operation` 合法性与构建逻辑从 `engine.go` 抽离。
  - Step3：补齐 `prohibition` 对 `TargetCondition` 的匹配（XQ01 前置最小切片）。
  - Step4：继续抽离 `queue_operation` 子流程中的 window/play legality helper，进一步压薄 `engine.go`。

## 改动清单

- 新增：`server/pkg/rules/state_write_guard_test.go`
- 修改：`server/pkg/rules/state_transitions.go`
- 修改：`server/pkg/rules/engine.go`
- 新增：`server/pkg/rules/queue_operation_flow.go`
- 新增：`server/pkg/rules/queue_operation_flow_test.go`
- 修改：`server/pkg/rules/prohibition.go`
- 修改：`server/pkg/rules/prohibition_test.go`

## 关键动机与实现重点

### 1) 状态写入 guardrail（Step1）

- 动机：`FaceDown/Revealed`、`Markers`、`Resolved`、`RandomResults` 仍存在“业务层直接写状态”的风险点，后续机制扩展时容易再次分叉。
- 重点：
  - 用源码扫描测试 `TestStateTransitionWriteGuardsForCriticalFields` 固化“只能通过 transition helper 写入”的约束。
  - 发现并修复 `engine.go` 中 `executeSetFaceDown` 的直接字段赋值，改为 `setFaceDown(...)`（位于 `state_transitions.go`）。

### 2) queue_operation 流程提取（Step2）

- 动机：`engine.go` 作为编排层过重，`queue_operation` 的合法性与构建路径混杂在主 switch 中，影响可读性与后续扩展。
- 重点：
  - 提取 `checkQueueOperationActionLegality(...)` 与 `buildQueueOperationFromAction(...)` 到独立文件。
  - `engine.go` 仅保留委托调用，维持语义不变（包括 window/scope/stack 相关约束）。
  - 新增单测覆盖：缺失 `cardId` 的非法路径、构建阶段 source 元数据注入路径。

### 3) prohibition TargetCondition 匹配补齐（Step3）

- 动机：此前 `matchesTargetCategory` 忽略 `TargetCategory.Condition`，导致例如 `AbilityKinds` 不匹配时仍错误禁用，阻断 XQ01 前置语义。
- 重点：
  - 在 `prohibition.go` 中引入 `matchesProhibitionTargetCondition(...)`。
  - 覆盖字段：`Kinds`、`Keywords`、`RegionID`、`AbilityKinds`、`Side`。
  - 新增 overlap helper，按“规则声明字段非空才参与约束”原则匹配。
  - 新增回归测试：
    - `AbilityKinds` 不匹配时不得禁用。
    - `RegionID + AbilityKinds` 同时匹配时应触发禁用。

### 4) queue_operation helper 继续外提（Step4）

- 动机：Step2 完成后，`checkCardWindowLegality(...)` 与 `checkQueuedCardPlayLegality(...)` 仍留在 `engine.go`，编排层仍持有子流程细节。
- 重点：
  - 将上述两个 helper 迁移到 `queue_operation_flow.go`。
  - `engine.go` 只保留调度，不再承载 queue 子流程细节实现。
  - 语义保持不变，继续通过现有 XQ22 / role action 回归面验证。

## 验证结果

- `go test ./server/pkg/rules -run "TestProhibitionCheckerTargetConditionAbilityKindsMismatchDoesNotProhibit|TestProhibitionCheckerTargetConditionRegionAndAbilityMatchProhibits"` ✅
- `go test ./server/pkg/rules -run "TestCheckQueueOperationActionLegalityRejectsMissingCardID|TestBuildQueueOperationFromActionReturnsSourceMetadata|TestSubmitActionRejectsQueueOperationWhenXQ22ReadyOnTable|TestSubmitActionAllowsQueueOperationForNonTransactionUnderXQ22|TestRoleActionIgnoresXQ31TargetLegality"` ✅
- `go test ./server/pkg/rules` ✅
- `go test ./server/...` ✅

## 边界说明

- 本轮未启用 XQ01 完整玩法，只完成其所需的 prohibition 条件匹配底座。
- 本轮不引入新事件模型或 DSL 扩展，属于结构收口与语义修正。
