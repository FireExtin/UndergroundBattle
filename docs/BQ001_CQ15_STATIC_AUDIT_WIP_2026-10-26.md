# BQ001/CQ15 共享宿主静态核查 WIP（2026-10-26）

基线 `0c587fa1d80455a5775edb1ad843e4096f217133`。本片仅审查现有共享宿主、付费、选择、离场/翻暗、重复执行和持久化边界，不新增卡或准入，不另建兼容旧牌桌的分支。生产只改 `rust-game/src/import20_swords.rs` 和 `rust-game/src/death_continuation.rs`。BQ001/BQ009/BQ033 仍为 `supported:false`，目录、卡面规格及 ABI/pool 字符串未改。

## 确认的问题与最小修复

旧 `cq15_destroy_search` 在 `remove_dead` 后立即生成三区选项。对有隐秘的明置 XG25，`remove_dead` 暂存其死亡，把可选封印声明和 DeathFinish 放入效果队列；CQ15 自己先写入 pending，drive 会停在该选择。`resolution.rs` 对 CQ15DestroySearch 直接 return，无法经普通逐步续帧等待死亡。于是玩家先看到不含这张待定死亡牌的检索选项，后来拒绝/失败封印也不能把它补入本次检索。这是静态调用链确认的时序缺陷，尚无本片运行时红绿证据。

本片复用既有 DeathCaller/DeathFinish：冻结原目标操控者和印刷名称，新增有限 CQ15Search caller kind；等新发生的死亡去向完成后才生成三区选项。恢复不再执行消灭或付费。校验要求完整原 CQ15 授予帧、已付费用、已执行游标、与原消灭实例对应的 DeathWait、相同操控者/名称，以及未预置的 offered。保持公共重复 caller/栈帧互斥校验。未假设死亡批次永远只有一个 wait；双类型附属离场波仍由既有批次机制处理。

## 对抗核查结论

| 路径 | 静态结论与实际覆盖边界 |
| --- | --- |
| 选择/扣费 | cq15_begin 在真实授予来源、宿主资格和目标绑定后才支付；cq15_return_cost 将同一双剑退给其拥有者，收据与宿主横置一起留在原帧。apply_at_revision 事务副本使失败动作不提交状态。无新增确定缺陷。 |
| 来源离场/翻暗 | cq15_validated_frame 验证发动时的宿主快照和费用，不重新依赖当前宿主明置/在场；新等待续体同样复用此帧。已有通用 CQ15 来源离场测试源码；BQ001 付费后翻暗以及等待期间宿主离场仍需资源窗口内执行定向验证，不能称已实测。 |
| 选择合法性/隐私 | search_finish 要求原发动者、当前冻结三区列表、至多三张不重复且来自 offered 的选择；CQ15 分支直接返回，不重跑原 Op。BQ001 的专属手牌选择校验排除了独立验证的共享帧，未误把 CQ15 选择认作 BQ001 自身能力。 |
| 重复结算 | 已有 Room 费用收据去重测试源码及公共 DeathCaller 同实例互斥；新增反例要求旧 choice 重放失败、只保留一次双剑退回及一次封印。所有本片新增运行断言尚未执行。 |
| 刷新/恢复 | 新 caller 校验冻结的目标操控者/名称、空 offered、原目标死亡收据；恢复依赖全部去向完成。新增 Game 与 RoomEnvelope 篡改/重复 caller 反例。语法解析成功不证明加载器行为已通过。 |

## 新增的未运行测试

先追加下列反例测试源码，再改生产文件。它们使用显式 unit_support 布局：BQ001/资产/手牌由 fixture 提供，XG25 的隐秘 1 为前置预置，RoomEnvelope 会话节奏是既有单元夹具。测试内断言真实引擎动作与恢复，但本片未执行；不是合法构筑自然轨迹或公开 HTTP/浏览器证据。

- `bq001_shared_grant_sword_waits_for_xg25_grave_before_three_zone_search`：先提供死亡封印选择，拒绝后把真实新墓地实例纳入三区；费用一次、旧选择拒绝、队列耗尽。
- `bq001_shared_grant_sword_sealed_xg25_is_excluded_then_other_copy_can_be_sealed`：成功封印的死亡实例不进三区，另一张原手牌同名牌仍可选；Game/Room 恢复和队列耗尽。
- `bq001_shared_grant_sword_death_wait_rejects_search_rebinding_and_duplicate_callers`：两种加载器拒绝改绑操控者/名称、预塞 offered、偷换普通 Frame kind、重复 caller 五类篡改。

## 验证与下一窗口

本片仅运行三份 Rust 文件的 `rustfmt --emit=stdout --edition 2021` 语法解析（输出丢弃、均 exit 0）和 `git diff --check`（exit 0）；没有 Rust 类型检查、编译或运行测试。此前静态 20/0 仅属于 0c587fa1，不改签为本片结果。Native/WASM/Worker/UI/浏览器均 **NOT_RUN**，本片状态 **WIP_UNCOMPILED**。原 32 输入逐项哈希一致。

资源获批后先运行 `cargo test --lib bq001_shared_grant` 与 `cargo test --lib import20_swords_tests`；必要时补/执行付费后翻暗、临时控制/拥有者分离和死亡等待中来源离场的真实命令场景，再检查受影响的死亡续体恢复用例。此处只是计划，未启动任何编译或测试服务。

已只读回核远端现有审查分支 `review/game-serial-abi99-wip-20261024` 仍为 `42bcb148d1aeef92c345f0a1ce8b784d4dc90164`。用户本次明确允许 WIP 正常 push，即使触发 CI；这是一次性例外。采用正常快进推送及合理 skip 标记，不改 workflow、不查询被拒的 Actions 接口、不合 main 或发布。推送结果以独立读回收据为准，本文写入时尚未执行。

详细静态检查命令和文件哈希见 `docs/evidence/bq001-cq15-static-audit-20261026/static-validation.json`；修复前源码哈希另存，不重写历史结果。
