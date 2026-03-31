# GO_CARD_FIXTURE_ENTRYPOINT_2026-03-31

Purpose: explains how `queue_operation` now enters the Go rules kernel through real shared CardLogic fixtures instead of placeholder labels.

## Scope

- 将 `shared/contracts/fixtures` 从 toy DSL 示例切换到第一批真实卡牌样例。
- 为 fixture 增加 `card.name`、`card.sourcePath`、`card.basicType`，并在 TypeScript 侧校验其必须回指 `organized_content` 中的真实卡牌记录。
- 更新 normalized contract 输出，保留 `cardName`、`sourcePath`、`basicType` 供后续工具链复用。
- 为 Go 增加 fixture catalog，并让 `queue_operation` 通过 `action.cardId` 解析真实卡牌入口。

## Current Batch

当前接入的真实卡牌样例是：

- `BQ005` 多重梦境迷宫
- `BQ010` 读心术
- `BQ013` 召现雷霆
- `BQ022` 合金指虎
- `BQ024` 脊椎强殖装甲
- `JZ74` 意外事故
- `WM088` 现场调查
- `WM090` 茶叶占卜法
- `XQ03` 力场束缚
- `XQ34` 灵感

这批样例同时覆盖了：

- 纯 DSL 与 `scriptId` 入口
- 直接结算与入 stack
- `player`、`region`、`character`、`asset`、`attachment` 等基础目标类型
- `none` 与 `permanent` 两类最小持续时间
- `exhaust`、`drawCards`、`inspectHand`、`dealDamage`、`addKeyword`、`modifyStat` 等当前已接入的基础 effect kinds

## Rules-Kernel Change

- `queue_operation` legality 现在要求 `action.cardId`。
- Go 会从共享 fixture catalog 中查找该 `cardId` 对应的 contract。
- 若 fixture 不存在，会返回结构化错误 `RULES_FAILED_CARD_LOGIC_MISSING`。
- 若 fixture catalog 不可读取，会返回结构化错误 `RULES_FAILED_CARD_LOGIC_UNAVAILABLE`。
- operation 会携带 `source` 元数据，包含 `cardName`、`logicId`、`sourcePath`、`basicType`、`targetKinds`、`requiresStack`、`scriptId`、`effectKinds`。
- 当 legality 需要识别“场上某张具体定义牌是否存在并生效”时，不能复用 `CardState.CardID`：
  - `CardState.CardID` 是场上实例 ID
  - `CardState.DefinitionID` 才是卡牌定义身份
- `XQ22` 的第一条禁止事务牌 slice 已经按 `CardState.DefinitionID == "XQ22"` 实现，不再依赖显示名称。

## Current Limit

- 这次改动接入的是“卡牌来源”和“脚本分流入口”，不是完整效果执行。
- 纯 DSL 卡当前只会以 contract-backed operation 的形式进入管线。
- `scriptId` 卡当前会被明确标记为 `requiresScript`，但脚本解释本身仍是后续阶段任务。
- `CardState.DefinitionID` 目前还是最小接入：
  - 已用于需要按卡牌定义识别场上实体的 legality（如 `XQ22`）
  - 但还没有贯穿到未来真正的“打出永久物并进入 table”生命周期建模中

## Validation

- `go test ./...`
- `cd tools/fixture-tools && npm run generate`
- `cd tools/fixture-tools && npm test`
- `cd tools/fixture-tools && npm run typecheck`
- `cd web && npm test`
- `cd web && npm run typecheck`

## Environment Note

- `tools/card-importer` 当前缺少本地已安装的 Node devDependencies，因此其 `npm test` / `npm run typecheck` 在 shell 层直接报 `tsc` / `vitest` 不存在。
- 这不是本次逻辑改动引入的断言回归，但仍需要在后续环境整理时补齐。
