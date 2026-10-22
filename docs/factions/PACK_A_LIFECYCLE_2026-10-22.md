# A 公共对抗生命周期续接（WIP）

起点为已整合的 `47256d59dea46750ea671b1f1257b657bb42a546`。本普通独立克隆原
`wip/batch50-pack-a-hidden-contest-20261022` 分支仍停在 `c5104f19…`，工作树原本干净。
新旧历史共同祖先为 `adf0a1c…`：共同基线挑拣了 A 五条和 B 三条历史，并保留主作者
退役 UID 修复。此次新建 `wip/pack-a-contest-lifecycle-20261022` 续接，不重新回放旧包。

只读了共同基线的 `PACK_AB_COMPATIBILITY_2026-10-22.md` 和 B
`2dd1f6d1fc2bdbc324e6575a4f8d68ac5a9ef093` 交付的 `CONTRACT_GAPS.md`、
`verification.json` 与接口探针。B 自有模块、原克隆、主作者树、XQ20/markers/plans
均未修改。这里不实现 B 对抗成本、收据或饮血/检索能力。

## B 可继续消费的公共契约

原五个挂钩、`mystic_contest_start(ResolutionFrame,u32)`、
`mystic_add_bonus(ResolutionFrame,u32)` 签名不变。五个挂钩在此基线上仍为空；
有限 B starter 路由、BQ038 收据及 BQ027 后置帧由 B/集成方接入。父端本轮后续交接了
B `37986a85…` 的实际实现；下文另记独立组合验证，不把它重复收入 A 的补丁。

```rust
// 活动最内层在前，其后依次为挂起的父层。只读，不执行/重放挂钩。
pub(crate) fn mystic_states(&self) -> impl Iterator<Item = &MysticContest>;

// MysticContest 新增兼容可选字段，旧非嵌套存档可以省略。
#[serde(default, skip_serializing_if = "Option::is_none")]
pub parent: Option<Box<MysticContest>>;
```

`self.mystic_contest` 总是当前活动层。B 的开始加值只能交给该层原 contest ID；
不能在子层活动时替父层发放。恢复验证如果需要查找已经挂起的 B 后置帧，遍历
`mystic_states()` 并匹配其原 contest/frame ID，不能只看当前顶层、按当前场面重算，
也不能把任一父层的收据当作当前层。A 的 WM061 后置帧现在同样在相应层验真。

每一层仍是原已付费、已接纳的 `ResolutionFrame`，从 `BeforeSelection`、`After` 或
下述 `Finishing` 被新的合法响应对抗挂起；秘密提交及奖励待选不能挂起。所有层使用原 effects 队列、
响应堆叠、Pending、RNG 及克隆提交。开始层先移顶牌和选牌前，等待该层新增响应以及
响应产生的声明/选择链。子层完成全部后置及相关触发后恢复父层原游标，不重付费、
不重跑开始挂钩、不重复洗牌或领取奖励。父层 After 依据自己的历史 contest ID 取结果，
不再错误要求它一定是最后一条历史。

嵌套层另有一个 `Effect::MysticFinish { contest_id }` 结束边界，一直位于该层普通
触发和更旧父层工作之间。过程本体及原帧完成后进入
`MysticPhase::Finishing { outcome: Box<ContestOutcome> }`，在这一阶段等本层产生的
获胜/进场触发及其全部响应链结束，才消费结束边界、恢复父层。旧的非嵌套存档没有
此额外边界，仍能读取。B 不应自行产生/删除这个边界，也不应把 Finishing 当作新开始。

子层响应可能淘汰父层原参与者。挂起的 BeforeSelection 保留开始快照及原收据，恢复到
实际移牌时才过滤该席位及其加值；B 收据验证应依冻结见证确认历史，而不要求该席位
或原观察者目前仍存活、仍在场。

开始挂钩仍正常追加声明，后置挂钩仍在当前 After 游标已推进、自己的续体已入队后
执行。引擎在单次命令和单个效果执行边界保留新追加的因果效果，使它们排在对应
续体之前；既有更下层队列保持在后。实现用一个仅内存的 `MysticQueueFence` 定位新
追加段，此枚举变体 `serde(skip)`，禁止保存/读入/执行，不分配实体 UID，不是新操作
或新响应内核。未处于隐秘对抗时走原路径。

胜利观察仍由核心一次合并排序。响应新产生的排序选择、可选触发和私有待选必须完成
后才到外层秘密选牌；B 不要额外自行调整全局队列或第二次登记同组观察。

## 取消和恢复

终局沿用 `finish`，一次清除整个活动/父层链、Pending、effects 和 stack，保留已完成
本回合结果。不再因残留 After 状态让合法终局回滚。非终局淘汰保留已经完成的历史
参与者；待开始的层在实际移牌前过滤已淘汰者。后置游标仍顺序消费，已经淘汰的展示者
不再运行个人后置。重开局仍清空历史。

两恢复入口共同要求所有层有独立原帧/对抗 ID、正确先后关系、完整程序和参与者，
父层只允许三个可挂起阶段。所有层所需续体/嵌套结束边界必须按内层到外层各出现一次；不能删层、
重复/重排续体，或把当前新响应伪装成开始时原已存在的下层堆叠。完整 A 验证器、
退役 UID 邻接规则、原卡完整程序比较没有放宽。

## 验证和计数

`mystic_lifecycle_tests.rs` 将 B 的三个失败探针改成修复断言。在共同基线先得到
3 项预期失败：新声明落在秘密选牌后、嵌套结束 pass 被拒、后置终局回滚；修后 3 项
均通过。后续扩展到多层 LIFO、嵌套胜利触发排序及弃牌、真实 BQ036 旧检索的私有
响应选择、下层队列、拒绝恢复、空库回滚、两种淘汰与重开，以及 Room 命令和日志回放。

源码检查点为 `04b76021383719a3e95942874a444feb540aec1f`，前置公共层检查点为
`82e0eed8836cf00a770d9b17ee97ebccb0152460`。实际执行：

- 生命周期 `cargo test --offline --lib --no-default-features mystic_lifecycle -- --test-threads=1`：15/15。
- 同一已编译测试二进制过滤 `pack_a batch4_vampires rumor batch3_entry_order win_flow recovery_validation_tests`：110/110，其中 A33、共同基线 B31；不包含 B 未整合的后续三个辅助测试。
- 直接 Node 执行 `tools/cards/batch4-source.test.mjs`：3/3。
- `git diff --check`：通过。

这些测试使用明确布局：开始挂钩尚未接通时从真实已付费 WM022 帧暂停或回置到挂钩
边界；终局探针用普通 Draw 表示后置引出的消耗牌库效果。不冒称实际 BQ038/BQ027
条款已经端到端运行。最终精确执行数、输入指纹及失败日志见交付的 `verification.json`。

额外一次复核以显式父层旧 Draw 布局复现了初版剩余问题：子层获胜触发之前旧父层
工作先执行。修前1失败已保留，新增的嵌套结束边界修复该顺序；另测 Finishing 时真实
付费响应再触发第三层、正确退回第二层获胜弃牌、最后才到第一层秘密选择。

## 固定 B379 真实响应链的组合复核

独立普通验证克隆 `UndergroundBattle-packAB-check` 从 B
`37986a8556c8dd3534dbd8a0afe73e864de49972` 出发，挑拣 A 两个源码提交，无冲突。
B 实现文件保持原样；验证克隆只追加了一条测试提交，供集成方独立审查。组合代码点
为 `a9fd2d9283ac5fb4a13642df24e8b566309c7eff`，测试交付点为
`2aa87568e09d1fa8780b20861056baac1260ef69`。实际同一 native-core 配置运行
`pack_a batch4_vampire mystic_lifecycle rumor batch3_entry_order win_flow recovery_validation_tests`
得到 **146通过、0失败、1忽略**；其中 A33、B52、生命周期15，其余46为相邻过滤。
另外直接 Node 来源3/3。此组合结果和上文独立 A 回归有重叠，不能相加冒充唯一总量。

B 原被忽略用例的 `pass_top` 按堆叠数量减少停止。A 修复使 WM022 现身帧立刻被其
强制触发帧替换，数量不减少，该辅助函数于是继续结算新帧。将测试停止条件改为
**原响应帧 ID 已离栈**，移除该用例的 ignored 后，实际 BQ032→BQ038→付费 WM022
声明能正确入栈，单跑通过。A/B 组合的费用、挂钩及邻近回归另列验证记录。

继续真实 let-pass 结算内层时仍有一个 **B 收据接线依赖**：
`batch4_vampire_contest.rs::b_contest_start_receipt`（B379第299行）只取
`self.mystic_contest.as_ref()`。内层开始时，外层尚在堆叠的 BQ038 帧仍要恢复验真，
它的原 contest ID 属于父层，因而被该判断拒绝。额外测试明确断言应成功，单跑实际
失败，错误为“掠食时刻只能观察原对抗开始阶段”，并确认整条命令原样回滚。

B/集成方需按收据原 `o.contest_id` 从 `mystic_states()` 选取对应层，再保留现有
BeforeSelection/turn/participants/来源身份检查；不是把收据重绑当前层。实际发放加值
仍只能经 `mystic_add_bonus` 落在当前活动层，不能借恢复可见性给挂起父层越序发放。
A 没有改写这段 B 费用/收据程序。验证克隆保留这个新测试为明确 ignored，并保留
实际 `--ignored` 失败日志；不能将组合结果说成全部嵌套已验收。旧声明阻塞与此收据
依赖是两个不同检查点，前者已被实际链验证关闭，后者待 B 接线。

完整数仍 **22/50 + 10 WIP**；本轮不增加完整卡数、构筑/公开/WASM准入。未运行默认
native 服务、全套、WASM、浏览器、性能测试或外部独立复审。未推送、CI、main 合并或
部署。所有测试单编译任务/单测试线程、两 CPU 亲和、nice15、离线独立构建缓存。

## 保留的裁定边界

手牌和牌库都为空时仍没有读到明确原文确定展示值/退出规则。此轮没有添加假牌、费用0
或推测淘汰；开始及开始响应后都明确拒绝，原命令原子回滚。该裁定与上述 B 挂起层
收据依赖、B 所报非吸血鬼饮血区域迁移裁定仍未完成，不能因此把这十张 WIP 改为完整卡。
