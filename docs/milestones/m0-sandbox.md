# M0 Sandbox

用途：冻结当前 live sandbox 的唯一规则基线，作为后续 golden scenario、replay、一致性和回归测试的真相说明。

## Canonical State

- 唯一初始状态来源：[`server/pkg/rules/m0.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/m0.go)
- 初始状态标识：`m0_sandbox_v1`
- 共享使用方：
  - Go 规则回归 tests
  - HTTP live sandbox session

## Initial Layout

- 玩家：`P1`、`P2`
- 起始 revision：`0`
- 起始 phase / step：`main / action`
- 起始 priority：`P1`
- RNG seed：`42`

初始牌面：

- `P1-HAND-SECRET`
  - `Secret Archive`
  - `owner=P1`
  - `zone=hand`
  - `revealed=false`
- `P1-TABLE-1`
  - `Dream Sentinel`
  - `owner=P1`
  - `zone=table`
  - `stats=2/2/0/1`
- `P2-HAND-SECRET`
  - `Black Ledger`
  - `owner=P2`
  - `zone=hand`
  - `revealed=false`
- `P2-TABLE-1`
  - `Frontline Adept`
  - `owner=P2`
  - `zone=table`
  - `keywords=blackBlade`
  - `stats=2/3/0/1`
  - `counters.damage=1`

## Supported Actions

规则核当前已实现并允许出现在 M0 baseline 的动作：

- `pass_priority`
- `advance_phase`
- `reveal_card`
- `inspect_card`
- `queue_operation`
- `resolve_top_stack`
- `roll_seeded_random`

当前 live sandbox 前端暴露的预置按钮：

- `Pass Priority`
- `Advance Phase`
- `Reveal Own Secret`
- `Inspect Own Secret`
- `Cast 读心术 (BQ010)`
- `Cast 多重梦境迷宫 (BQ005)`
- `Equip 合金指虎 (BQ022)`

## Sample Cards In Scope

- `BQ010 / 读心术`
  - 直接结算
  - 最小效果：检视对手手牌、抽 1 张占位手牌
- `BQ005 / 多重梦境迷宫`
  - 进入 stack
  - 双方连续 pass 后结算
  - 最小效果：使目标角色 `exhausted=true`
- `BQ022 / 合金指虎`
  - 直接结算
  - 通过 continuous layer 为目标附加 `blackBlade`

## M0 Regression Scenarios

场景文件位于 [`server/pkg/rules/testdata/m0`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/testdata/m0)。

当前冻结的 baseline 覆盖：

- `bootstrap-projections`
- `pass-priority-transfer`
- `double-pass-empty-stack-ends-step`
- `reveal-own-secret-public`
- `read-minds-direct-resolve`
- `multi-dream-maze-queued`
- `multi-dream-maze-resolves-after-double-pass`
- `alloy-knuckles-applies-permanent-keyword`
- `illegal-not-your-priority`

## Stable Snapshot Surface

每个 M0 scenario 只快照这些稳定面：

- `revision`
- `turn.phase / turn.step`
- `priority.currentPlayerId / windowKind / passCount`
- `stack`
- `resolved`
- `actionLog`
- `P1 / P2 / spectator` 的最终 card projection
- 结构化 rejection 的 `reasonCode / messageKey / hook / context`

辅助回归工具：

- stable-surface diff：
  - [`server/pkg/rules/regression.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/regression.go)
  - `DiffScenarioResults`
- invariant validator：
  - [`server/pkg/rules/regression.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/regression.go)
  - `ValidateScenarioResultInvariants`

## Known Non-Goals

M0 不是完整对局，也不是完整规则手册实现。本阶段明确不覆盖：

- 完整战斗阶段
- 完整地区争夺与得分闭环
- 完整 dependency engine
- WebSocket 联机
- OpenTelemetry / Playwright
- 扩大量真实卡池
- 复杂 script 卡执行

## Gating Rule

- 任何较重规则改动，必须先通过全部 M0 baseline scenarios。
- 如果需要改变 M0 行为，先修改规则，再有意识地更新 scenario fixture；不允许把未知回归直接写进新的 baseline。
