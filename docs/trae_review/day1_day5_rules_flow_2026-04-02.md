# Day1~Day5 规则闭环执行记录（2026-04-02）

## 目标

按“基础包优先、扩展后置”口径，串行完成以下 4 项：

1. 赢区去向修正（地区牌进入计分区，而非弃牌区）
2. 抓牌空库终局判定（单方失败 / 双方平局）
3. 先手标志特权显式动作化
4. 基础机制补齐：相邻地区移动 + 卡牌级标记注册表

## 已落地实现

### 1) 赢区去向与流程顺序

- `rulebook_flow.go`
  - 地区赢取后由 `moveCardToDiscard(region)` 改为 `moveCardToScore(region)`。
  - 增加 `cleanupWonRegionTableState()`，在地区入计分区前清理该地区单位与附属生命周期。
- `state_transitions.go`
  - 新增 `moveCardToScore()`，补充统一状态迁移入口。

### 2) 抓牌空库终局

- `rulebook_flow.go`
  - 增加 deck model 分支：`hasExplicitDeckModel()`。
  - 增加 `drawOneFromDeckPerPlayer()`、`applyDeckOutResult()`：
    - 单方空库：`MatchEndReasonDeckOut`
    - 双方同时空库：`MatchEndReasonDeckOutDraw`
- `types.go`
  - 增加终局原因枚举：`deck_out`、`deck_out_draw`。

### 3) 先手标志特权显式动作

- 新增 `first_player_privilege_action.go`
  - action kind: `use_first_player_privilege`
  - legality：仅先手、每回合一次、必须存在可打破的非零平局
  - execution：标记请求 -> 刷新控制权 -> 消耗特权并产出事件
- `engine.go`/`types.go`
  - 接线 action/operation/event 常量和执行分发。
- 前端 `live.ts`
  - 新增预置按钮：`Use First-Player Privilege`。

### 4) 移动与卡牌标记 V1

- 新增 `move_card_action.go`
  - action kind: `move_card`
  - 基于地区 `RegionOrder` 实现最小相邻判定。
- 新增 `card_marker_actions.go` + `marker_registry.go`
  - action kinds: `set_card_marker` / `remove_card_marker`
  - `BoardState.CardMarkers` authoritative registry。
- `projection.go` + 前端 `PlayerViewPanel.tsx`
  - 卡牌投影新增 `markers`，前端卡片详情可直接显示。

## 测试与回归

- 新增/更新测试：
  - `rulebook_flow_integration_test.go`
  - `first_player_privilege_action_test.go`
  - `movement_and_card_markers_test.go`
  - 前端 `live.test.ts`、`LiveDebuggerShell.test.tsx`
- 回归通过：
  - `go test ./server/...`
  - `cd web && npm test`
- 覆盖率（`server/pkg/rules`）约 `86%`；
  - 新增规则文件中边角/异常分支覆盖均大于 60%。

## 仍待后续阶段处理

- 先手特权费用支付仍是最小钩子，尚未接完整 payment pipeline。
- `move_card` 仍是最小相邻模型，尚未接更完整时机/来源限制。
- card marker 已有 registry，但尚未接入更多“标志被消费/触发”的卡义语义。
