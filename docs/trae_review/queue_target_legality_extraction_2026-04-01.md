# Queue Target Legality 预检查外提（2026-04-01）

## 输入来源

- 用户指令：继续推进降复杂度改造。
- 当前子目标：减少 `engine.go` 中卡牌规则细节，保留编排职责。

## 输出落点

- 修改：`server/pkg/rules/engine.go`
- 修改：`server/pkg/rules/queue_operation_flow.go`
- 修改：`server/pkg/rules/engine_modularity_guard_test.go`

## 改动内容

1. 新增 `checkQueueOperationTargetLegality(state, action)`（放在 `queue_operation_flow.go`）：
   - 仅对 `ActionKindQueueOperation` 且带 `targetCardId` 的动作检查 `XQ31` 目标限制。
   - `declare_attack / declare_investigation` 保持不受该检查影响。
2. `engine.go` 中原先内联的 `BuildTargetLegalityChecker(...)` 逻辑改为调用上述 helper。
3. 扩展结构守卫：
   - 禁止 `engine.go` 继续出现 `targetLegalityChecker := BuildTargetLegalityChecker(state)`。

## 验证记录

- RED：
  - `go test ./server/pkg/rules -run TestEngineOrchestrationGuard_NoActionPermissionHelpers -count=1`（先失败，确认守卫生效）
- GREEN：
  - `go test ./server/pkg/rules -run "TestEngineOrchestrationGuard_NoActionPermissionHelpers|TestDeclareAttackIgnoresXQ31TargetLegalityRestriction|TestDeclareInvestigationIgnoresXQ31TargetLegalityRestriction|TestTargetLegalityXQ31RestrictsEnemyTargets|TestCheckQueueOperationActionLegalityRejectsMissingCardID|TestBuildQueueOperationFromActionReturnsSourceMetadata" -count=1` ✅
  - `go test ./server/...` ✅

## 风险边界

- 本次属于结构迁移，未扩展新卡语义。
- 错误优先级与旧行为保持一致：仅将检查实现位置从 `engine.go` 挪到 queue flow 模块。
