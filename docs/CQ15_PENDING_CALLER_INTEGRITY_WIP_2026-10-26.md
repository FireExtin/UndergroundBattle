# CQ15 pending/DeathCaller 快照排他校验 WIP（2026-10-26）

基线 `fffab715d01dcae772e3acde0d6fc864fad571f3`。父端独立静态 review 指出 P2 持久快照完整性缺口，本片已按源码核实。范围仅为伪造/损坏持久快照，不是已证实的正常点击重复结算；没有运行时复现或红绿测试结果。

## 静态问题与最小修复

合法 XG25 待选时复制 CQ15 的 DeathCaller，正常拒绝封印进入 CQ15 search 后，把复制 caller 的去向补成实际 Grave 实例并塞回 effects。旧逻辑能分别接受合法 search pending 与这个具有完整费用/死亡收据的 caller，原公共排他条件只检查其他 caller、stack、Effect::Frame，没有检查 pending Frame。静态推演是原选择完成后残留 caller 可再次生成 search。

仅在 `rust-game/src/death_continuation.rs` 现有公共 frame_id 排他条件中加入同 frame_id 的 `pending ChoiceResolution::Frame` 冲突检查。`Game::from_persisted` 经 `validate_restored_domain` 调用该校验，Room 加载也校验其 game。保留原等待、完整去向收据、CQ15 费用/原目标操控者/同名绑定；不同 frame_id 的死亡响应待选不受本条件影响。不改 CQ15 执行程序、卡文、目录、准入或 ABI。

## 先写的未运行反例

`rust-game/src/bq001_shared_grant_tests.rs` 新增：
`bq001_shared_grant_sword_search_rejects_completed_caller_replay_snapshot`。

该测试源码先于生产修复写入，断言包括：

- 原合法 waiting 状态 Game/Room 恢复。
- 原合法 search pending 与补全真实墓地收据的 caller 单体分别合法，定位为并存冲突。
- 显式捕获 completed-only 边界（去除 pending 副本，仅保留完成的 caller）仍能 Game/Room 恢复，drive 一次后选择清空，双剑退回恰好一次、宿主保持已支付横置。
- 将同一 caller 加入合法 search 快照，Game 与 Room 两加载入口均须以公共排他错误拒绝。

沿用显式 unit_support 牌/资产/隐秘预置和 Room 单元会话夹具；completed-only 也是明确的内部边界捕获，不是公开开局、自然牌局轨迹或浏览器证据。上述断言全部 **NOT_RUN**。

## 验证与保存边界

只运行新增测试文件的 rustfmt 格式化/语法解析、生产文件 `rustfmt --emit=stdout --edition 2021` 解析及 `git diff --check`，均 exit 0。这些检查不提供 Rust 类型检查或运行时保证。Native/WASM/Worker/UI/浏览器全部 **NOT_RUN**，状态 **WIP_UNCOMPILED**；资源窗口获批后先执行本反例及现有共享宿主/死亡续体定向用例，再由父端复审。

原32输入逐项哈希一致；不改旧结果或旧证据。本轮只本地 WIP 提交及恢复 bundle，未推送、未查 Actions、未改 workflow、未合 main、未部署。上轮一次性 CI/push 例外已在 fffab715 消费，不延续到本片。

修复前源码哈希与实际静态命令见 `docs/evidence/cq15-pending-caller-integrity-20261026/`。
