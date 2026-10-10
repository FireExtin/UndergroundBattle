# 灰色 JZ44 / JZ45 首批候选（2026-10-10）

范围：仅新增 `rust-game/src/gray_expansion.rs`、`gray_expansion_tests.rs` 及本说明。基于协作分支 `codex/jz48-combined-review-20261007` 的已审起点 `37d05bc09aeaaea409dcd04b2d25f0c1f4e89a0e`。共享注册、卡池、枚举和调用挂钩由父端统一集成；没有编辑 UI、Worker、D1、Sites 或发布文件，没有提交或推送。

## 原图复核与完整实现

实际查看两张完整原图，并与 `docs/factions/card-specifications.json` B23 逐字段核对。

| 卡 | 原图字段 | 候选程序 |
| --- | --- | --- |
| JZ44 洛杉矶巡警 | 角色：人类 / 警察；灰；费用 3；灰忠诚 1；防御 1；永久战斗 1、势力 1；临时战斗 1；公开、护卫 1；只能打出在城市 | 可响应的可选进场触发，仅目标原地区正面罪犯角色，复用 `Hide(Target(0))` 后 `Draw(Actor, 1, Top)`；结算守卫失效时整程序取消，不能独立抓牌 |
| JZ45 警用直升机 | 角色：人类 / 警察 / 载具；灰；费用 5；灰忠诚 3；防御 3；永久调查 / 战斗 / 势力各 1；无临时图标；公开、机动；只能打出在城市 | 可响应的可选进场模式：在原地区角色或暗藏者上放 1 个锁定，或消灭原地区有锁定标志的角色 / 暗藏者；机动复用既有 JC014 的对抗开始、须来源未横置、组队仅相邻地区程序 |

原图 SHA256：JZ44 `b5ca6ecde925805b86eeac70bd796ad402695daf342835abfd669e9a157637eb`；JZ45 `20e18006a7b2f032d7d236f909dc57d75bfc14dc3acf36291c7ca16f3962008f`。无新语义决定或待补条文；自我锁定 / 自我消灭未印“其他”，所以保留合法。护盾、屏障、目标控制者、原实例、标志清除、死亡触发、附属离场均走现有规则。

## 共享集成挂钩

1. `lib.rs` 注册 `mod gray_expansion` 和默认非 `society-fixtures` 测试模块。`rules::{ability,target,with_abilities}` 改为 `pub(crate)`。
2. `rules::Op` 添加无参数 `JZ45LockLocalTarget`；`TargetPredicate` 添加 `JZ45LockedLocalTarget`。专用枚举避免拓宽原 `JC069` / `JZ43` 封闭程序。
3. `rules::definitions` 注册两张 `gray_expansion::definition(id)`；`validate_ability` 调用模块同名函数；`validate_definitions` 每张调用 `validate_definition`。现有 finite predicate 准入对 JZ45 使用模块精确准入（完整原模式和选定模式展开版本）；city restriction 的封闭定义准入增加 JZ44 / JZ45。
4. `resolution::source_snapshot` 捕获 JZ44 / JZ45 的地区实例；`valid_binding` 的原地区实例条件从 JZ43 扩至三张，并且仍只限 `Range::SourceRegion`。新增 predicate 仅当来源 JZ45 且目标 `lock_markers > 0`；不可复用警犬追逐谓词，后者有“不能原地移动”的发动检查。
5. `resolve_frame` 前置 `validate_gray_expansion_frame`；新操作调用 `jz45_lock_local_target(&frame)?`。`Game::from_persisted` 与 `RoomEnvelope::from_persisted` 各调用 `validate_gray_expansion_state`。
6. `data/cards.json` 添加以上实读完整字段及原图来源。父端统一版本号、总数断言和差异审查。

声明只准入完整原程序、JZ45 的两个精确选定模式以及既有 JC089 / 声望获授奖励。待选状态同时校验 Mode / Target / Accept 的合法阶段形状、席位与冻结行动者相符、playerId、完整选项和投影、kind、标题与说明、非空 choiceId、min / max / allowDecline、空 amount / previewCards；错误阶段不能绕过选模式或选目标，敌席不能借篡改的 pending.seat 代替冻结行动者执行。帧校验绑定具体来源、冻结行动者、目标槽、完整步骤、堆栈身份、真实打出来源。两张新增程序不会悬停选择，拒绝持久化已接受守卫、非零游标或伪造帧内选择。自然 `engine::close_window` 为机动使用 legacy `source_region_instance=None`；仅机动准许这个值，进场与打出继续要求原地区实例。

## 回归与证据边界

新增 22 个原生规则回归。复用 `jc029_tests` 的真实 Game / Room 帮手，每步保存恢复并比较四席视图；没有 mock 解释器。覆盖真实付费部署、四席及所有拥有者、费用 / 忠诚 / 城市 / 公开 / 发动者原子拒绝、本地角色与暗藏者目标、模式合法性、完整抓牌取消、目标 / 来源离场、响应地区替换、屏障 / 护盾、护卫真实战斗分配、机动自然窗口、原实例及锁定保持 / 清除、所属者墓地与冻结操控者死亡触发、两牌与警犬的锁定闭环、持久化变异拒绝。另对五类正常待选声明各作 15 项独立元数据变异，验证 Game / Room 恢复拒绝以及敌席真实 choose 原子拒绝；实际既有声望 / JC089 威名程序验证 Accept 阶段仍可保存并结算。

另有连续真实 Room 命令链：巡警付费部署 → 进场选目标 → BeginResponse / SubmitResponse 发动 JC075 付费救援 → 目标离场 → 巡警整效果取消且不抓牌。每个 accepted journal 从旧状态重放得到完整新状态；Game / SubmitResponse 的付费、选择、响应提交日志再次重放被版本守卫拒绝；过期响应提交状态不变。既有 BeginResponse 对相同编辑意图允许重复，因此不误称所有会话事件均拒绝重复。

父端统一构建运行；此子任务没有并发 cargo build。可设置：

```sh
GREEN_EVIDENCE_DIR=/workspace/faction-evidence/gray/native-bodies \
GRAY_EXPANSION_FIXTURE_DIR=/workspace/faction-evidence/gray/fixtures \
GRAY_EXPANSION_CHAIN_DIR=/workspace/faction-evidence/gray/chains \
GRAY_EXPANSION_INVALID_DIR=/workspace/faction-evidence/gray/invalid \
cargo test --locked --offline -p hegemony-server --lib gray_expansion_tests -- --test-threads=1
```

首轮父端统一原生专项日志 `/workspace/faction-evidence/focused-native-first.log`：整个批次 61 通过 / 16 失败，其中灰色 20 通过 / 2 失败。两项为测试预期错误：BQ022 已有规则为宿主离场回拥有者手牌，误断言墓地；BeginResponse 已有重复同意图语义，误断言所有日志再次执行均失败。已据现有规则修正，保留付费 / choose / SubmitResponse 的严格重复重放拒绝断言，未修改共享规则。需要父端执行最终重跑追加真实结果。仅交两张引擎 / 卡池候选，不声称 UI 独立试玩、完整批验收或线上发布通过。
