# CARD_DSL

Purpose: defines the minimal CardLogic DSL contract boundary shared by TypeScript tooling and the Go authority.

## Authority Boundary

- TypeScript 负责 schema、fixture、作者工具和前置校验。
- TypeScript 不是语义权威。
- Go 是最终解释者和最终语义权威。
- 同一份 fixture 必须能被 TypeScript 和 Go 共同读取与验证。

## Minimal CardLogic DSL

当前最小 DSL 字段包括：

- `id`
- `schemaVersion`
- `speed`
- `targetKinds`
- `requiresStack`
- `durationKind`
- `scriptId`
- `effects`

当前 `effects` 只覆盖最小基础效果种类，用于契约测试和作者工具早期校验，不等于完整规则引擎。

## Fixture Gate

- 每张新卡进入主卡池前，必须先有 contract fixture。
- fixture 是主卡池准入门槛，不是附属文档。
- fixture 不通过，则该 DSL 样例不可用。
- TypeScript 侧 fixture 校验通过，不代表最终语义成立；仍需 Go 侧解析通过。
- fixture 必须携带最小真实卡牌锚点：`card.name` 和 `card.sourcePath`。
- `card.sourcePath` 必须回指 `organized_content` 下的真实资源文件，防止 DSL 样例脱离上游卡牌资产。

## Real Card Entry Point

- `queue_operation` 不再依赖测试标签作为主要来源，而是通过 `action.cardId` 读取共享 fixture。
- Go 规则核会把 fixture 解析结果挂到 operation source 上，包括 `logicId`、`targetKinds`、`requiresStack`、`scriptId`、`sourcePath` 等字段。
- Go 规则核还会把最小 `effects` 载荷复制进 operation source，保证进入执行管线后的卡牌来源不是只剩 effect kind 标签。
- operation source 还会显式标记 `executionKind = dsl | script`，用于统一堆叠入口后的执行分流。
- 因此 stack item 现在代表真实卡牌入口，而不是匿名占位操作。
- 当前这层只负责把卡牌来源接入统一执行管线，不等于已经实现完整卡牌语义。

## Minimal Executable Effects

- 当前 Go 规则核已经开始执行一小部分纯 DSL effect，而不再只是 contract-backed resolve。
- 当前真正落地到 `GameState` 的 effect 包括：
  - `drawCards`
  - `inspectHand`
  - `exhaust`
  - `addKeyword`
  - `modifyStat`
  - `placeInfluence`
  - `dealDamage`
- 这些 effect 的目标由运行时 action 提供最小 target 载荷：
  - `targetPlayerId`
  - `targetCardId`
- 该运行时 target 目前是 Go 规则核内部执行输入的一部分，不是作者侧 DSL schema 字段。
- `addKeyword` 和 `modifyStat` 现在会注册到 Go 侧最小持续效果层，并在 commit 内统一重算一次有效值。
- `placeInfluence` 和 `dealDamage` 现在会直接写入 board counters，而不是伪装成持续效果。
- `dealDamage` 的结果已经开始接入更正式的角色数值语义：commit 后会结合 `EffectiveStats.Defense` 做最小致命伤害判定。
- 持续效果产生的 `grantPermission` / `prohibitPermission` 已开始接入真实 legality hook；当前最小接入点是 `inspect_card`。
- 这仍然不是完整 dependency engine；当前只实现最小可回放、可测试、可扩展的执行骨架。

## Scripted Cards

- 当 `scriptId` 为 `null` 时，该卡可以被视为纯 DSL 候选。
- 当 `scriptId` 非 `null` 时，Go 必须标记 `requiresScript: true`。
- 带 `scriptId` 的卡不得被当作纯 DSL 可执行卡直接处理。
- 带 `scriptId` 的卡仍然可以进入统一 operation / stack / revision / projection 管线，但完整效果要留给后续 Go 脚本解释入口。

## Version Rule

- fixture 的 `schemaVersion` 必须与 `shared/schemas/card.schema.json` 声明的当前版本一致。
- DSL schema 变更时，必须同步更新 fixture、双端测试和对应文档。
