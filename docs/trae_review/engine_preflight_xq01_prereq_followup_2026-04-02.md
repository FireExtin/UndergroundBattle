# Engine Preflight + XQ01 Prereq Follow-up (2026-04-02)

## 背景

对齐 `docs/NEXT_GEN_RULE_PLAN.md` 中“2026-04-01 第十六次补记（Engine 去重型收口下一步）”的前 4 项执行顺序，做本轮收口：

1. 抽离 `engine.go` 的通用 preflight。
2. 增加 engine 结构守卫，防止细节回流。
3. 在不引入 XQ01 全局沉默的前提下，补 `AbilityKinds` 到现有动作权限入口的框架映射。
4. 跑全套回归基线。

## 本轮改动

### 1) 通用 preflight 从 `engine.go` 抽离

- 新增：`server/pkg/rules/action_preflight_flow.go`
- 新函数：`checkActionPreflightLegality()`
  - 覆盖 `priority` 前置
  - 覆盖 `empty stack` 前置
  - 覆盖 `target player/card` 存在性前置
- `engine.go` 改为仅调用该 preflight 函数，不再内联上述通用判断。

### 2) Engine 结构守卫扩展

- 修改：`server/pkg/rules/engine_modularity_guard_test.go`
- 新增测试：`TestEngineOrchestrationGuard_NoGenericPreflightChecks`
- 守卫目标：防止上述 preflight 细节重新回流到 `engine.go`。

### 3) XQ01 prerequisite（框架层）接线

- 修改：`server/pkg/rules/action_permission_flow.go`
  - `checkCardActionPermissionLegality` 新增 `actorID` 入参。
  - 新增 `evaluateActionAbilityKindProhibition()`：
    - 将当前动作映射为 `TargetCondition.AbilityKinds`（当前映射到 `"action"`）。
    - 通过 `ScopedProhibitionChecker` 走现有 prohibition 框架判断。
    - 命中时返回 `LEGALITY_FAILED_ACTION_PROHIBITED`。
  - 新增 `abilityKindsForActionKind()`（仅框架映射，不引入生产 XQ01 规则）。
- 修改调用点：
  - `server/pkg/rules/engine.go`
  - `server/pkg/rules/role_actions.go`

> 说明：本轮不新增 XQ01 生产规则，不改变“XQ01 仍 deferred”的既有结论，仅把能力类型映射接入既有入口，避免未来再走错误全局沉默实现。

### 4) 新增测试

- 新增：`server/pkg/rules/action_permission_flow_test.go`
  - `TestEvaluateActionAbilityKindProhibition_BlocksMatchingActionKind`
  - `TestEvaluateActionAbilityKindProhibition_IgnoresMismatchedAbilityKind`

## 回归结果

- `go test ./server/... -count=1` ✅
- `(cd tools/fixture-tools && npm test)` ✅
- `(cd web && npm test)` ✅

## 风险与后续

- 当前只做了 `AbilityKinds` 的入口映射，`RegionID` 与更细粒度 ability model 仍是后续工作。
- `XQ01` 语义仍保持 deferred，避免再次误落全局 `prohibitPermission(attack/investigate)`。

