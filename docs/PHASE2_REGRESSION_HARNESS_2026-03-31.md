# PHASE2_REGRESSION_HARNESS_2026-03-31

用途：记录在 `M0` baseline 冻结之后，继续把当前 sandbox 升级为可读、可判定、可安全重构的规则回归机。

## This Task Added

- stable replay diff helper：
  - [`server/pkg/rules/regression.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/regression.go)
  - `DiffScenarioResults`
  - `DiffScenarioExpectations`
- scenario invariant validator：
  - [`server/pkg/rules/regression.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/regression.go)
  - `ValidateScenarioResultInvariants`
- regression harness tests：
  - [`server/pkg/rules/regression_harness_test.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/regression_harness_test.go)

## Why It Was Worth Doing

`M0` baseline 只解决了“有固定快照可回归”。  
这轮补的是另外两件事：

- 当 replay 或 projection 跑偏时，要能看到**哪一层稳定面变了**
- 当 state / history / projection 之间出现内部漂移时，要能被统一 invariant 校验抓到

换句话说，这轮不是新增规则，而是把“回归失败时怎么解释失败”补齐。

## Invariant Scope

当前 invariant validator 会检查：

- `history.actions / operations / events / revisions` 长度与 `state.revision` 对齐
- revision 序号单调
- action id 唯一
- `stack` 里的 operation 必须 `pending`
- `resolved` 里的 operation 必须 `resolved`
- continuous registry 不允许在 scenario 结束后仍处于 `inProgress`
- projection 的 `gameId / revision / turn` 与 authoritative state 对齐
- projection 的 `stack / resolved / randomResults` 必须与 authoritative state 对齐
- hidden / visible card identity 必须符合投影规则，不允许泄露

## Diff Scope

`DiffScenarioResults` 比较的是稳定快照面，而不是整个 `GameState`。

这意味着它会优先指出：

- `revision`
- `actionLog`
- `turn / priority`
- `stack / resolved`
- `views.P1 / views.P2 / views.spectator`

而不会把不必要的内部细节直接喷成一整坨不可读的 full-state dump。

## Current State

到这一步，`Phase 1 + Phase 2` 的核心目标已经具备：

- 有唯一 `M0` 初始状态
- 有 committed baseline scenarios
- 有 replay consistency
- 有 projection leak coverage
- 有 legality reason-code coverage
- 有 targeted fuzz
- 有 invariant validator
- 有 stable diff helper

下一步才适合推进第一套完整可玩规则闭环，而不是继续扩 UI 或 transport。
