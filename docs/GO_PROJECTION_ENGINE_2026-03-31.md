# GO_PROJECTION_ENGINE_2026-03-31

Purpose: explains how the Go rules kernel now derives client-safe views from authoritative FullState without leaking hidden information.

## Authority Boundary

- `FullState` 仍然是 Go 服务端内部的唯一真相源。
- 客户端不得直接消费 `FullState`。
- 客户端只能拿到投影后的 `PlayerViewState` 或 `SpectatorViewState`。

## View Split

- 同一 revision 下，不同玩家可以得到不同的 `PlayerViewState`。
- 玩家自己的隐藏牌可以在自己的 view 中可见。
- 未公开信息在对手 view 和 spectator view 中必须保持隐藏。
- 一旦牌被翻开，下一次 commit 之后，双方和 spectator 都能看到公开信息。

## Projection Timing Rule

- legality 检查阶段不得生成 projection。
- 只有在原子 state commit 完成、revision 已写入之后，才允许生成 projection。
- 这样可以保证客户端永远看不到中间态，也不会把未提交状态误当作事实。

## Protocol Rule

- `StatePatched` 现在是单个 audience 的 patch，而不是“所有玩家视图打包一起发”。
- 这样可以避免协议层一次性携带多玩家视角，造成跨视角泄露。
- 服务端内部仍可保留完整 projection bundle，用于按连接分别分发。

## Scope Boundary

- 当前只实现最小隐藏信息隔离样例：自己可见、公开可见、单方检视可见、spectator 默认隐藏。
- 还没有进入复杂的连锁公开、历史可见性、检视时效、日志脱敏和多人座位映射规则。
