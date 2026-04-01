# Engine Modularity：Action Permission Flow 收口（2026-04-01）

## 输入来源

- 用户指令：继续按既定降复杂度路线推进。
- 当前阶段目标：继续压薄 `engine.go`，让其仅保留 orchestration。

## 输出落点

- 新增：`server/pkg/rules/action_permission_flow.go`
- 新增：`server/pkg/rules/engine_modularity_guard_test.go`
- 修改：`server/pkg/rules/engine.go`

## 改动内容

1. 将动作权限合法性 helper 从 `engine.go` 外提到独立模块：
   - `checkCardActionPermissionLegality(...)`
   - `permissionForActionKind(...)`
2. 增加结构守卫测试 `TestEngineOrchestrationGuard_NoActionPermissionHelpers`：
   - 禁止以上 helper 回流到 `engine.go`。
3. 语义保持不变：
   - `prohibitions` 拦截动作；
   - `requiredPermissions` 在未 grant 时拒绝动作；
   - 动作到 permission 的映射保持原值（`inspect/reveal/set_face_down/attack/investigate`）。

## 验证记录

- RED：
  - `go test ./server/pkg/rules -run TestEngineOrchestrationGuard_NoActionPermissionHelpers -count=1`（初始失败，确认守卫有效）
- GREEN：
  - `go test ./server/pkg/rules -run "TestEngineOrchestrationGuard_NoActionPermissionHelpers|TestInspectCardRejectedWhenContinuousProhibitionBlocksAction|TestInspectCardRequiresGrantedPermission|TestInspectCardSucceedsWhenPermissionIsGranted|TestDeclareAttackAppliesCombatDamageAndExhaustsAttacker|TestDeclareInvestigationPlacesInfluenceOnRegionAndExhaustsInvestigator" -count=1` ✅
  - `go test ./server/...` ✅

## 风险边界

- 本次是结构收口，不引入新规则语义。
- 仍需继续推进：
  - 状态迁移写入 guard 扩围（`Destroyed/Zone/Exhausted`）；
  - `engine.go` 继续拆出局部 legality 子流程。
