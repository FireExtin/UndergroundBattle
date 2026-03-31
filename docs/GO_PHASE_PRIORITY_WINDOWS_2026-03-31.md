# GO_PHASE_PRIORITY_WINDOWS_2026-03-31

Purpose: explains the finer-grained phase and priority window rules now enforced by the Go rules kernel, plus the new `dsl / script` stack-item execution split.

## Scope

- 为 `PhaseState` 增加更细的 `step` 概念。
- 为 `PriorityState` 增加显式 `windowKind`，区分 `action / response / closed`。
- 让共享 fixture 中的 `speed` 真正参与 legality。
- 让 stack 顶结算从“直接弹栈记账”变成“按 card source 进入 `dsl / script` 分流后再结算”。

## Window Model

当前最小窗口模型是：

- `action`
  说明：正常行动窗口。`slow` 卡只能在这里进入执行管线。
- `response`
  说明：已有 stack item 后的响应窗口。`fast` 卡可继续压栈，`reaction` 未来只应在此使用。
- `closed`
  说明：当前步骤已经结束，只允许进入下一阶段或后续系统动作。

## Speed Rule

- `slow`
  只能在 `action` window 中提交。
- `fast`
  可以在 `action` 或 `response` window 中提交。
- `reaction`
  目前规则核已预留 legality 入口，但要求必须在非空 stack 的 `response` window 中提交。

## Step Rule

- 当前步骤活跃时，`PhaseState.step = action`。
- 双方连续 pass 且 stack 为空时，步骤结束：
  - `PhaseState.step = ended`
  - `PriorityState.windowKind = closed`
- 阶段推进或新的可执行动作发生后，会重新打开行动步骤。

## Stack Resolution Change

这次之前：

- stack 顶结算只是把 pending operation 从 stack 弹出，标记为 resolved，然后写入 `BoardState.Resolved`。

这次之后：

- stack 顶先 `PopTop`。
- 然后按 operation source 的 `executionKind` 进入：
  - `dsl`
  - `script`
- 再把 resolved operation 写入 `BoardState.Resolved`。

这意味着 stack item 的来源已经不再是抽象占位 operation，而是明确的真实卡牌 contract 入口。

## Observable State

这次新增后，外部可以在序列化状态中直接观察到：

- `TurnState.Phase.Step`
- `TurnState.Priority.WindowKind`
- `Operation.Source.ExecutionKind`
- `Event.Step`
- `Event.PriorityWindow`

因此后续前端调试器、回放器和日志面板已经可以展示更具体的“为什么这张牌此刻合法/不合法”。

## Validation

- `go test ./...`
- `cd tools/fixture-tools && npm test`
- `cd tools/fixture-tools && npm run typecheck`
- `cd web && npm test`
- `cd web && npm run typecheck`
