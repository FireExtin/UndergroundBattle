# LIVE_SANDBOX_2026-03-31

Purpose: records the first end-to-end playable sandbox wiring between the Go rules core and the React debugger.

## What Exists Now

- Go `server/cmd/api` 不再只是日志 stub。
- 服务端现在会启动一个最小 HTTP sandbox，并持有单个内存内 session。
- session 启动时会生成 revision `0` 的初始 projection，并输出三条 `StatePatched`：
  - `P1`
  - `P2`
  - `spectator`
- Web 前端优先请求 live feed；如果请求失败，则退回 mock protocol 数据。

## HTTP Surface

- `GET /api/debugger/messages`
  - 返回当前 session 持有的 protocol envelope 历史数组
- `POST /api/debugger/actions`
  - 接收一个 `Action`
  - 返回这次提交产生的新 protocol envelope 数组
  - 合法动作会返回 `ActionAccepted + StatePatched*`
  - 非法动作会返回 `ActionRejected`

## Sandbox Cards And Actions

- 初始 sandbox 状态包含：
  - `P1` 手牌中的隐藏牌
  - `P2` 手牌中的隐藏牌
  - `P1` 与 `P2` 各一张场上角色
- 当前前端动作面板提供的预置动作包括：
  - `Pass Priority`
  - `Advance Phase`
  - `Reveal Own Secret`
  - `Inspect Own Secret`
  - `Cast 读心术 (BQ010)`
  - `Cast 多重梦境迷宫 (BQ005)`
  - `Equip 合金指虎 (BQ022)`

## Why This Is Still Minimal

- 没有 websocket
- 没有数据库
- 没有多房间 match service
- 没有完整动作作者工具
- 没有真实身份和权限隔离
- 前端 viewer switcher 仍然是调试器能力，不是正式对局客户端权限模型

## Local Run

### Dev

```bash
go run ./server/cmd/api
cd web
npm install
npm run dev
```

打开 `http://localhost:5173`。

### Built

```bash
cd web
npm install
npm run build
cd ..
go run ./server/cmd/api
```

打开 `http://localhost:8080`。
