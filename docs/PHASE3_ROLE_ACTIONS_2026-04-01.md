# PHASE3_ROLE_ACTIONS_2026-04-01

用途：记录 `Phase 3` 的第一刀实现，即把“角色动作入口”真正接进现有 Go 规则核，而不是继续只靠卡牌 DSL 效果做演示。

## This Task Added

- 新动作：
  - `declare_attack`
  - `declare_investigation`
- 新 operation：
  - `declare_attack`
  - `declare_investigation`
- 新结构化事件：
  - `damage_applied`
  - `card_destroyed`
  - `investigation_applied`
- 新事件字段：
  - `sourceCardId`
  - `targetCardId`
  - `appliedAmount`
  - `destroyedCardId`

核心代码：

- [`server/pkg/rules/role_actions.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/role_actions.go)
- [`server/pkg/rules/types.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/types.go)
- [`server/pkg/rules/engine.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/engine.go)
- [`server/pkg/rules/projection.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/projection.go)
- 测试：
  - [`server/pkg/rules/role_actions_test.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/role_actions_test.go)

## Design Choices

### 1. 角色动作不走 stack

当前这两个动作直接结算，不入 stack。  
原因是这一步的目标是先把“角色如何真正改动 board state”接进 rules core，而不是同时展开更复杂的战斗时序。

### 2. 角色动作读取 `EffectiveStats`

- 攻击读取 `EffectiveStats.Combat`
- 调查读取 `EffectiveStats.Investigation`

这意味着已有的 `modifyStat` continuous effects 不再只是改面板，而是真正影响后续角色动作结果。

### 3. 角色动作会使执行者 Exhausted

这是本轮明确采用的最小规则假设。  
如果不这样做，角色动作在当前规则核里会过于松散，无法形成最小行动成本。

### 4. 攻击的销毁仍复用现有最小致命判定

攻击本身只负责增加 `damage` counter。  
真正的销毁判定仍由现有 continuous / derived semantics 流程统一处理：

- `damage >= EffectiveStats.Defense`
- 则目标 `Destroyed=true`
- 并移动到 `discard`

这样可以避免起第二套平行的战斗结算语义。

## Boundaries

这轮还没有做：

- 完整战斗阶段
- 反击/阻挡/伤害分配
- 地区争夺结算
- 得分与胜利条件
- 角色动作 UI 按钮
- M0 baseline 扩成包含地区/得分的新 golden scenarios

## Why This Matters

到这一步，`Phase 3` 已经不再只是“未来计划”：

- 角色动作现在是 rules-core 的一等公民
- continuous effects 会真实影响角色动作
- replay / projection / revision 仍然走同一套 authoritative pipeline

下一步就可以在这个基础上继续接：

- 地区争夺
- 得分
- 胜利条件
