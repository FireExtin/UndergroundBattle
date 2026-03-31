# Phase 3 Region Scoring

本次任务把 `Phase 3` 从“只有角色动作入口”推进到“最小地区争夺、得分、胜利条件”。

## 本次新增

- `GameState.Score`
  - `ByPlayer`
  - `VictoryThreshold`
  - `WinnerPlayerID`
- `CardState.InfluenceByPlayer`
- `CardState.ControllerID`
- 地区控制权刷新辅助：
  - `refreshAllRegionControl`
  - `refreshRegionControl`
- 最小得分与胜利辅助：
  - `awardControlledRegionPoints`
  - `evaluateWinner`

## 当前语义

- 地区控制权只看地区牌上的 `InfluenceByPlayer`
- influence 最高者成为 `ControllerID`
- 若最高值平局，地区没有控制者
- `declare_investigation` 会：
  - 让执行者 `Exhausted`
  - 增加地区总 influence counter
  - 记录到行动者名下的 `InfluenceByPlayer`
  - 立即刷新地区控制权
- `placeInfluence` DSL 作用到地区时，也会同步更新 `InfluenceByPlayer` 和控制权
- 当 `advance_phase` 让阶段从 `end` 回到 `main` 时：
  - 先按所有已控制地区为控制者加分
  - 再增加 `TurnNumber`
  - 再检查是否达到 `VictoryThreshold`

## 当前默认值

- `VictoryThreshold` 默认是 `3`
- 每个已控制地区在一次 `end -> main` 结算时提供 `1` 分

## 当前刻意未做

- 没有把 `Score` 和 `WinnerPlayerID` 投影到 Web 调试器
- 没有在新回合开始时轮转 `ActivePlayerID`
- 没有实现更复杂的地区争夺、占领、结算窗口
- 没有实现“胜利已产生后拒绝继续动作”

## 新增测试

- `region_scoring_test.go`
  - 地区按玩家分别记录 influence
  - 控制地区者在回合结束后得分
  - 平局地区不产生分数
  - 达到阈值后产生 winner
  - scoring 相关 action log 可 replay 到同一状态
- `continuous_test.go`
  - `placeInfluence` DSL 作用到地区时会同步控制权

## 为什么先这样做

这一步的目标不是完整复刻《隐秘世界》所有地区规则，而是先把“地区争夺 -> 得分 -> 胜利阈值”这条最小闭环立住，并保持：

- 可序列化
- 可回放
- 可测试
- 不引入额外 transport / UI 复杂度
