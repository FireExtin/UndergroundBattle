# BQ001 来源 WIP：装卡前暂停检查点（2026-10-25）

当前源码/测试固定于 `401445a07de099b80849a19bcfb58757ca864ef3`，生产实现提交为 `8998964934575489500033b888ace6cf58b1b736`。本页提交仅归档，不含进一步实现。收到用户安装 R9700 的收尾指令后暂停；未新启研发或测试，未推送、发布、部署、合 main、操作原桌。

运行时为 `rust-v0.2.113-bq001-anchor-source-wip`，卡池 `limited-v2.104-bq001-anchor-source-wip`：278 定义、236 开放、42 关闭。BQ001/BQ009/BQ033 继续 `supported:false`；已准入的 BQ034/CQ25/XG25 不变。本轮完整 Native 基线、WASM、Worker、UI、浏览器验证均 **NOT_RUN**，ABI112 的旧证据和32原始输入不改签为 ABI113。

## 原图与实现范围

BQ001「脑叶专家」原图为 `resource/ymsj-fun.github.io/cards/BQ001 脑叶专家.jpg`，SHA256 `c2c6de0b1d71e37a913c9b2e2c358a0e28702037d6983430bc7bc10a9a1b78a1`。费用1、黄色忠诚1、心灵领域、人类/法师/学者、永久势力1、防御1、白名非独有。其紫色 +X 符号为隐秘展示加值；旧规格文字“地区势力+X”存在转写错误，本片保留原图依据，没有照此改动地区图标。

- 持续：隐秘对抗时，本人展示的牌获得 +X，X 为本方秘社上名为精神锚点的实际封印牌数量。本人操控的多个正面脑叶专家叠加；横置不关闭持续能力，暗置不生效。冻结展示时的来源、载体和实际锚点，历史不随后来离场或消费重算。
- 现身触发：从本人真实手牌选择一张星辰领域牌，封印在本方秘社上，然后抓一张。使用同一共享 `seal_hand_card`、Sealed 事件和 Draw；不将该选择伪作额外费用或目标。无符合手牌时后续抓牌仍执行；可选触发被放弃则不执行。已接受触发的原来源被真实消灭或被哈达莎夺控，仍由冻结执行者完成。
- 普通正面派遣不触发现身能力；付费现身保留现有真实支付收据、旧/新实例匹配和恢复校验。新增付款前载荷反例先失败，再接入有限校验。没有给 BQ003/BQ007 或其他黄色新卡实现。

生产文件完整差异见 [production-c7-to-401445a0.patch](evidence/bq001-source-wip-20261025/production-c7-to-401445a0.patch)。新增 `bq001.rs`；公共接点在 catalog/data、rules、model、engine、resolution、mystic_contest 和 lib。单元/自然测试及 unit_support 增量由对应源码提交保留；没有第二套费用、对抗或 apply 内核。

## 实际验证

|验证|结果|边界|
|---|---|---|
|`cargo test --offline --locked -p hegemony-server --lib bq001_ -- --nocapture --test-threads=1`，focused-complete-08|11/0|该次真实工作树内容随后保存为89989649；原收据仍保留当时HEAD2a81fd47与源快照|
|`... --lib bq001_supply_public_gate -- --nocapture --test-threads=1`，closed-public-create-join-09|1/0|正式构筑验证器分别拒绝三张关闭卡；join失败保持原状态|
|`... --lib bq001_candidate_natural -- --ignored --nocapture --test-threads=1`，natural-candidate-same-turn-13|1/0|401445a0 源码加独立目录的三处 supported 开关；不等于主线公开准入|

实际命令均由本地 Node `tools/build-storage.mjs` 调用固定 Rust1.90、offline/locked、单线程并保留超时与完整输出。13次运行的成功及失败原收据、源码快照和日志都保留，索引见 [run-receipts-index.json](evidence/bq001-source-wip-20261025/run-receipts-index.json)。包括初始未注册反例、两处编译接口修正、旧实例/非法标准响应脚本、未接入口的载荷反例、候选快照遗漏 compile-time 文档、Forecast 排序及正常跨回合历史清理脚本失败；没有删断言或重签旧证据。

自然轨迹种子 **345**，两套正常50张牌组，同名合计均≤3；真实 create/join 构筑、ready/start、建立资产、秘密派遣并付费现身 BQ001、从手牌封印 BQ009、预测1、隐秘对抗展示加值、BQ033实际支付3及锚点、进场使敌方随机弃牌、暂停/恢复。共 **236** 条 Room 命令、**472** 个席位视图；每步 Room 恢复，原状态序列化一致。重复提交在 Native reducer 层以旧 expectedVersion 拒绝且保持状态，共236次；**本 BQ001 片的 HTTP 收据回放未跑**。

第5回合，BQ001实际来源 i139，封印锚点 i143；展示JC008为费用2+来源加值1=3。BQ033收据为资产 `[i135,i152,i155]` 与 `BQ033Anchor { payer:0, society_host:"society:p0", anchor:i143, deck_instance:i166 }`，实际原锚点移回拥有者牌库。无锚点时先真实拒绝一次付款，错误为“必须支付本人秘社实际封印的一张精神锚点”，状态不变。对抗与后续支付安排在同一真实回合，保留原引擎下回合清理历史规则。

候选代码没有增设 `cfg(test)` 注册表覆盖或在运行中塞牌、资产、墓地、标志。只在导出的副本中翻三处目录开关，全部构筑/命令继续使用原验证器；差异见 [candidate-catalog-only.patch](evidence/bq001-source-wip-20261025/candidate-catalog-only.patch)，全部编译输入哈希见 [candidate-source-manifest-v4.json](evidence/bq001-source-wip-20261025/candidate-source-manifest-v4.json)。自然测试默认 ignored，须显式内部候选环境。主线 public create/join 的三卡拒绝已单独验证。

## 前三张准入的父审补充

`fe2b7f93` 相对 `5fc422ba` 的全部生产差异原文见 [three-card-production-5fc-to-fe2.patch](evidence/bq001-source-wip-20261025/three-card-production-5fc-to-fe2.patch)：仅卡池102→103、BQ034/CQ25/XG25三处 supported=true，以及删除 deck.validate 的 cfg(test) 候选注册表分支。engine.rs该段仅更新cfg(test)旧准入集合，无运行时卡牌接线重写。

测试接口移除候选 registry/scope，用正常构筑；本地测试 wrapper 仅固定构造 RNG 和可信时钟，生产 WASM 相同，不注入局面、不开放卡。5个公开HTTP用例：

|用例|原始命令|冷重开|
|---|---:|---:|
|BQ034真实鲜血入墓→进场饮血→反制，seed1603|386|4|
|BQ034展示→真实目标/饮血，seed1603|332|4|
|CQ07月标→XG25死亡封印/拥有者行动→CQ25非空检索，seed1723|357|4|
|BQ034同名两张进场独有牺牲，seed171|229|4|
|未包裹生产Worker的随机种子普通建房/加入/开始/暂停/重开恢复|5|1|

前四例合计1304命令，每个原始命令立即重发验证已存HTTP收据及存储不变，最终冷重开后再重放首条旧收据；第五例5个命令亦逐条重复。共1309原始命令、17次冷重开。该阶段完整Native2638/0、5个HTTP用例/既有集成/源码门禁/构建的原始证据仍绑定fe2b7f93，详见既有 [三卡准入交接](NEXT20_THREE_CARD_PUBLIC_ADMISSION_2026-10-25.md)，不作为本ABI113的新测试结果。

## 参考库与下一批草案

已按本地原记录和固定源对象核实 `RETORETO/UnderWorldPro@0d60c78b68697867697f20f0c171e41445735cc4`。README保留所有权利；用户原消息 `Sentinel_e03beb50ee4881918270242ed3e5bcde`（2026-10-21T08:45:37Z）确认代码及插画作者直接许可。保留原出处/作者，不宣称变成MIT。667条数据和135注册效果不能等同667张完整实现。已新读取15个候选TRES，逐一按Git blob校验；没有执行外库脚本。

`SuezKrish/Underworld@a8586caccd9be0cad9957e78b918245f0ddd6f85` 的README本轮重新核实无整体许可，与此前仅观察README的1fb10f60版本一致。仅参考目标实例/区域版本、费用与续体、墓地区域别名、重复选择、封印载体失效和生成指示物的测试思路，独立Rust实现；本轮没有复制其源码/素材到产品，也没有执行或声称其测试通过。读取的bq/cq效果入口用于识别依赖，不照抄该实现的特例或随机UUID。

下一批20仅为**未完成核对的草案，0实现/0准入**：CQ11、JZ53、XQ13、CQ10、CQ09、BQ084、XQ35、BQ086、BQ088、CQ27、CQ14、XG19、BQ047、CQ13、BQ095、JC028、JC053、JC097、BQ055、BQ056。其中先复用现有事件、手牌/墓地移动、共享隐秘、封印与有限选择；生成指示物完整生命周期、形态/身份转换、墓地别名与核心角色是各自新增依赖。JC028/JC053/JC097既有疑点仍独立保留，BQ055/BQ056依赖核心角色，不把它们当立即可实施项。没有把旧已整合20重算。

本轮只直接新核6张原图（CQ11/JZ53/XQ13/CQ10/CQ09/BQ084），不能声称20张全核；例如CQ09原图两处列表含“梦龙”，旧转写漏项，已记录而未开始实现。TEST015/034/052/033与ZHWM026缺固定UW数据身份对照，保留为后备，不凭近名替换。逐项事实、未核内容与源blob索引见 [reference-intake-status.json](evidence/bq001-source-wip-20261025/reference-intake-status.json)。用户装卡前已停止此研究。

## 恢复与暂停

原32输入逐一哈希与本片开始前一致，仍保持原位置/原内容，未纳入提交；原桌97EC1A99E58C、双席、线上部署不动。当前源码和测试保存本地，单一开发线，没有新writer/worktree。最后两个正式审查远端只读ref分别为e7b3366b16ed7d70783ce00bf96e87df4d220789与a520f7680387831b7ee011b42d4c445fca2183de，未发生已知远端变化。未配置本地branch upstream，不自动改写远端映射。

没有推送：动态Copilot workflow触发定义尚不明确，先前白名单拒绝没有重试，父端正等待用户对CI例外的明确答复。未改workflow、Actions、安全设置。装卡后须等明确恢复指令；下一步首先父审本片生产差异与候选隔离，再补适用回归及ABI113跨端证据，最后另审三卡准入。完整基线未跑的状态继续保留。

原始证据压缩包与逐成员校验：`/home/ddd/Documents/Codex/2026-10-24/task-7/checkpoints/bq001-wip-pause-evidence-20261025.tar.gz`，SHA256 `df710a2b87d84adc93b89b33fe38b02f1d4b5ee9ac745fd829c142458e0c751f`（24144960 bytes，4536 members）。全部失败、最终真实轨迹与原始源码快照在包内；不含凭据、私人记录、数据库、依赖缓存和构建产物。最终Git bundle及最后自建进程核对记录在Task7的 `BQ001_PAUSED_HANDOFF_20261025.json`，由父端用于关机前确认。
