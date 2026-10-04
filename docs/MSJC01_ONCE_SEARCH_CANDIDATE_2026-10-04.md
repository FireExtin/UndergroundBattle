# MSJC01 每局一次与黄色独有检索：本地批内候选

本篇是 `084af34` 的历史单卡检查点记录；后续完整整批候选与真实黄色目标见 [整批说明](YELLOW_SEARCH_ONCE_PRIVATE_TOP_BATCH_2026-10-04.md)。

本次在 `codex/hegemony-jc008-turn-modifier-candidate-20261004`、来源研究提交 `f4e1a65efbc90c57e09661aa723ffb115a82110b` 上实现 MSJC01；功能基线是已审 JC008 的 `cd8265ca104b1089239a3e237b9dcf5f1b03511a`。本候选尚未独立审查，不推送 GitHub、不保存或发布 Sites、不访问公网 QA 房。整批回归、版本兼容检查与完整交付依最新指示留到批末。

## 来源与项目解释

已实际查看 `resource/ymsj-fun.github.io/cards/MSJC01 帷幕守望.jpg`，SHA-256 `c61c887558cb184a532758eac5db766234c32a3a3605162a38c0899993e7bdf2`；MSJC09 原图以及手册 PDF 第 4、5、6、7、9、11 页和玩家指南第 14、24 页见前一篇来源研究。帷幕守望是黄色独有秘社/法师结社，起手 6，无打出费用或防御；构筑至少 25 张黄色普通牌。秘社不计入普通牌至少 50 张，也不计入 25 张黄色。

“每局一次”的消费点采用父线程批准的**项目解释**：合法宣告并成功支付全部费用后即消费；拒绝或费用不足不消费；付费后被终止或空搜不返还；回合重置、持久化恢复、journal 重放与旧 revision 重试不返还，仅新局开始清空。未声称存在 MSJC01 专属官方 FAQ。

## 实现落点与边界

- `rules.rs` 的 `AbilitySpec.once_per_game` 默认 false；限次只支持主动宣告，触发能力被验证器拒绝。`PrintedColorAndUnique` 同时匹配印刷派系和 `CardDefinition.unique`，不限制牌类型，也不把“唯一”构筑关键字当作金字独有。
- `SocietyZone.used_once_per_game` 是每席稳定秘社区上的 abilityId 集合，绑定秘社来源而非易变卡牌 instance。当前只支持秘社限次主动能力，未扩展普通场上角色的稳定限次身份。
- `pay_ability_costs` 先检查资格，再支付全部费用，最后消费集合键；外层事务失败整体回滚。四席分别消费，换同名秘社 instance 不绕过额度。
- MSJC01 首能力完整复用 MSJC09 的费用 3 + 横置、结算时判断先手后抓 1 程序。第二能力费用 4 + 横置，检索本人牌库中黄色独有牌、展示入手、洗牌，每局一次；两 abilityId 独立。
- 沿用现有强制检索：有命中时 min=1，空选拒绝；无命中时正常空搜并洗牌。资格与 `legalActions` 不查询私有牌库是否有命中。未改全局 Search 的选择规则。
- 新公共投影 `CardView.usedOncePerGame?: string[]` 仅返回已消费的公开 abilityId，无牌库命中信息。CardTile 使用已有能力标签显示“本局已使用”，旧核缺字段时不显示；行动仍由核心 `legalActions` 决定。
- 普通牌仍 51，旧 51 项编译后 catalog 与已审 JC008 候选逐项相等；MSJC09 catalog 保持相等。秘社仅增加 MSJC01，原图扫描 52→53。候选版本 `limited-v2.10-msjc01-candidate` / `rust-v0.2.13-msjc01-candidate`，正式 Site 及其旧核未改。

## 批内验证与失败记录

新增 13 项 Native 聚焦测试通过：24/25/27 黄色、秘社不计两种数量、四席额度、能力独立、费用及横置拒绝原子性、空搜消费与洗牌、不以私有命中控制资格、过滤牌类型/金字/“唯一”、取消不退款、跨回合及保存恢复、换 instance、防重复消费、新局重置。此前已开始的默认 Native 全量 181 项通过，最新节奏调整后未重复跑。

定向前端 7 项通过（MSJC01 4 + 已有 Society 3），TypeScript + Vite 构建通过。MSJC01 测试使用真实候选 WASM 与正常本地双人 Room 建牌、费用、横置、空搜、下回合及公共已用标记；未声称浏览器公网自然 UI 验收。

默认 oracle 仍无条件收录 MSJC01 的 16 个场景；新 `--slice-msjc01` 仅供批内定向验证，不是默认覆盖门槛。Native/WASM 定向比较通过：**16 场景、862 转换、3436 投影、4 拒绝命令、4 拒绝建牌**，u64 seed 及 opaque state 保持精确，journal 重放和恢复一致。一个正常四人场景从真实建牌、加入、准备和开局起步，经建资产、支付首能力和第二能力、空搜及跨回合证明四席消费。

其他命中/取消/换实例/结束局场景明确为合成检查点。正式黄色独有普通牌为 0；命中场景只将付费 frame 的过滤器改为**中立独有**并用真实 LC23，未改 LC23 派系或注册虚构黄色目标。因此这些场景证明通用过滤/私密选择/展示入手换 instance/洗牌调用链，不证明自然黄色检索收益。

保留失败记录：首次 Native 聚焦中的跨回合场景因测试推进上限 100 不足而失败，改成有界 200 后通过。已启动的默认 WASM 比较失败于合成付费 frame 被直接装入 `RoomEnvelope::from_game`，缺对应响应窗口；修正为通过真实 Room 付费/让过建立检查点，并验证每个初始检查点可持久化恢复。待选时的空选拒绝也改为当时记录的转换，避免错误拿最终状态比较。修正后只跑 MSJC01 定向比较，**不声称默认 96 场景全量已绿**。

本地证据目录 `/workspace/.private-validation/msjc01-once-candidate-20261004/`：`native-focused.log`、`native-default.log`、`default-wasm-verify.log`（保留失败）、`msjc01-targeted-native-oracle.json`、`msjc01-targeted-wasm-compare.log`、`frontend-targeted-final.log`、`frontend-necessary-build.log`、`scope-check.json`。旧完整 oracle 是修正前生成的历史失败输入，不作为当前通过证据。

## 下一批候选，只供选择

实际查看两张原图并确认黄派系、金色名字；二者均未注册/解锁，未计入本候选卡池：

| 候选 | 可复用与最小机制差距 |
| --- | --- |
| **WM003 千机庙离**，费用 1，黄色忠诚 1 + 星领域忠诚 2，公开 | 公开、快速行动 5 + 横置、私人牌库选择、入手换 instance、洗牌、独有进场规则均有基底；最接近现有检索。原文检索任意一张牌**未写展示**，现有 Search 入手总记公开展示，需核对通用规则是否要求展示，否则应最小扩展“是否展示”，不能擅自公开其选择。SHA `2409ef206e3af93d1eece3ec0bf4ba00a9e3d0c12a8e70443877cc8a28cc9bff`。 |
| **LC01 西比尔**，费用 5，黄色忠诚 2 | 独有/普通出场及检索目标资格可复用；“你可以随时检视你的牌库顶牌”需要一个按本人 controller 可见的持续顶牌私密投影/阅读入口，随顶牌变化刷新，离场即取消且队友/对手无泄漏。它不抓牌、不改变牌库顺序。SHA `a0defb07b43463cc882e15b993c73eda22ea77ca39062da71e6518a365cc5bd6`。 |

父线程选择后再扩卡；真正黄色目标的自然检索收益、批末默认全量、版本兼容、独立审查与试玩均待后续完成。
