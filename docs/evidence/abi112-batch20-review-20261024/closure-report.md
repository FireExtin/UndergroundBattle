# ABI112 首批20张非作者独审：修复复验

结论：原固定 `91ae5336a89a69049c23544be3abf67ca593b6bc` 上独立确认的 R1（P1）和 R2（P2，两种场景），在作者固定候选 `33bb5824f7c756a87f68c0b19ad1612a0b59e233` 的本次窄范围内完成修复闭合。四个原失败用例全部通过；这只闭合两个已确认问题，不构成完整 ABI112、全20张或发布验收。

## 固定来源及隔离

初始审查的精确基线、原卡文/规则、bundle 和输入哈希、20张静态覆盖仍见 [report.md](report.md)、[review.json](review.json)、[coverage.json](coverage.json)。原始独立运行共5个不同用例：1个无合法传闻宿主对照通过，4个反例失败。原作者全量2494通过/9失败及95、72定向结果仍为作者证据，未改写为完整重跑通过。线上 ABI111 的6回合9:0、20原图加载不作 ABI112 验收。

复验沿用独立普通克隆 `fixed-91ae-clone`，拥有自己的 `.git`。从原91依次正常 cherry-pick `86b4cdd33f1fcce02deb46a2f12ea100036c5905` 和 `33bb5824f7c756a87f68c0b19ad1612a0b59e233`，克隆提交分别为 `8ae12c12162439de6f4804d427ede071ef08aebf`、`e3eed7604c17f70582d38d1924096eca347197b1`。按父线程授权省略82个消费者/工具接线提交；已核对已提交的 rust-game、wasm、data、Cargo 输入与作者固定33候选无差异。唯一工作树变更是 `rust-game/src/lib.rs` 中三个 `cfg(test)` 引用；三份审查员探针源逐字未变。作者工作树未写入。

## 静态复核

独立复读最终33相对原91的三个产品运行时文件，最终补丁保存在 [closure-runtime-final.patch](closure-runtime-final.patch)。

- `rumor.rs:420,514–519`：传闻 cause 先校验完整 CQ28 canonical frame，只额外接纳 CQ28 原技能的 `CQ28Mode { mode:0 }`、原 actor 的牌库磨牌、非重抓、至多6张。原传闻续体恰好一次校验，以及 CQ28 的来源、双模式顺序、原目标、实际费用、游标和已接受 guard 校验仍在；没有放开任意 Op 移植。
- `batch3_entry_order.rs:21–22,51`：有限名单加入 WM057、WM025、BQ039；WM057/WM025 引用含实际 contest_id，BQ039 含冻结的 battle observation.event_id。继续引用原 effects 队列中的声明，不复制可修改程序。
- `engine.rs:1535–1548`：用 BQ039 观察产生的真实 event_id 注册排序。tail 起点在已有 Damage/Recipient 入队之后，仅对随后 path/eye/glory 声明使用既有排序机制，保留先伤害后胜利触发的处理顺序。最终33也修正86中该匹配分支所需的 `crate::rules::Op` 命名空间；以最终33哈希和实际编译结果为准。

在这一修复边界内未确认新的问题。没有新增兼容内核、扩大费用横杠解释或修改共享声望/战斗荣誉的授权边界。

## 独立动态证据

在父线程明确释放 CPU31 且无作者 Cargo 后运行；`CARGO_BUILD_JOBS=1`、`RUST_TEST_THREADS=1`、`nice19`、`taskset -c31`、offline/locked，使用本任务本地 Zig/linker 和 batch4-build 缓存。完整命令、环境、探针哈希、起止时间见 [closure-execution.json](closure-execution.json)，原始输出见 [closure-native.log](closure-native.log)。

```text
taskset -c 31 nice -n 19 node tools/build-storage.mjs --output-dir /home/ddd/Documents/Codex/2026-10-21/task-3/serial-intake-20261024/abi112-independent-review -- cargo test --offline --locked -p hegemony-server --lib -- --nocapture --test-threads=1 independent_review_cq28::independent_cq28_rumor_pair independent_review_order:: independent_review_eye::
```

| 原问题及正常命令场景 | 原91独立结果 | 最终候选独立结果 |
| --- | --- | --- |
| CQ28 实付4，磨6+磨6，牌库含传闻且有公开合法宿主，正常最后响应 pass | 被传闻 cause 校验拒绝 | 到达合法传闻选择，通过 |
| CQ28 实付4，磨6+抓1，同上 | 同类拒绝 | 到达合法传闻选择，通过 |
| 同控 WM057+WM025，实付2并横置，四位实际展示，己队赢得隐秘对抗 | 强制 WM057 先声明，无排序组 | 本人得到恰好2项排序选择，通过 |
| 两张 BQ039 均正常实付1结附同角色，真实2:1战斗胜利并分配伤害 | 第一眼固定先声明，无排序组 | 伤害后本人得到恰好2项排序选择，通过 |

实际输出：`4 passed; 0 failed; 0 ignored; 2507 filtered out`，退出0，运行28.529秒，其中测试0.76秒。该4项是原4项的复跑，不增加“不同用例”数量。原无宿主通过对照按要求未重跑；没有重跑作者完整或相邻套件。编译器原有4条警告保留在日志。

独立探针验证边界必须明确：CQ28 两项只独立验证到首个合法传闻替换提示；隐秘双触发和双眼只验证到本人恰好两个原声明的排序提示（隐秘用例另作 restore 校验）。之后传闻接受/放弃、两种次序各自的完整声明与结算链，本次未独立实跑。作者三个新增回归及130个相邻通过仍是作者证据；随后6个正常 Room 跨端见证由作者继续建设，不归入本报告。

## 保存和剩余范围

原 [initial-review-manifest.json](initial-review-manifest.json) 锁定的16个文件（包括原报告、覆盖、失败日志及三份探针）全部大小/sha256未变，见 [closure-original-integrity.json](closure-original-integrity.json)。原第一次缺 cc 的启动失败仍保留，不算测试失败或本次阻塞。初始91报告继续如实表示该固定快照需要修改；本报告单独记录修复候选闭合。

本次没有全量 Native 重跑，没有独立 WASM/Worker/浏览器验收，也没有为其余16张增加动态覆盖；20张的完整组合空间和更多损坏存档不在复验范围。没有读取其他 task、被拒路径或旧私有会话，没有外部 Oracle、push、CI、main、部署及公开端点操作。当前审查无待解权限或工具阻塞。独立 Cargo 已于2026-10-24 12:38:21 UTC结束，CPU31已归还父线程，此后不再启动 Cargo。
