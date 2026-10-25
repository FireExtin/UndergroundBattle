# BQ033 / BQ034 独立切片交付

固定开发基线：`476e09bc162afcec63cc22e7d96365da65499304`（ABI112）。
独立普通克隆：`/home/ddd/Documents/Codex/2026-10-24/task-6/UndergroundBattle-next20-blue-black`。
开发分支：`dev/next20-blue-black-20261025`。
原主线只读观测为 `bb74e675bc78cf41ac875451658ea5a898b283ef`，未用其替换开发基线。

这是未接入主线的模块 WIP，**不是两卡完整准入，也不是20张批次完成**。内核、费用续体、cards.json、ABI及WASM产物均未直接改动。

## 原图及规则

已直接查看并核对下列原图，完整原图保留在克隆中：

| 牌 | 原图 | SHA-256 |
| --- | --- | --- |
| BQ033 热夜之梦 | `resource/ymsj-fun.github.io/cards/BQ033 热夜之梦.jpg` | `a00ff13389fddf8e27073ec3e507f979d5c12b6eeffb8cf28211a5fdaef8ede8` |
| BQ034 雾羽弥生 | `resource/ymsj-fun.github.io/cards/BQ034 雾羽弥生.jpg` | `f91de56f6dfdaa41bb0d32759fcf32ad01985dc4c00998fc0753e8622fec59b2` |

BQ033：3费、无忠诚、角色／梦境／造物、星辰领域、防御1、永久战斗1和临时战斗1、灵体。额外费用是把本人秘社上封印的一张“精神锚点”洗入其**拥有者**牌库；进场时目标玩家随机弃一张手牌。

规则手册“暗藏者翻面”第3步明确要求付费现身也付额外费用。因此行动检查同时约束普通派遣和付费现身，秘密派遣不支付这项费用。支付辅助函数从本人秘社封印区取牌，重置实体并洗入真实拥有者牌库，不先送回手牌，也不发弃牌或死亡事件。

BQ034：4费、两个蓝色忠诚、独有“美浓之鬼”、角色／吸血鬼／忍者、血脉领域、防御1、永久战斗／势力各1、临时调查1。隐秘展示加1；本次隐秘对抗后目标本人吸血鬼饮血1，与获胜无关。进场饮血1。本人吸血鬼成为敌方**卡牌**目标时，可牺牲结附于该吸血鬼的本人附属，然后终止原卡牌。

旧规格的“隐秘地区势力+1”不作为执行文本：原图是隐秘加值符号。本切片的目录建议改为“隐秘+1”；未直接修改共享来源JSON。目录中的忠诚使用引擎要求的“蓝色”，领域使用现有“血”，未照搬来源索引的简写。

## 已写的模块

- `next20_blue_vampires.rs`：固定定义、封印费用候选及支付收据、随机弃牌、卡牌指定目标观察、牺牲及终止、饮血、展示后置续体、行动／待选／声明／帧及队列检查。
- `next20_blue_vampires_tests.rs`：隔离原生定向用例。原生夹具直接布局与实际客户端行动用例在测试注释中区分。

饮血复用 `blood_options`、`attach_selected_blood` 和正常进场触发。当前饮血角色的操控者选择本人墓地或本人秘社封印的鲜血。终止使用原卡牌事务的帧ID，牺牲仅限原吸血鬼上、由发动者操控的附属；未牺牲、卡牌已消失或选错宿主均不能终止或误付。被终止的事务走已有的拥有者入墓路径，不退款，不记角色死亡。

## 主线需要完成的接线

1. 增加 `Op::Next20Blue(BlueProgram)`、`Event::Next20VampireTargeted`、`FrameChoice::Next20Blue(BlueChoice)`；登记模块及固定定义。目录加载器还需保留BQ034已核对的副名称“美浓之鬼”。
2. `rules::validate_ability` 在通用隐秘能力校验前路由 `related`；`validate_definitions` 调用本模块校验。行动与选择检查在支付前调用。
3. `source_snapshot` 为两卡保存原地区实体；普通支付、守卫和响应继续使用现有内核。
4. 真实卡牌事务的 `push_frame` 及立即卡牌分支，与 WM058 同一观察点调用 `bq034_observe_card`。不能对普通发动／触发能力无条件调用此观察器。
5. 隐秘数值计算叠加 `shown_bonus`；展示后钩子调用 `bq034_shown`。目标和饮血沿原 `MysticPhase::After.post_frame` 恢复，完成后清掉本后置帧。
6. `validate_next20_blue_state` 接入运行时和持久恢复入口；帧、声明及待选入口分别调用相应校验，不能只接执行分支。
7. **BQ033 费用依赖尚未封口**：基线没有 BQ009 精神锚点运行时。主线须为普通派遣和付费现身增加真实额外费用选择与 `PaidCost` 收据，调用 `bq033_pay_anchor`，并验证部署／现身帧携带该收据。不能只接受 `cost_selected` 而未实际支付。夹具内的 BQ009 只是命名资源测试记录，无黄色玩法实现。

`NEXT20_BLUE_VAMPIRES_PARTIAL_CORE_WIRING_2026-10-25.patch` 只供主线 review 接线，未直接覆盖共享文件；费用闭包和黑卡死亡替代未包含，不能单凭此补丁开放准入。四卡目录追加建议另存为 `NEXT20_BLUE_VAMPIRES_CATALOGUE_SUGGESTION_2026-10-25.json`，均为 `supported: false`。

## 验证范围

最终执行结果与精确源码散列见 `NEXT20_BLUE_VAMPIRES_TEST_EVIDENCE_2026-10-25.json`。原始构建／测试日志按该文件路径保留。

隔离夹具脚本 `NEXT20_BLUE_VAMPIRES_HARNESS_2026-10-25.py` 仅写 `/tmp/next20-blue-black-harness`，用未修改的固定基线内核加有限接线编译这两个模块；它不是生产安装器。运行时目录和命名 BQ009 夹具仅供本地原生测试，不构成准入。构建离线、`CARGO_BUILD_JOBS=1`，未启动服务、浏览器或占用端口。

全量回归、原主线整合后验收、WASM／Worker、双席正常UI试玩、发布、部署、GitHub CI：**NOT_RUN**。
