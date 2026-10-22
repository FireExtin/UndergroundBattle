# ABI94 独立恢复回归补充

固定基线为 `1e9e679ad52798882108fcd99229e5bc00bcde18`。本补丁仅增加测试模块注册、独立测试和本文，不更改规则实现、卡牌定义、持久化 schema 或公开接口。使用普通独立克隆，没有 worktree、共享 Git 对象目录、提交、推送、CI 或部署。

实测结果：**5 passed / 0 failed / 1569 filtered out**。执行于 2026-10-22 12:11:32–12:12:05 UTC。测试自身耗时 0.69 秒；含独立缓存编译的总墙钟时间 32.809 秒。没有发现这五项契约中的新增失败。

## 独立范围与验收

作者已有三项恢复测试、十二项 Room 测试及四快照十六视图的 WASM 比较；本次未运行这些测试组。下表按新增测试函数计为五项，表中变体不是额外五项测试以外的独立通过数。布局复用仓库现有测试助手，断言和反例组合独立编写。

| 测试函数后缀 | 本次新增反例或边界 | 验收结果 |
| --- | --- | --- |
| `pending_domain_mutations_agree` | JC062 已付款、暂停于私密检视的合法续体，分别损坏选择座位、choice 玩家、检视卡身份、支付凭据、游标与已接受 guard 的关系，共五个变体 | 原状态两入口接受；五种损坏在 Game/Room 两入口均拒绝，并返回相同领域错误 |
| `private_pending_roundtrip_preserves_authority` | 实际使用反序列化后的 Game/Room，以交错顺序重复查询四席；错误席位、已失效选择 ID；正确确认；用新 revision 再确认同一旧选择 | 查询不改完整状态；仅选择者收到检视预览，其他人的暗藏身份保持遮蔽；两条恢复路径均只抓一张、不再次支付；三种非法确认均无状态、revision 或 journal 变化，Room 返回 `invalid_action` |
| `old_target_is_not_rebound_by_restore` | 单目标 JC062；旁边已有同印刷、同控制者的另一暗藏者；原目标通过实际 reveal 响应更换 UID；保存后分别用 Game pass 和 Room `PassResponse` 继续 | 两入口接受合法历史旧 UID；恢复后的结算整项取消，不重绑定同印刷牌、不退款、不抓牌，法术只进弃牌一次；两路径最终完整 Game 相同 |
| `domain_schema_and_versions_reject` | 私密续体缺少 Game schema、旧 schema、错用 Room schema，以及旧 engine/cardPool/rules，共六个变体；Room 外壳保持当前版本 | 两入口均拒绝；不借 Room 外壳静默升级。不同入口的版本错误文本无需完全相同 |
| `room_metadata_stays_an_envelope_boundary` | Game 完整不变；仅外壳缺 schema、旧 schema、旧 engine、revision 不匹配，以及私密选择期间伪造响应窗口，共五个变体 | Game 仍可恢复，Room 拒绝；共享领域校验不取消 Room 自身约束 |

核心差别是**继续使用恢复出的对象**：私密选择经 `RoomEnvelope::transition` 确认，目标失效经 `PassResponse` 继续结算；每次成功 Room 转换返回的状态再次通过恢复入口。它补充了仅检查 decode 成功或继续操作恢复前对象的证据。

响应窗口由既有 `jc029_tests::envelope` 助手构造为显式有效原生布局；局面也使用原生测试布局。不能把它描述为自然完整对局、真实浏览器刷新恢复或生产数据损坏。

## 复现与资源约束

使用 Rust/Cargo 1.90.0，在准备好依赖缓存的本地环境执行：

```sh
cargo test --offline --locked -j 1 -p hegemony-server --no-default-features --lib independent_recovery_tests -- --test-threads=1
```

这是 x86_64 Rust 核心库测试；`--no-default-features` 不启用 HTTP/数据库服务。实际执行限定 CPU 31、nice 19、I/O idle、一个 Cargo job、一个测试线程，每进程 8 GiB 虚拟地址上限；最大子进程 RSS 为 1,596,684 KiB（约 1.52 GiB），不是整棵进程树同时占用之和。编译、Cargo/ Zig 缓存与临时目录均位于本任务目录，依赖离线读取。没有同时运行性能基线或第二轮编译。

一次构建出现既有 `bounded_search_tests.rs:10` 未使用导入警告；无本次新增编译警告，没有修改它。完整运行记录随交接包提供：`recovery-evidence/attempt-1.log`、`attempt-1.json`、`source-check.json` 和 `manifest.json`，不把本机缓存、工具链或大构建目录放入补丁。

## 集成边界

补丁以该固定提交为基线，新增文件 `rust-game/src/independent_recovery_tests.rs`，并在 `lib.rs` 以 `#[cfg(all(test, not(feature = "society-fixtures")))]` 注册。依赖现有 `unit_support` 与 `jc029_tests::{envelope, pass_top}` 测试助手。其它分支的测试助手或模块布局不同，应由集成负责人对照后应用；本次未验证 batch4 当前分支或 Bot 分支。

本次不重跑 WASM/Worker/浏览器，不测试真实登录、HTTP 鉴权、持久化 receipt 幂等、暂停到期竞态、所有 schema 迁移或完整四席对局。旧选择重复确认特意使用当前 revision，以单独验证规则层的选择消费；它不能替代服务端重复命令 receipt 验收。未复制第三实现的代码或卡图。

建议按现有合流流程接纳这组测试，不增加新的恢复框架或第二套领域校验清单。全套浏览器验收和其它并行线仍按各自证据推进。
