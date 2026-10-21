# Rust规则核与Native房间服务

本crate同时包含确定性《霸权》规则核和可选的Native宿主。当前源码、目录数量、版本与测试阶段以[根README](../README.md)及[整改记录](../docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md)为准；旧v2.1的25张卡、固定临时路径和42项测试记录不代表当前版本。

## 实现入口

| 文件 | 职责 |
| --- | --- |
| [model.rs](src/model.rs) / [engine.rs](src/engine.rs) | Game、动作、选择、效果队列、优先权与视图 |
| [rules.rs](src/rules.rs) / [resolution.rs](src/resolution.rs) | 类型化能力、目标与费用校验、效果续体；结合专有规则模块实现准入卡 |
| [room.rs](src/room.rs) | RoomEnvelope、完整命令、服务端时钟、响应窗口、暂停与恢复 |
| [service.rs](src/service.rs) / [main.rs](src/main.rs) | Axum、鉴权、房间串行锁、SQLite事务、回执与HTTP |
| [catalog.rs](src/catalog.rs) / [society.rs](src/society.rs) | 运行目录与秘社准入；基础牌面见[data/cards.json](data/cards.json) |

规则核不解释任意中文规则文本或执行任意脚本。代码包含有限通用原语和明确的卡牌专有校验，不能宣称“生产解释器完全不比较牌号”。WASM宿主复用同一crate，并关闭`native`默认特性以排除HTTP/SQLite依赖。

## 本地启动

从仓库根目录执行，先按根README配置Rust1.90.0、Node与独立`UNDERGROUNDBATTLE_BUILD_ROOT`：

```sh
HEGEMONY_DB="$UNDERGROUNDBATTLE_BUILD_ROOT/native-dev.sqlite3" \
  node tools/build-storage.mjs -- cargo run --locked -p hegemony-server
node tools/build-storage.mjs -- cargo test --locked -p hegemony-server
```

第二条是独立测试命令，不在运行中的服务终端同时执行。默认监听`0.0.0.0:8090`，`PORT`可覆盖；`HEGEMONY_DB`默认历史文件名`rust-game-v2.8.sqlite3`，`WEB_DIST`默认`web/dist`。不要根据数据库文件名推断引擎版本。Native开发路径无需外部冻结WASM。

隐秘世界的持续开发约定是只保留最新内核，不兼容旧牌桌，以性能和体积优先。Native和Sites都只接受当前规则/卡池/引擎完整身份；旧身份明确失效并要求新开，旧数据保留，不迁移opaque状态，不增加旧核路由或运行保留包。新内核开发使用独立数据库；历史快照仅用于追溯。当前schema/状态完整性、u64精度、权限、费用和重复请求保护仍必须保留。

## 协议与状态

- 公共接口：`GET /api/health`、`GET /api/catalog`、`POST /api/rooms`、`POST /api/rooms/join`。
- 席位鉴权接口：`/api/rooms/{roomId}/state`、`catalog`、`commands`、`quote`、`events`。令牌使用Authorization Bearer，不放入URL。
- 房间外层schema3、Game schema2。命令含commandId、expectedVersion、完整会话动作；服务端决定时钟和实际规则结果。
- SQLite在事务中保存状态、逐版本日志与命令回执；拒绝和系统时钟推进按Room transition语义区分。重复命令不得成为重复费用或效果。
- 本人pendingChoice与全桌waitingChoice分离；返回值是席位View，不是完整Game。Native保留SSE接口，当前React主路径使用轮询。
- Native没有Sites创建/加入requestId回执协议；不能把命令幂等扩展解释为所有入桌请求都可无条件重试。

只读回放审计入口如下，必须提供相同内核版本可识别的数据库和房间ID：

```sh
node tools/build-storage.mjs -- cargo run --locked -p hegemony-server \
  --bin hegemony-audit -- /absolute/path/to/snapshot.sqlite3 ROOM_ID
```

可追加多个房间ID。该工具使用`Store::open_read_only`，匹配退出0，不匹配退出1；不修改数据库。具体公开命令回归在[tests/audit_regressions.rs](tests/audit_regressions.rs)。本地回放与服务测试不等于浏览器完整对局验收。
