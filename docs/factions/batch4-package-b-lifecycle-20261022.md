# B 收据嵌套接线续交

本轮从清洁 B `37986a8556c8dd3534dbd8a0afe73e864de49972` 建立普通独立分支
`agent/package-b-lifecycle-20261022`，旧 B 分支保留。与 A
`cd450d89509b9a760eba53dcb06da30486021c64` 的共同祖先为
`47256d59dea46750ea671b1f1257b657bb42a546`。先逐项验证 A 交付31个文件的
SHA-256，再无冲突挑拣 A 三条提交（没有只取初版生命周期修复）：

|A 原提交|本地挑拣提交|
|---|---|
|82e0eed8836cf00a770d9b17ee97ebccb0152460|7d3a8498（完整 SHA 见交付清单）|
|04b76021383719a3e95942874a444feb540aec1f|75254840（完整 SHA 见交付清单）|
|cd450d89509b9a760eba53dcb06da30486021c64|3f623ccaf44e0ddffef727f3e0dcc7f65cabfc58|

A/B 整合点 `3f623cca…` 是本轮 B-only 补丁的基线。父端已有 A 时，应只取 B-only
补丁；另提供从 B379 开始的含 A 顺序提交，避免重复导入 A。

## 改动与边界

BQ038 未结算收据现在按原 `contest_id` 在 `mystic_states()` 中找对应层，保留
BeforeSelection、回合、参与者及来源身份原校验，不把外层收据改绑当前子层。
真正结算前还须确认该 ID 属于当前活动层；不满足时连新的结算事件 ID 都不分配。
B 的一次性观察检查遍历所有层已保留的加值，拒绝“父层已记已结算收据、原响应帧
却仍等待”的重复状态。

未改写 A 共享协议、UID 认证、费用内核、克隆事务、恢复入口、RNG、XQ20 或
markers/plans。仅 B 两个生产模块和 B 测试发生后续改动。原本被忽略的真实声明
入栈回归与 A 补充的父层收据回归都已经取消 ignored，正常参与执行。

## 可复现验证

命令均经本克隆已有 `tools/build-storage.mjs` 包装器，离线、单 Cargo 构建任务、
单测试线程，默认 native 配置（与 A 交付的 no-default-features 结果分别记录）。

```text
cargo test --offline --manifest-path rust-game/Cargo.toml --lib -- pack_a batch4_vampire mystic_lifecycle rumor batch3_entry_order win_flow recovery_validation_tests --test-threads=1
cargo test --offline --manifest-path rust-game/Cargo.toml --lib a_b_nested -- --test-threads=1
node --test --test-isolation=none tools/cards/batch4-source.test.mjs
```

- 本地复现修前原回归1失败，错误为“掠食时刻只能观察原对抗开始阶段”，并确认
  被拒命令保持完整状态不变。日志未丢弃。
- 修后原回归1通过。扩展真实嵌套4项通过，覆盖两层非零加值、原帧恢复、不重付2/
  不重复弃自身、放弃子层加值、越层发放拒绝、原ID/回合/来源篡改拒绝。
- 组合150通过、0失败、0忽略：B56、A33、生命周期15、相邻46。
- 组合运行后仅在同一个嵌套恢复测试中追加父层重复收据断言；再跑嵌套4通过。
  生产代码没有改变；两组重叠，不相加成154项。
- 来源门禁3通过；`git diff --check` 通过。

B 实际链是明示原生布局之后真实 BQ032付2弃自身→BQ038开始触发→优先权轮转→
付费WM022现身→子层/父层各选牌、各结算一次。每个普通命令做 Game 恢复，关键
边界做 Room 恢复与四席视图比较。测试按原响应帧 ID 停止，避免把被强制声明替换
后的新堆叠帧也意外消费。A 生命周期中使用的人工 BeforeSelection/After 边界夹具
仍保持原标注，不冒称全部都为自然 B 卡组合。

此前已知声明延后及 B 父层查层阻塞在此组合关闭；以前交付中的5个“复现故障”探针
和 ignored 状态是历史证据，不是本轮验收结论。完整卡计数仍不增加：空手空库展示
规则、BQ038非吸血鬼饮血区域迁移仍待原文裁定；本轮不填0、不新造规则。

仅本地提交，原作者和 A 克隆保持只读，未推送或联网复核上游。父端提供的进度远端
共同点仍记录为47256d59；本地 A 输入是cd450，不把它写成远端SHA。未CI、合main、
部署、强推或reset。未跑全套、WASM、浏览器、性能基线或新增SQLite回执测试。
交付中 A 的压缩测试日志属于其已授权必要验证文档，不含数据库、凭据或构建缓存。

本轮生产源码固定于 `5d599798d6fa056463f9c3526efba58c9402f323`；验证输入与日志散列
见 [`verification.json`](../evidence/batch4-package-b-lifecycle-20261022/verification.json)。
续报“容量失败”后再次核验工作树清洁、没有本任务构建/测试进程，全部已跑日志可读，
没有重复运行测试。只读发现父端 HEAD 已到 `ef9415e9e48de8b1a134f3e30e3b3f1e0e50b7fa`，
该提交是父端 XQ20 规则接线；本分支未覆盖或导入该保留文件，完整补丁另做临时index
兼容性检查，不冒称已测试父端最新混合树。
