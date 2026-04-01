# State Write Guard 扩围（2026-04-01）

## 输入来源

- 用户指令：继续沿降复杂度路线推进。
- 当前目标：强化“统一状态迁移入口”的约束，防止关键字段被业务层散写。

## 输出落点

- 修改：`server/pkg/rules/state_write_guard_test.go`

## 改动内容

- 在 `TestStateTransitionWriteGuardsForCriticalFields` 新增 3 组 guard：
  - `exhausted_assignment`：禁止业务层直接写 `Board.Cards[i].Exhausted`
  - `destroyed_assignment`：禁止业务层直接写 `Board.Cards[i].Destroyed`
  - `zone_assignment`：禁止业务层直接写 `Board.Cards[i].Zone`
- 三项均限定只能在 `state_transitions.go` 发生写入。

## 验证记录

- `go test ./server/pkg/rules -run "TestStateTransitionWriteGuardsForCriticalFields|TestEngineOrchestrationGuard_NoActionPermissionHelpers" -count=1` ✅
- `go test ./server/...` ✅

## 风险边界

- 本次仅增强 guard，不改变规则语义。
- 后续如果新增合法迁移 helper，应同步更新 guard 白名单，避免误报。
