# GO_MINIMAL_RULES_KERNEL_2026-03-31

Purpose: explains the shape and constraints of the first authoritative Go rules kernel skeleton.

## Why This Kernel Exists

- Go 是唯一规则语义权威，因此最小规则核必须先于 UI 存在。
- 这一步不是完整规则引擎，而是把“动作提交 -> 裁决 -> 提交 -> 回放”这条主干先固定下来。
- 只要这条主干稳定，后续 legality、堆叠、持续效果、脚本卡和投影视图都能在同一管线上继续长。

## Minimal Modules

- `GameState`：最小可序列化总状态。
- `TurnState / PhaseState`：当前回合编号、主动玩家、优先权玩家、当前阶段。
- `BoardState`：最小堆叠、已结算操作、随机结果记录。
- `Action`：玩家提交的意图。
- `Operation`：规则机内部统一执行单元。
- `Event`：一次提交产生的状态变化记录。
- `Revision`：每次 commit 后单调递增的版本号。
- `HistoryState`：action / operation / event / revision 日志。
- `RNGState`：种子化随机源，保证 replay 可重演。

## Pipeline

当前最小执行管线固定为：

1. `SubmitAction`
2. `CheckLegality`
3. `BuildOperation`
4. `Put On Stack / Resolve Directly`
5. `Build Event`
6. `Commit State`
7. `Generate Revision`

这个顺序刻意保持显式，避免未来为了“先跑通”而绕开统一裁决管线直接改状态。

## Replay Contract

- 所有成功提交都会记录 action log。
- 所有成功提交都会生成 revision。
- replay 依赖：初始 `GameState` + action log + 相同 RNG seed。
- 只要这三者一致，最小规则核就必须产出同一最终状态。

## Scope Boundary

- 还没有实现完整卡牌效果。
- 还没有实现数据库、websocket、UI。
- 还没有实现复杂 replacement / prevention / trigger。
- 当前只实现最小直接结算、入堆叠、栈顶结算、阶段推进和种子随机，以便把规则内核骨架和测试基线先固定下来。
