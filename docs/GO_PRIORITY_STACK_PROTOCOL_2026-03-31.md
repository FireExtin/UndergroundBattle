# GO_PRIORITY_STACK_PROTOCOL_2026-03-31

Purpose: records how the Go rules kernel now handles priority windows, stack resolution, and structured machine-readable rejection results.

## What Changed

- 引入 `PriorityState`，显式记录当前行动权归属、连续 pass 计数、最近 pass 的玩家。
- 引入最小 `StackEngine`，统一处理入栈、栈顶结算、后进先出。
- `CheckLegality` 不再只依赖纯文本错误，而是返回 `LegalityResult`。
- 新增最小协议对象：`ActionAccepted`、`ActionRejected`、`StatePatched`。

## Priority Rule

- 默认是双人优先权窗口。
- 单次 `pass_priority` 会把优先权转给对方，并累积 `passCount`。
- 双方连续 pass 时：
  - 若 stack 非空，则立即结算 stack 顶，并把优先权重置给主动玩家。
  - 若 stack 为空，则标记当前步骤结束，并把优先权重置给主动玩家。

## Stack Rule

- `queue_operation` 会把 stack item 入栈。
- stack 结算严格按后进先出。
- 当前最小规则核保留 `resolve_top_stack` 这个显式动作，便于测试和调试；后续可逐步收敛为更自动的系统驱动结算。

## Structured Error Rule

- 规则机内部统一产出 `ReasonCode`。
- `LegalityResult` 包含：
  - `OK`
  - `ReasonCode`
  - `MessageKey`
  - `Hook`
  - `Context`
- 当前已落地的 code 前缀包括：
  - `LEGALITY_FAILED_*`
  - `TARGET_FAILED_*`
  - `COST_FAILED_*`
  - `STACK_FAILED_*`
  - `RULES_FAILED_*`

## Why Keep Protocol Objects This Early

- 规则机不是只给 Go 测试自己看，后续还要给前端、调试器和回放工具消费。
- 现在先把 `ActionAccepted / ActionRejected / StatePatched` 这三类最小对象固定下来，可以避免未来出现“引擎内部状态可用，但外部协议临时拼字段”的问题。

## Scope Boundary

- 还没有实现复杂卡牌目标选择、费用支付、触发式能力和替代式效果。
- 这一步只是在最小规则核上补齐响应窗口、LIFO 堆叠和结构化错误协议。
