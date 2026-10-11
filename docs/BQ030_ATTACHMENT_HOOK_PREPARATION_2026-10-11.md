# BQ030 裁定官：结附观察接口与可执行回归准备

> 本文保存本地 `f98d1e0` 的准备阶段。后续实际纳入与验证结果见 [BQ030_CARD_ADMISSION_2026-10-11.md](BQ030_CARD_ADMISSION_2026-10-11.md)，本文的未准入边界不是当前状态。

意图：`user_requested`。从本地已审查提交 `327dc8db49e3ff650ac6236c90903b2ba4e9ec00` 接续，完整原工作保留。本轮新建本地准备分支，不新建 worktree；不修改 `engine.rs`、三项 P1 或前端交互。BQ030 保持未准入，运行卡池、规则和版本不变。此准备补丁不构成整卡实现或发布批准。

## 来源与已确定语义

固定 `docs/factions/card-specifications.json` 的 B16/BQ030，原图实印24/98，SHA256 `85aaa7444b65060ab09f33f2ea9057647f88c961ff103fcca0c0763ce2564b86`。原文：“触发 当一个附属结附于本方角色上时，抓一张牌。”手册印刷P1/P2/P6/P11规定本方仅指本人操控、普通触发可放弃且可响应、同一玩家选择同时触发的入栈次序；该卡面没有地区、附属控制者或每回合次数限制。XQ47卡面明确结附到“另一个”目标角色；既有宿主横移语义不是重新结附。

因此只按 `host.controller == observer.controller` 筛选所有地区明置裁定官，不比较 team、owner 或附属控制者。合法敌方／队友附属结附于本人的角色也符合；队友宿主不符合。成功转结同样观察；失败守卫、来源消失、普通宿主移动、EnterRegion、Reveal均不产生新成功结附事件。

## 给共享机制任务的接口

本轮实际准备了不派发事件的只读接口：

```rust
Game::snapshot_attachment_observers(&self, attachment_id: &str)
    -> Option<AttachmentObservation>
```

返回真实附属实例、宿主实例、读取时的宿主操控者及符合条件的裁定官 `SourceSnapshot` 集合；观察者操控者及地区被冻结。验证附属已在场且明置、有合法明置角色宿主。返回的发现顺序不是声明／堆叠顺序。接口不决定读取时点，不做抓牌、不排次、不入队，也没有生产调用点。

待父端安排的派发接口建议为：

```rust
Game::observe_attachment_committed(&mut self, attachment_id: &str)
```

它取得上述快照，按批准的同时触发选择流程建立 `Event::AttachmentCommittedObserved` 声明，绑定 BQ030 唯一完整程序 `Op::Draw { player: PlayerRef::Actor, count: 1, end: DeckEnd::Top }`。BQ030自身事件不能登记为Enter，不复用死亡／地区宿主上下文字段。若要在存档中证明历史宿主资格，另需明确并验证其事件快照格式；仅冻结source/actor只能验证程序与行动者，不能独立证明历史宿主关系。

无需改动当前独占的 `engine.rs` 的接线方案：

- 新附属：`engine.rs::resolve_stack` 已写入fresh后的附属，并完成Attached操控权更新，再经 `enter_triggers` 到 `resolution.rs::emit_event(Event::Enter)`。可在后者仅对真实 `attachments` 中的新实例接派发接口；这只是候选代码接点，不能由既存代码顺序推定卡牌裁定。
- 换宿主：`attachment.rs::reattach_source` 成功改写不同宿主后接相同派发接口；失败不接。与持续属性、操控权及死亡清理的前后顺序仍须父端确定。
- 完整能力、冻结actor、单步Draw帧、待选与堆叠容器校验可沿 `validate_blue_expansion_state/frame` 的现有接线扩展，不必在engine新增调用。当前准备补丁未做该扩展。

以下是正式实现的阻塞，不能用FIFO或代码顺序自行裁定：初拥即时操控变化前／后的宿主资格；结附立即导致裁定官死亡时的快照时点；同一玩家多个同时触发的自主入栈排次。现有逐个FIFO询问不等于玩家自由选择排序。相关历史死亡观察裁定不自动适用于新结附事件。

## 最小补丁与可执行测试边界

`bq030_preparation.rs` 只提供候选快照收集器与四项单元契约；lib只登记未调用模块，没有BQ030卡池／规则登记。契约中的BQ030身份是明确的synthetic初始对象，只验证可见性、跨地区、控制关系、冻结和集合，不作为真实Room／已准入牌证据。

`tests/bq030_attachment_regression.rs` 的真实Room用例只在初始布局设置牌；此后每步只走 `RoomEnvelope::transition`，响应使用PassResponse，核对返回journal重放、持久化恢复及四席视图。它提供：既有BQ040真实付款测试；BQ030完整字段门禁；真实付款部署后跨地区结附，接受恰好抓一张／拒绝不抓；敌方／队友附属与队友宿主矩阵；真实XQ47支付两资产转结。真实Room用例不补调快照／观察器来伪造通过。

当前BQ030未准入，四项BQ030候选准入门禁应实测失败，不能忽略或记为绿。后续完整登记候选字段／规则但暂不接hook，再复跑，应继续在“成功结附没有观察待选”断言处失败；接批准入口后才能绿。这一后续阶段本轮未执行。

复跑命令（先source `/workspace/tooling/env.sh`，使用磁盘／内存护栏）：

```sh
node tools/build-storage.mjs -- cargo test --locked -p hegemony-server --lib bq030_preparation -- --test-threads=1
BQ030_TRACE_DIR=/workspace/bq030-preparation-20261011/room-traces node tools/build-storage.mjs -- cargo test --locked -p hegemony-server --test bq030_attachment_regression -- --test-threads=1
```

原图副本、规范归一化候选JSON、原始日志和准备patch保留在 `/workspace/bq030-preparation-20261011`，不开放生产卡牌。Native新卡测试尚红，尚无BQ030生产WASM等价、SQLite回执／重开、完整排序、非法存档或真实浏览器验收；这些必须在协调后完成，不能借用上一轮两卡WASM结果。

实际验证：收集契约初次运行因未准入registry访问而1通过／3失败；改为独立冻结SourceSnapshot后，最终 **4通过／0失败，839过滤**。最终Room目标 **1通过／4失败，0忽略**，四项失败均为BQ030尚未准入的明确门禁；既有BQ040框架共导出5个真实Room转换，逐条journal重放、恢复和四席视图通过。审查发现的teams分队及拒绝后残余效果漏检也已修正。生产WASM目标的 `cargo check --locked -p hegemony-wasm --target wasm32-unknown-unknown` **编译通过**，仅验证这份未接线代码可跨端编译，不能称BQ030行为等价已通过。

卡图修复另已正常推送到不存在的新审查分支 `codex/card-scan-provenance-review-20261011-e067b3b`，远端精确SHA `e067b3b167367fc98562095abbd02db337fd9c5a`。仅4相关文件，三份功能文件与327dc8d逐字节一致并独审通过；完整证据留本地。未覆盖原候选分支，未合main或发布。
