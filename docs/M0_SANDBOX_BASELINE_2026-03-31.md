# M0 Sandbox Baseline

用途：记录这次把 live sandbox 固化为 `M0` 基线，并升级为规则回归机的原因、结构和验证范围。

## What Changed

- 抽出了唯一 `M0` 初始状态来源：
  - [`server/pkg/rules/m0.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/m0.go)
- HTTP live sandbox 不再维护自己的隐式初始局面：
  - [`server/internal/api/session.go`](/Users/ddd/Downloads/UndergroundBattle/server/internal/api/session.go)
- 新增了 M0 scenario loader / runner / stable snapshot comparator：
  - [`server/pkg/rules/scenario.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/scenario.go)
- 新增了 committed golden scenarios：
  - [`server/pkg/rules/testdata/m0`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/testdata/m0)

## Why This Matters

- 现在的 live sandbox 已经不是脚手架，而是第一版可操作原型。
- 如果继续直接堆卡、堆按钮、堆 transport，而不先冻结基线，后续每次改规则都很难判断是“预期变化”还是“静默回归”。
- 这次把当前最小能力冻结成可重放、可快照、可比对的 M0 baseline，目的是让后续规则扩展可以安全重构。

## Regression Harness Scope

这轮新增的 Go 回归面覆盖了：

- 共享 M0 初始状态
- baseline scenario snapshots
- replay consistency
- projection hidden-info isolation
- legality reason-code coverage
- dispatch payload 不泄露 `FullState`
- targeted fuzz:
  - `CheckLegality`
  - `ProjectionEngine.Generate`
  - `RecalculateContinuousEffects`

对应测试入口：

- [`server/pkg/rules/m0_sandbox_test.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/m0_sandbox_test.go)
- [`server/internal/api/session_test.go`](/Users/ddd/Downloads/UndergroundBattle/server/internal/api/session_test.go)

## Boundaries

- 这轮没有新增 UI 功能。
- 没有改 HTTP protocol envelope。
- 没有扩展 transport。
- 没有新增完整规则能力，只冻结和验证当前能力。

## Next Use

后续 Phase 2/3 应该基于这套 M0 baseline 继续推进，而不是绕开它：

- 先补更系统的 replay / invariants / targeted fuzz 护城河
- 再进入第一套完整可玩规则闭环
- 再扩首发卡池与更真实的对局服务
