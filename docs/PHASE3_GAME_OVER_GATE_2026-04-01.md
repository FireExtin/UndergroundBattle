# Phase 3 Game Over Gate

本次任务把 `Phase 3` 再推进一格：当 `WinnerPlayerID` 已经写入权威状态后，对局不再继续接受新动作。

## 本次新增

### Go 侧

- 新增结构化拒绝码：
  - `RULES_FAILED_GAME_ALREADY_OVER`
- `CheckLegality` 现在会在基础结构校验之后、规则动作校验之前检查：
  - `state.Score.WinnerPlayerID != ""`
- 若已有 winner：
  - 返回 `rules.game.already_over`
  - `hook = score.winner`
  - `context.winnerPlayerId = <winner>`

### Web 侧

- live debugger 会读取当前 patch 里的 `score.winnerPlayerId`
- 若当前 patch 已存在 winner：
  - 预置动作按钮全部禁用
  - 显示 `Game over. Winner: <player>`
  - `Reload Feed` 仍可用

## 为什么这样做

前一轮已经有：

- 地区得分
- 胜利阈值
- `WinnerPlayerID`

如果对局在 winner 出现后仍能继续提交动作，那这个 winner 只是“显示用字段”，不是真正的规则边界。  
这一步把 winner 从“可见结果”推进成“会影响 legality 的规则状态”。

## 新增测试

### Go

- `TestWinnerStopsFurtherActionsWithStructuredReasonCode`
  - 胜利达成后继续动作会被拒绝
  - 拒绝码为 `RULES_FAILED_GAME_ALREADY_OVER`
  - rejection context 包含 `winnerPlayerId`

### Web

- `LiveDebuggerShell` 新增测试：
  - live patch 已有 winner 时，动作按钮被禁用
  - 页面显示 `Game over. Winner: <player>`

## 当前刻意未做

- 没有把“对局结束”做成独立 phase 或 terminal state enum
- 没有实现赛后页面或重开一局入口
- 没有把 winner 相关行为扩展到更完整的房间/session 生命周期

## 当前结果

当前最小 sandbox 的行为已经变成：

1. 地区控制产生分数
2. 分数达到阈值时写入 winner
3. 新回合主动玩家继续正常轮转
4. 一旦已有 winner，后续动作直接被规则核拒绝
