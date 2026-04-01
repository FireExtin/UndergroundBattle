# WEB_DEBUGGER_2026-03-31

Purpose: explains the first minimal web debugger and match shell for the UndergroundBattle project.

## Scope

- 这一轮只实现：
  - 对局基础页面
  - stack 面板
  - action log 面板
  - current revision / phase / step / active player / priority 显示
  - public score / winner 显示
  - legality failure 显示
  - mock 的 per-player view 切换
  - 基于 Go HTTP sandbox 的 live feed
  - 预置动作提交按钮
  - 最小 custom JSON 动作提交入口
  - live 不可用时退回 mock fallback
- 不实现：
  - websocket
  - 前端裁判逻辑
  - 复杂状态管理库

## Data Model

- 协议 envelope 严格遵守 `shared/protocol/messages.schema.json`：
  - `version`
  - `kind`
  - `messageId`
  - `name`
  - `revision?`
  - `payload`
- `payload` 不重新设计中间 DTO，直接复用 Go 当前真实协议结构：
  - `ActionAccepted`
  - `ActionRejected`
  - `StatePatched`
- live HTTP 接口当前最小化为：
  - `GET /api/debugger/messages`
  - `POST /api/debugger/actions`

## UI Decisions

- 当前 viewer 切换固定为：`P1 / P2 / spectator`。
- revision / phase / step / active player / priority / score / winner 取当前 viewer 对应的最新 `StatePatched`。
- `stack` 面板固定按“栈顶在上”展示，因此 UI 会反转底层 push-at-end 的数组。
- `action log` 只显示 accepted/rejected，不把 `StatePatched` 也混进日志行里。
- `legality failure` 显示结构化字段，不退化成单行字符串。
- 动作面板提供少量预置动作，并提供最小 JSON 编辑器用于提交自定义 `Action`；仍不引入完整作者工具。
- 当当前 patch 已有 `winner` 时，live 动作面板会禁用提交按钮，但仍允许 `Reload Feed`。
- Web 仍然不是语义权威；按钮点击只是提交 `Action`，最终是否合法仍由 Go 决定。

## Hidden Information

- 当前前端不自行推断隐藏信息。
- viewer 切换完全依赖 `StatePatched` 中的 `playerView` / `spectatorView`。
- 因此同一 revision 下，`P1`、`P2`、`spectator` 可以看到不同卡面信息，这与 Go projection 设计一致。

## Runtime Modes

- `live`：连接 Go sandbox，展示真实 revision / stack / priority / projection 更新。
- `fallback`：如果 `/api/debugger/messages` 不可达，则自动退回仓库内 mock protocol 数据。
- fallback 只用于调试 UI，不意味着 TS 取得了裁判能力。
