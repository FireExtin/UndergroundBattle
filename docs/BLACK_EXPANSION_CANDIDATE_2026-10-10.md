# 黑色两牌规则引擎候选（2026-10-10）

协作分支：`codex/jz48-combined-review-20261007`；已审基点：`37d05bc09aeaaea409dcd04b2d25f0c1f4e89a0e`。本候选仅负责 WM059 雨夜屠夫与 BQ078 专业清理员的引擎定义、固定程序及 native 回归。卡池元数据及共享接口由父端统一集成。未提交、未推送、未部署，不代表独立桌面试玩验收。

## 认领文件

- `rust-game/src/black_expansion.rs`：两牌定义、闭程序/来源/冻结目标/存读守卫；BQ078 非目标死亡回收续体。
- `rust-game/src/black_expansion_tests.rs`：21 个 native 回归，调用生产 Game/Room 接口；明确标注布局与 primitive 事件输入。
- 本文档。

未编辑 UI、Worker、D1、Sites、公共卡池或其他派的模块。

## 原图与规则落地

已用 `view_image` 直接阅读两个完整原图，并复核现有完整规格：

- `resource/ymsj-fun.github.io/cards/WM059 雨夜屠夫.jpg`：金字唯一角色，副标题连环杀手；费用 2、黑色忠诚 1、人类/罪犯、防御 1、永久战斗 1、杀伤 1。文字为“当雨夜屠夫进入一个地区时”，故使用 `EnterRegion`：公开派遣、付费现身及公开在场移区均可触发；目标是冻结进入地区中的敌方公开角色，印刷费用不超过 1。声明与结算均检查合法性；来源后续离场、移区或转控不抹掉已经声明的能力。目标离场/转为本方/离开原地区会取消该能力。
- `resource/ymsj-fun.github.io/cards/BQ078 专业清理员.jpg`：费用 2、黑色忠诚 1、人类/雇员、防御 1、永久影响 1、临时战斗 1、公开；没有魔法领域或精神防御。进场触发使用 `Enter`，指定一个存活玩家，牌库顶至多四张使用既有置墓与重置路径；移区的 `EnterRegion` 不重复触发。空库及短库均自然结算，不执行抽牌、不触发死亡。墓地遵循既有拥有者归属规则。
- BQ078 死亡触发原文没有“目标”。声明阶段可接受或不发动；接受后响应窗口结束时查看冻结操控者的当前墓地，按名字“无名尸体”提供候选，有牌时必选一张，无牌时自然结算。选中的牌重置为新实例，拥有者仍保持；目的地是冻结操控者的手牌。离场前快照允许拥有者与操控者不同。既有 XQ46 无名尸体直接进入这条链，不补空效果。

本批没有尚待解释的卡文语义。所有策略与 UI 可用性仍需父端独立试玩。

## 父端共享挂钩

由父端统一修改：

1. `lib.rs` 注册模块与测试；`rules.rs` 将 `ability`、`target`、`with_abilities` 开放为 `pub(crate)`，插入两牌定义，调用本模块 `validate_ability` 与 `validate_definition`。
2. 新增固定 `Op::BQ078ReturnNamelessCorpse` 与 `FrameChoice::BQ078CorpseReturn`。
3. `source_snapshot` 为 WM059/BQ078 保存原地区实例；WM059 同区目标使用原地区实例校验，以阻止地区替换继承旧目标。
4. `resolve_frame` 初始调用 `validate_black_expansion_frame`；固定 Op 分支调用 `bq078_return_start`；`choose_frame` 固定 choice 分支调用 `bq078_return_complete`。
5. `Game::from_persisted` 与 `Game::choose` 调用 `validate_black_expansion_state`；它审查派遣/现身空程序、触发声明、冻结目标、堆栈、执行游标、费用记录以及死亡回收选项的完整性。

只放行本批精确印刷程序与既有精确声望/威名获授程序；禁止重键、移植、嵌套固定回收 Op、伪造成本、重复步骤或续体越权。

## 回归范围与证据

测试使用明确 native 布局，不构造空效果，不 mock Game/Room。正向付费动作经过实际 `Game::apply`、目标选择、响应堆栈和守卫；每个公共 checkpoint 序列化并恢复 RoomEnvelope，比较四席 view。

- 两牌完整元数据与能力结构；部署费用/忠诚失败原子性；公开 BQ078 禁止从手牌暗藏。
- WM059 生产战斗结算中仅公开且竖直参战时加入杀伤 1，暗藏/横置不贡献杀伤；BQ078 临时战斗图标只在先手方贡献。
- WM059 真实派遣与付费现身，只提供同区敌方印费 ≤ 1 的公开角色；友方、远区、费 2、暗藏目标非法；可不发动，无合法目标则不发声明。
- WM059 冻结行动者与进入地区；来源离場/移区/转控不取消；目标转控/移区/离场取消。真实 JC075 救援响应先返手，旧目标失效，已付响应费用不返还。
- WM059 印刷费用不受目标当前费用减免影响；目标护盾在结算消耗一个并阻止消灭。两牌的目标声明及已入响应堆栈存读重放执行相同行动得到相同状态。
- 地区移动的明确 primitive 输入经生产 `emit_event`/`drive`/声明/堆栈完成：WM059 使用 `EnterRegion`；BQ078 不将移动当进场。
- BQ078 四席行动者 × 四个目标玩家 × 牌库长度 0/1/3/4/5，始终只置墓该目标玩家的牌库顶至多四张，非抽牌/死亡；死亡回收与来源消灭响应分别真实覆盖。
- BQ078 冻结死亡前操控者、跨拥有者原牌身份、非目标选择、结算时才看当前墓地、无候选、不发动、暗藏死亡不触发；拒绝错席、错墓地、零选/多选/重复及重复旧选择命令。
- BQ078 真实 JC091 消灭后完整回收；Game 待响应及待回收存读前后同命令同状态；Room 回收命令重放同状态、重复 command receipt 不重复回收。
- 精确定义禁止重键/移植/修改触发/额外成本/嵌套/新增 traits；34 种冻结 frame/回收 metadata/牌定义篡改要求 Game 与 Room 两个存读入口同时拒绝。
- BQ078 扫描墓地前用 fallible 目录查询验证每张牌定义及实例归属，再构建回收投影；候选与非候选定义被改为 unknown 时，`catch_unwind` 要求 Game/Room 存读、真实选择、固定操作开始/完成均返回 `Err`，不 panic 且不改状态，非法持久样例同时导出。

统一构建由父端执行，避免共享缓存并发。可设置 `BLACK_EXPANSION_EVIDENCE_DIR` 导出关键 Room 状态、四席投影、真实回收命令与 expected transition、非法持久样例。本代理未自行运行 cargo。

父端首轮原始证据：`/workspace/faction-evidence/focused-native-first.log`，黑色 19 项通过、1 项失败。失败为救援响应测试在行动方仍持优先权时读取防守方的 legal actions，未取得激活动作；本候选已修为通过真实 `pass(0)`、`pass(1)` 交出优先权后取得并提交防守方救援，保留 GREEN/Room 证据路径。首轮未据 19 项通过冒称完成。

父端开发只读审查另发现待回收墓地牌定义改 unknown 后的 `card()` panic 路径；已改 fallible 候选集合验证，并新增第 21 项无 panic 回归。

第二轮原始证据：`/workspace/faction-evidence/focused-native-second.log`，黑色原 20 项全通过，第 21 项断言因红色模块的全目录守卫更早返回通用定义错误而失败。现 Game/Room/真实 choose 要求准入定义相关 `Err`，并直接调用黑色 state guard、start、complete 单独要求 BQ078 错误，保留 `catch_unwind` 与原状态不变断言，避免靠其他模块保护掩盖黑色操作漏洞。

最终专项原始证据：`/workspace/faction-evidence/focused-native-final.log`，黑色 **21/21 通过**；整批 **83 passed, 0 failed, 0 ignored; 736 filtered out; finished in 86.91s**。本批两个 Rust 文件 `rustfmt --edition 2021 --check` 返回 exit 0。黑色 fresh 证据目录为 `/workspace/faction-evidence/final/black/`，共 41 个 JSON，包含 34 个非法持久样例（含 `invalid-100.json`、`invalid-101.json`、`invalid-102.json`）、四席死亡回收选择、真实 paid deploy/reveal 目标与 Room 回收重放命令。实际 Game/Room 步骤的 fresh GREEN 原始 body 在 `/workspace/faction-evidence/final/native-bodies/`，由父端统一收录与 WASM 重放。

## 数量与状态

本批完成两张规则候选，21 个开发回归通过，四派专项 83 项通过。其余黑色牌未认领；本候选未将其他牌列为已完成。父端继续全量 Rust/WASM 重放、完整批复审与独立桌面试玩，再由父端决定同分支正常提交与统一发布。未提交或推送，未宣称独立试玩通过。
