# GO_CONTINUOUS_EFFECTS_2026-03-31

Purpose: records the first minimal continuous-effect layer added to the Go rules authority.

## What Changed

- 新增 `ContinuousEffect`、`ContinuousLayer`、`ContinuousEffectRegistry` 和 `RecalculateContinuousEffects(state)`。
- 当前最小 layer 为：
  - `LayerProhibition`
  - `LayerPermission`
  - `LayerCost`
  - `LayerNumeric`
  - `LayerActionQuota`
- 同层按 `Timestamp` 排序，跨层按固定 layer 顺序执行，其中禁止优先于许可。
- 每次 commit 只会在 `PendingRecalculation` 为真时执行一次完整重算。
- 重算阶段显式带 `InProgress` 和 `CycleGuardTrips`，防止递归重算失控。

## Why The Routing Split Exists

- `addKeyword` 和 `modifyStat` 会改变卡牌的有效特征，属于“从基础真相推导有效值”的问题，所以接到持续效果层。
- `placeInfluence` 和 `dealDamage` 更像直接落在棋盘对象上的计数器，不需要进入持续重算，所以直接写入 `CardCounters`。
- 这样可以先把“导出态重算”和“直接状态写入”分开，避免还没实现 dependency engine 就把两类语义混在一起。

## Current Boundaries

- 这不是完整持续效果系统，还没有实现完整 dependency graph。
- `ResolveConflict(a, b)` 目前保留为空实现，未来用于更复杂冲突裁定。
- `DependencyKey` 已预留，但当前只做字段保留，不做依赖比较。
- 持续效果当前主要作用在 `CardState` 的导出字段：
  - `EffectiveKeywords`
  - `EffectiveStats`
  - `Permissions`
  - `Prohibitions`

## Test Coverage Added

- 数值修正生效。
- 禁止优先于许可。
- 本回合持续效果在后续回合失效。
- 同一 commit 内重复请求重算时只执行一次完整重算。
- 重算幂等。
- 显式防循环。
- 纯 DSL fixture `BQ022` 会注册真实持续效果。
- `dealDamage` / `placeInfluence` 走 counters，不注册持续效果。
