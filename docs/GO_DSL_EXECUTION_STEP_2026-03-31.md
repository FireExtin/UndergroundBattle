# GO DSL Execution Step

本次任务把 Go 规则核中的 `dsl` 分支从“只做 contract-backed resolve”推进到“最小可执行状态变更”。

## 本次范围

- 只扩展 Go 规则核。
- 不引入完整规则引擎。
- 不引入数据库、websocket、UI。
- `scriptId != null` 的卡仍然不进入纯 DSL 执行。

## 本次新增能力

- `Action` / `Operation` 新增最小运行时 target 载荷：
  - `targetPlayerId`
  - `targetCardId`
- `CardOperationSource` 现在携带最小 `effects` 载荷，而不只是 `effectKinds`。
- `resolveDSLCardEffect` 现在会执行一小部分纯 DSL effect，并把结果写回 `GameState`。

## 当前已执行的 effect

- `drawCards`
  - 以确定性方式向控制者手牌区追加占位抽牌记录。
  - 该实现当前不依赖真实牌库，只保证 state mutation 和 replay 一致。
- `inspectHand`
  - 将目标玩家手牌标记为被当前行动者检视。
  - 现有 projection 机制会据此让检视者看到对方手牌。
- `exhaust`
  - 将目标卡牌标记为 `exhausted = true`。
  - 若该牌对观察者可见，则 projection 会带出 exhausted 状态。

## 明确未做

- 没有实现完整 target selection 系统。
- 没有把 target 缺失提升为严格 legality gate。
- 没有实现 `dealDamage`、`modifyStat`、`addKeyword`、`placeInfluence` 的最终语义。
- 没有实现脚本卡的真实脚本执行。

## 测试

- 新增 Go 原生 `testing` 用例覆盖：
  - 直接结算的纯 DSL 卡执行 `inspectHand + drawCards`
  - 入堆叠的纯 DSL 卡在结算后执行 `exhaust`
- 全量验证通过：
  - `go test ./...`
  - `cd tools/fixture-tools && npm test`
  - `cd tools/fixture-tools && npm run typecheck`
  - `cd web && npm test`
  - `cd web && npm run typecheck`
