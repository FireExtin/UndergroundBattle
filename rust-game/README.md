# Rust 云端霸权规则核与房间服务

这是原 Go 工程旁的模块化单体：纯规则解释器、角色隐私投影和 SQLite 房间服务。服务只开放已逐卡实现的 21 张真实玩家牌及 5 种实际卡图核对过的地区，各地区两张。四套 50 张牌组是受限卡池的自组预组，不是官方预组。

原稿依据和卡图哈希见 [data/source-verification.json](data/source-verification.json)；接口和原 PDF 印刷页码见 [云端切片契约](../docs/HEGEMONY_CLOUD_SLICE_2026-10-02.md)。

## 启动与验证

```sh
cargo run -p hegemony-server
cargo test -p hegemony-server
```

默认监听 8090，SQLite 为当前目录的 rust-game.sqlite3，可通过 PORT、HEGEMONY_DB 设置。WEB_DIST 默认 web/dist；构建过的客户端由同一服务托管。开发客户端通过 Vite 的 /api 代理访问服务。

Rust 1.90.0。本工作区的工具链在 /workspace/.cloud-setup/cargo/bin；如未在 PATH 中，可使用：

```sh
export PATH=/workspace/.cloud-setup/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/.cloud-setup/rustup
export CARGO_HOME=/workspace/.cloud-setup/cargo
```

## 状态与时序

每局使用一个串行锁。已验证动作在状态副本上执行；拒绝动作不会修改状态、版本、身份序列或 PRNG。SQLite FULL 同步事务同时写入完整状态、版本化回放日志和 commandId 原始响应；提交成功后才更换内存状态、广播 SSE 和返回确认。

随机座位令牌只保存 SHA-256 摘要，HTTP 状态、动作和 SSE 都必须携带所属房间的 Bearer 令牌。邀请只能占大厅空座。SSE 始终返回请求者自己的 View；其他玩家只有手牌数量，队友暗藏者也不公开牌面。公开 waitingChoice 只含选择者、类型和标题，完整 pendingChoice 只给选择者。

固定版本、初始 seed 和 journal 可通过 Store::replay 完整重演。重开延续确定性 PRNG 序列，产生新的洗牌。SQLite 重启恢复包含堆叠、效果续体、当前快速窗口、团队让过记录和待选。离场、进入新区域类型与翻面使用新 instanceId；普通机动在场移动保留实体身份和横置状态。

资产付款自动使用本人的未横置资产，忠诚同时计算资产派系及魔法领域。建立资产和秘密派遣不入栈。现身支付完整印刷费用并保留横置状态；事务和角色能力进入可响应的后入先出堆叠。每个准备、抓牌、行动、对抗前后、赢区前和结束步骤都有快速行动窗口。

## 验证覆盖

规则测试涵盖 21 张牌和 5 种地区的印刷能力、临时图标、差值奖励、护卫和杀伤、屏障与护盾、同地区目标再验证、独有牺牲、墓地与牌库回收、机动顺序与响应、赢区牌底顺序和撤回、团队优先权和距离、弃牌和预测选择、隐藏投影、退出及队友续局，以及完整两人与四人游戏、重开和确定性回放。

服务集成测试涵盖令牌权限、commandId 去重、版本冲突、拒绝无变化、待选持久恢复、SSE 刷新和故障事务回滚。模拟 SQLite 提交路径错误时，服务不确认、不广播、不改变可读状态。
