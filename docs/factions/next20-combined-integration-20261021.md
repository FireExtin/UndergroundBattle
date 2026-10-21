# 下一批20张组合源码整合（Native定向通过，待构建与发布）

普通独立克隆：`/workspace/audit-underground-100161b/next20-combined-dev`；分支 `progress/next20-combined-20261021`；固定基线 `d611b3269c48612c8e1500b349fb363915779b22`。未修改根目录或原分组克隆，未推送、部署或触发GitHub CI。

组合源码及指定P2/UX差异已合并，Native lib定向回归最终为 **129通过、0失败、0忽略**，1281项未进入本次过滤范围。被测源码提交为 `e15883946d8b9c519228c8d48fae6ea275a9aa32`；完整命令、首次失败、修复和最终日志哈希见[运行证据](next20-combined-native-20261021.json)。本记录不代表全套、ABI93运行制品、浏览器对战、独立复审或发布验收通过。编译槽已交回owner，后续构建与启动由owner接手。

## 输入与目录

| 分组 | 原始提交 | 本地整合提交 |
| --- | --- | --- |
| 蓝灰7张及界面字段 | 已在固定基线，蓝灰最终源 `9966d705f35c1f54831d99de187eb0369ba41ef8` | 基线包含 |
| 黑5张 | `305a0a8ef2c2d750cb3b57bd85eba21c7c4fd704` | `4b903de` |
| 红5张 | `b6d4e3faf8abca1a2765a949f6873bf5625892b8` | `817736d` |
| 灰3张 | `359c1f3` → `bfc2bd6` → `4f820a4` → `920881e` → `28fbb63` → `17015c4` | `f190892` → `3996873` → `33f41aa` → `80afda3` → `ef0ad09` → `321daa8` |

20张精确名单、原图哈希和各卡当前状态见[机器账本](batch20-next-2026-10-21.json)。目录共193张，非地区183张，新增为红黑蓝灰各5张；没有新增中立或双色牌。原173条记录与 `7f4e6d63` 完全一致，规范化JSON的SHA-256为 `2e3c30a1172f4a995c8a5227e484360d47562dfd63c4a2e5974a55cf68303fd9`，已写入当前来源回归测试。原始历史夹具和冻结证据未改。

## 共享挂点的实际处理

对照外部辅助报告 `NEXT20-INTEGRATION-HOOKS-20261021.zh-CN.md` 后，逐项读取最终模块和实际冲突，未整段采用ours/theirs。

1. `jc035.rs` 合并JC055、BQ085、CQ08三种准备程序、事件及无附属宿主的角色来源。BQ085保持强制；CQ08保持同控制者三姐妹资格和同名每回合一次；JC055接受发动后有合法目标必须选一个，再回到剩余排序。三种待选及原事件/帧身份校验均保留。
2. 图标计算累加JZ23实体永久战斗标志、CQ17实时光环、JZ32毁灭地区永久势力、JC051临时势力光环及黑组动态图标。`fresh`清除实体标志；灰色当前公开状态与蓝色标志DTO同时保留。两个WASM示例的`SourceSnapshot.observed_exposure`初始化保留。
3. `source_snapshot`原地区UID名单取各组并集，JC039、XQ39、JZ41保留宿主UID。`requires_region_instance`仅为BQ077WaterRegion和JZ34DestroyedRegion相应谓词加绑定，既有机动目的地区槽语义不变。
4. 死亡批次在移除前各捕获一次红/黑观察者；实际成功移除后各通知一次，统一同时触发排序包含两个新死亡事件。没有第二条死亡执行路径。
5. 保留灰组`rumor.rs`逐牌替代转移。完成批次后，仅以实际`Grave`收据调用黑组真实观察器一次；删除灰组同名空stub。JZ50保留独立的一次真实入墓回调。JZ28与JC040共用原弃牌捕获/发布入口；封印成功不产生弃牌或牌库入墓观察。
6. JZ23/CQ26的传闻cause调用本模块完整帧验证，并仅接受两步原程序在Accepted/cursor=2的末步弃牌：JZ23目标玩家弃一、redraw=false；CQ26行动者弃一、redraw=true；两者必须是单张手牌转移。cause的来源/事件加入原观察去重集合，禁止与响应帧或声明重复。未放开普通`Effect::Frame`准入。原有余步唯一/不可丢失校验及JZ57独立续体均保留。
7. 红、黑、蓝来源及准备观察拒绝移植灰色条件机动的`observed_exposure`收据；合法跨卡机动仍由原严格公共校验先行认证。Game和Room恢复、apply前后及聚合状态入口的各组检查全部保留。

## 已通过的跨组定向验收

[next20_cross_tests.rs](../../rust-game/src/next20_cross_tests.rs)新增7个定向测试，所有客户命令经`Game::apply`和四席持久化往返；非法快照同时检查Game与Room拒绝。

- 精确193/183目录、20张完整定义及10张独有牌。
- JC055/BQ085/CQ08同场准备排序、强制不可跳过、接受后目标必选、同名共享一次。
- JZ23弃WM042的封印/入墓两条路径、已支付费用和永久标志不重放、伪造动态收据及重复响应拒绝。
- CQ26真实磨牌批次在传闻封印后只观察实际入墓牌一次；随后的WM042效果弃牌只在真实入墓时抓牌并通知JZ28一次。
- JZ34/BQ085/XG27共用死亡批次，按来源分别核对事件数量并拒绝重复观察。
- 四组角色通过真实JC074授予声望及真实LC12/JC089授予战功；原地区替换UID后不奖励替代地区。

原红组 `next20_red_jc051_cross_package_badge_public_search_host_restore` 已解除ignore并通过，使用真实JC039/XQ39定义，在已选秘社牌和空秘社区两种状态下共检查4种组合；覆盖原牌库UID离开、新附属UID、一次洗牌、重复提交拒绝和恢复。该结果来自当前组合源码，独立红组原有14通过/1忽略仅保留作历史记录。

使用独立 `UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/audit-underground-100161b/next20-combined-build`，经 `tools/build-storage.mjs` 门禁运行 `cargo test --manifest-path rust-game/Cargo.toml --lib --offline --locked -- next20 exposure_ society_zone_ jc035_ batch20_ --nocapture`。默认Native特性，未启用society-fixtures；过滤词按libtest的OR子串规则匹配。

| 模块 | 最终通过数 |
| --- | ---: |
| batch20_tests | 27 |
| black_next20_tests | 21 |
| exposure_tests | 22 |
| jc035_tests | 15 |
| next20_blue_gray_tests | 13 |
| next20_cross_tests | 7 |
| next20_red宿主夹具 | 1 |
| next20_red_tests | 14 |
| society_zone_tests | 9 |
| 合计 | 129 |

首轮为127通过、2失败，保留原始日志。两个跨组声望/战功用例发现蓝灰待响应声明误用印刷能力筛选，漏校验授予的奖励能力，导致移除原地区UID的快照被接受。`e158839` 将蓝灰正式来源纳入运行声明与选择校验，仅准入绑定原公开角色及原地区UID的精确奖励程序；没有放宽普通执行帧。补充奖励程序篡改及真实徽章空秘社区覆盖后，修复定向3/3通过，原129项范围再运行全部通过。所有日志及SHA-256见[运行证据](next20-combined-native-20261021.json)。未扩大全量测试范围。

## 身份与静态边界

下一批未构建预留身份由92改为 `rust-v0.2.93-batch20-next`，pool仍为 `limited-v2.83-batch20-next`。ABI92留给第一批P2。本检查点保留已有77–89、91原件路由，未制造或冻结ABI92；等待owner提供P2正式原件身份和字节后再接入retained92。

初始整合静态结果：29个变更Rust文件解析成功；变更JavaScript文件语法检查成功；`git diff --check`无错误；193/183和原173记录一致性已直接核对。随后当前组合源码已完成Native lib类型检查与上述129项定向执行。旧冻核仍保留其173张目录断言。

P2源码和指定触发文字UX已导入，基线JC065 UX继续保留。本任务未重复开发UX，也未改总README；基线README第154行仍描述“当前候选ABI92”，需由owner随最终P2/93构建说明统一更新。独立克隆为节约磁盘省略大型Web/Sites夹具和原图工作副本，完整检查应由owner提供原始输入或调整本克隆稀疏范围后执行，不能以缺少副本略过验收。ABI93 WASM/Native服务制品、启动与Sites运行检查、当前UI验收、全量Rust测试、真实浏览器对战和独立复审不在本轮通过声明内。


## 后续指定差异导入

第一批最小P2修复 `a0b1b3da71887bde9aaed06f85dc74638d36d38c` 已导入为 `731269a`；触发文字UX `4a722341f7d302300351bb15300d3a17cba1575d` 已导入为 `af325e5`。P2只移除秘社附属宿主对已选秘社牌的要求，保留新增秘社标志和校验；Table的目标按钮允许空秘社区，同时保留暴露、隐秘和手牌上限显示。基线已含JC065 UX，没有重复导入。ABI93/pool83和既有ABI91路由不变，第一批ABI92尚未发布，仍未创建冻结输入。

P2的9项秘社区Native回归及JC051真实徽章空秘社区组合已通过；触发UX的界面验证由owner负责。本轮编译槽已释放，未推送或部署。owner报告第一批P2 v74已官方保存，Site source为 `fe182e656f35153be9225c0b17fbce708831b1ba`，正在等待独立补验且尚未部署；本轮没有创建宣称已发布的ABI92冻结身份。owner接手ABI93构建和后续oracle/界面用例，本记录不预报它们通过。
