# GO_PER_CLIENT_DISPATCH_2026-03-31

Purpose: explains how projection output is now turned into a transport-ready per-client dispatch batch.

## Why This Exists

- `ProjectionBundle` 解决的是“同一 revision 下，不同 audience 看到什么”。
- transport 层还需要解决“这些不同视角该分别发给谁”。
- 如果没有正式 dispatch 协议，服务端很容易退化成临时拼 JSON，再次引入跨视角泄露风险。

## Minimal Dispatch Model

- `DispatchTarget`
  - 标记 audience 是 `player` 还是 `spectator`
  - 对玩家还带 `AudienceID`
- `ClientDispatch`
  - 一条发给单个 audience 的 envelope
  - payload 目前只允许三类：`ActionAccepted`、`ActionRejected`、`StatePatched`
- `DispatchBatch`
  - 一次 action 处理后产出的完整 per-client 输出

## Commit Dispatch Rule

一次成功提交后：

- 只有 action 提交者会收到 `ActionAccepted`
- 每个玩家都会收到自己那份 `StatePatched`
- spectator 会收到 public-only 的 `StatePatched`

这里的关键不是“有一个 patch”，而是“每个 audience 都有自己那一份 patch”。

## Hidden Information Rule

- 服务端内部可以持有完整 `ProjectionBundle`
- 但 protocol / transport 层不得把所有玩家视角混在同一个 `StatePatched` 里下发
- `StatePatched` 必须已经绑定 audience

这样做的目的是把“按谁可见”从调用约定升级为结构约束。

## Scope Boundary

- 目前只是规则核内的最小 per-client dispatch 批次
- 还没有 websocket session、订阅管理、断线重发和消息持久化
- 但协议形状已经可以直接作为后续 transport 层的输入
