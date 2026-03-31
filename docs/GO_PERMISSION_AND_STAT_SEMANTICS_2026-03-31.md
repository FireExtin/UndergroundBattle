# GO_PERMISSION_AND_STAT_SEMANTICS_2026-03-31

Purpose: records the next semantic step after the minimal continuous-effect layer by wiring permissions into legality checks and wiring defense into lethal-damage evaluation.

## Permission Hooks

- `grantPermission` / `prohibitPermission` no longer stop at derived state only.
- 当前第一个真实 legality 接入点是 `inspect_card`。
- 如果目标牌带有 `RequiredPermissions: ["inspect"]`，但当前没有被授予该 permission，则动作会被拒绝。
- 如果目标牌当前带有 `prohibitPermission(inspect)`，动作会被拒绝，即使也存在 grant。
- 这让持续效果第一次真正影响 `CheckLegality`，而不只是影响投影或测试辅助字段。

## Stat Semantics

- `dealDamage` 仍然先写入 `CardCounters.Damage`。
- 在同一次 commit 的后续重算里，规则核现在会比较：
  - `Counters.Damage`
  - `EffectiveStats.Defense`
- 当伤害达到致命阈值时，角色会被标记为 `Destroyed`，并从 `table` 进入 `discard`。
- 因为判定读取的是 `EffectiveStats.Defense`，所以 `modifyStat` 现在已经影响真实规则结果，而不只是面板导出值。

## Why This Split

- permission 语义属于“这个动作现在能不能做”，所以接进 legality。
- stat 语义属于“这个状态变化现在意味着什么”，所以接进 commit 后的正式状态判定。
- 这两条线一起推进，能让持续效果第一次同时影响：
  - action acceptance
  - board outcome

## Current Limits

- 当前只把 `inspect_card` 作为第一个 permission hook。
- 当前只实现了最小致命伤害判定，没有展开完整战斗系统。
- 还没有把 `grantPermission` / `prohibitPermission` 作为作者侧 DSL fixture 的正式 effect 种类公开出去；这一步目前先在 Go 规则核内建好落点。
