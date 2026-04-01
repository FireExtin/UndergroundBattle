# 状态迁移入口与 CardEffect 执行分层继续收口（2026-04-01）

## 1. 输入来源

- 上下文来源：用户连续要求继续推进降复杂度改造，重点为：
  - 统一状态迁移入口（避免散写字段）
  - 规则声明与执行实现分层（engine 只做编排）
- 关联代码基线：
  - `server/pkg/rules/engine.go` 已引入 marker action 分流，但仍有 `inspect` 散写和 card effect 解析逻辑驻留在 engine。

## 2. 本轮输出落点

- 新增：
  - `server/pkg/rules/card_effect_resolution.go`
  - `server/pkg/rules/marker_actions.go`（承接上一轮完成内容，本轮继续保留）
- 修改：
  - `server/pkg/rules/engine.go`
  - `server/pkg/rules/dsl.go`
  - `server/pkg/rules/state_transitions.go`
  - `server/pkg/rules/state_transitions_test.go`

## 3. 改动动机与重点

### A) 状态迁移入口继续收口：`inspect` 不再散写

- 新增 transition helper：`markCardInspected(card *CardState, inspectorID string)`。
- 将以下散写统一替换为 helper 调用：
  - `executeInspectCard`（engine）
  - `applyInspectHandEffect`（dsl effect）
- 行为约束：
  - 自动去重同一 inspector
  - 空 inspector/no-op 安全
  - nil card/no-op 安全

对应测试：
- `TestMarkCardInspectedTransitionDeduplicatesInspector`

### B) 规则执行实现继续分层：抽离 card effect resolver

- 从 `engine.go` 抽离以下函数到 `card_effect_resolution.go`：
  - `resolveCardEffect`
  - `resolveDSLCardEffect`
  - `resolveScriptCardEffect`
  - `finalizeResolvedOperation`
  - `markOperationResolved`
- 目标：
  - `engine.go` 聚焦提交流水线编排
  - card effect 执行细节进入独立模块
- 行为保持不变（纯结构性重排）。

## 4. 结果与验证

- 代码行数变化：
  - `engine.go` 降至 `1079` 行（从上一轮 `1127` 继续下降）
- 验证命令：
  - `go test ./server/pkg/rules -run "TestMarkCardInspectedTransitionDeduplicatesInspector|TestSetMarkerActionUpdatesMarkerRegistry|TestRemoveMarkerActionCannotDropBelowZero|TestSetMarkerActionRejectsMissingMarkerType"`
  - `go test ./server/pkg/rules`
  - `go test ./server/...`
- 结果：全部通过。

## 5. 风险边界与未完成项

- 本轮未引入新规则语义，仅做结构收口与迁移 API 收敛。
- 尚未覆盖的后续收口点（建议下一轮）：
  - 将 `drawCards` 生成卡片 append 也抽象到 transition helper（进一步减少 DSL 层对 board 字段直写）
  - 继续拆分 `engine.go` 中与 legality/card source catalog 相关段落，保持“编排层”职责纯粹
  - 按同样模式逐步收口 `RandomResults`、`Board.Resolved` 等 append 类状态写入点

