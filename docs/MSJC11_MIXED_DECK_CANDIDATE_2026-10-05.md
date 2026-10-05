# MSJC11 有限混搭牌组私有候选

基线游戏：`3b70a6fc96ffeba39ed6fa8acfcd257f6569c730`。源审阅计划：`bed5bce688b25fd4c921b48f0daf74d74a7f7ff3`。实施分支：`codex/hegemony-msjc11-mixed-deck-20261005`。本候选尚未保存或部署到 Sites；当前已发布 Site28 为 `63a58d3d651797d8bcd0717e9fa9bf3e357e8bfd`。父线程批准源计划后才开始此有限实现。

实际新引擎 `rust-v0.2.29-msjc11-mixed-deck-candidate`；池 `limited-v2.26-msjc11-mixed-deck-candidate`；规则身份仍为 `hegemony-pdf-v1`。普通定义仍 89 张（79 玩家牌、10 地区），原五预组和原五秘社保持原义。唯一新增秘社 MSJC11；原图总数从 94 到 95。冻结 accepted28 的五个 ABI 文件保持已发布 WASM `a8a928e12e239161c082671a6008b64775c7b8e9e0181e7d765db23af8bb32e8` 字节；所有更早冻结核保持原样。

## 已实施行为与源对应

实际查看的整张原图 `resource/ymsj-fun.github.io/cards/MSJC11 S．P．T执行部.jpg` 为 289×404、239506 字节，SHA256 `df9964832e48bfef57bcaa0d5f4db67ce2ab9fe0e371ffecc2736c1728b1b853`；公开扫描为同一原始字节。金色独有秘社，绿、企业/部门、副题直属特遣队、起手六张，没有印刷费用、魔法领域、忠诚、防御或额外印刷图标。

构筑持续条件只查目录中印刷字段：允许绿和中立牌的各普通种类；异色仅允许印刷人类角色且永久战斗图标大于零。战斗场上强化、领域、白底先手加成不影响构筑。保留至少 50、同名合计三张及既有无限牌例外；秘社不计入普通 50 张。绿黑和绿红各 50 张导入文件随审阅包提供，两者没有额外加牌。

两种能力都是标准、可响应、真实三费用加横置本席秘社；费用不跨队友资产。本回合赋予用既有回合修正表中的两个省略默认值字段；不新增永久关键词计数器或通用关键词解释器。新 Op 只接受 MSJC11、固定能力 key、固定费用、单个固定目标和单个固定操作，模式、嵌套、复制操作、异卡使用及更换目标关系均闭合拒绝。

杀伤能力沿原文没有 owner、controller、队伍或同地区限制；可选择既有目标保护规则允许的任意正面角色。杀伤按印刷值与各赋予叠加，仅正面未横置且实际战斗图标大于零的本团队参战角色贡献伤害。非战斗角色可以被合法赋予，但不会因此参加战斗或额外产生杀伤奖励。

地区撤回按结算瞬间一次快照：正面角色、当前 `controller == actor`、当前黑底战斗大于零。黑底查询为 P（当前永久部件）＋O（普通黑底本回合加成）＋XQ43 实际转换的 T；持续到回合末不会把 O 当白底，先手本身不会把未转换白底变为黑底。队友不属于“本方”。之后进入或之后才得到黑底的实例不补领；已领的同一实例移动或失去图标仍保留至回合末/离场。隐藏、离场、再公开/重入产生的新实例不继承。

MSJC11 地区目标声明真实捕获 `region_instance`，结算必须存在且等于现地区实例；替换地区、遗失捕获会取消已付款效果，不退费。已有 JC118 等非地区附属操作仍使用此前地区 index 语义；没有将原本仅附属适用的 guard 悄悄泛化至旧操作。

赢区沿现有真实 Win 和附属整批归还流程：赋予撤回的借入角色回 owner 手牌并换实例；不属于 actor-controller 的队友牌回自己的牌库底；BQ022 按自己的 owner/原附属目的地处理。前端仅正面地区牌显示“本回合杀伤 N / 本回合撤回”，所有暗藏视图连当前 controller 都省略新增动态字段。

地区赋予的一次快照是父线程明确批准的本项目裁定。已查看的现有 FAQ 未定位到 MSJC11 专条；不得把此裁定写成找到官方 FAQ。

## 验证边界

两条四席完整 50 张自然 Room 指令路径（绿黑/绿红）由正常新局、四席加入、准备、开局、调度、资产、JC016 部署、JC131 检索及真实付费发动组成。每席都付费发动两种能力，队友/敌方资产不变；同名绿牌由不同 controller 操控，在同地区/异地区关系均实际出现；下回合新赋予消失。共 2551 次指令转换、10192 次投影视图。它们是离线原生/WASM 指令验证，不是公网浏览器自然 UI 验收。

全部 28 个边界初始布局明确标为 synthetic；只在初始检查点准备板面/手牌，三种取消样例另明确准备已经合法付款但目标被删除/替换/捕获损坏的帧。之后移动、图标授予/失去、隐藏/再公开、战斗、赢区与清理都走真实 Room 指令，未在中途注入状态。JC008/JC005、白底与灵体等准备牌可能不在此席 MSJC11 构筑中；这是现有已接纳操作的孤立边界，不冒充该牌组自然可达。

没有已接纳灵体能实际提供正临时战斗 T；转换正 T 战斗的公式通过共享纯部件原生向量证明，不能称为完整自然/WASM 可达正例。实际 WASM JZ58＋XQ43 零战斗反例已覆盖，不会把领域当战斗。

六个原生变异分别破坏地区捕获、controller 选择、普通黑底加成、零战斗杀伤排除、杀伤叠加与换实例清理；六个精确命名测试都运行一项并断言失败，原字节全部恢复。首个 admission-red 用错 `--exact` 名称运行零项，不算红证据；后续完整名称真实未知 MSJC11 红与实现后绿已记录。

accepted28 和 current29 各自正常四席房间在独立 WASM 内存恢复，四席视图和下一完整 Room 指令一致，跨版本 view/apply 都明确拒绝。current29 另外五个有效赋予检查点（叠杀伤、后来进入、失去图标、移动、过期）分别复原四席视图及下一指令一致。都是本机独立 QA，无公网旧房凭据、朋友房访问、D1/API 代打或浏览器 DOM 注入。

公网 Site28 已发布；此前新匿名 `/api/catalog` 被站点边缘 403 `error code: 1010` 阻挡。未使用 get_site 的 bypass token、替代路由或旧执行器凭据；没有新增公网浏览器或正常刷新验收。Site28 的发布成功不代表本 current29 候选已经发布或公网验收通过。

## 最终检查结果（保留首轮失败事实）

实际 WASM：2063840 字节，SHA256 `78b6f630ecef5a35b50cd07852374566cd1063d55af15a3802e54df7cc27b19a`。实施提交 `025460a0c8de398422ef12c432f64c113a558212`；本报告与审阅证明另作交付提交，不改变被测生产源码/ABI。

- Rust 唯一一次全量首轮 **385/390**；五个旧秘社数量断言失败。仅定向补跑原五项均绿，不称为一次全量全绿。12 个新机制定向测试已通过，六个精确原生变异均真实断言失败并恢复。
- 前端唯一一次全量首轮 **430/438（53 文件）**；八个旧扫描数量断言失败。仅定向补跑八个受影响文件 **27 项全绿**，不称为一次全量全绿。
- 原六项 native 特性集成目标 **32 项全绿**；前端类型/生产构建通过。
- 唯一一次当前默认生成 **433 个完整样例**，随后实际新 WASM **433/433**：**24403 转换 / 96975 投影视图 / 1 quote / 319 拒绝指令 / 42 拒绝新局构筑**。
- 独立新增索引 **32/32** 实际新 WASM：**3109 转换 / 12412 投影视图 / 5 拒绝指令 / 14 拒绝构筑**。32 个完整正文共 **348919621 字节**，随包压缩保留，逐个验长/SHA；包含两条正常四席路径、两项构筑和 28 个合成边界。
- 旧 **401 个完整正文**流式读已验 SHA 归档，只有两种版本身份字符串替换，其余逐字节等同。旧 **89 普通定义、五秘社、五预组、94 旧扫描、131 冻结文件、48 保护文件**保持原样；新冻 accepted28 五 ABI 也保持真实 old28 字节。`docs/factions/card-specifications.json` 始终只读且原 SHA 不变。Site checkout HEAD `63a58d3d651797d8bcd0717e9fa9bf3e357e8bfd` 干净。
- 默认 native **590.629s / 4531668KiB**；默认 WASM **116.296s / 1496716KiB**；新增32实际WASM **16.111s / 947200KiB**；旧正文/源码核对 **12.381s / 862968KiB**。所有列出的有效检查阶段 OOM/oom_kill 增量零；历史基线计数仍为6/2。原始命令与逐阶段资源记录在 validation-summary。

五处 Rust 修正（只改断言）：

| 文件 | 精确测试 | 修正 |
| --- | --- | --- |
| `rust-game/src/jc008_tests.rs` | `jc008_tests::jc008_printed_admission_and_finite_modifier_program` | five-item society ID list → same five plus MSJC11 |
| `rust-game/src/msjc06_tests.rs` | `msjc06_tests::msjc06_original_whole_card_and_finite_existing_search_program` | societies.len() == 5 → societies.len() == 6 |
| `rust-game/src/msjc07_tests.rs` | `msjc07_tests::msjc07_original_whole_card_and_finite_existing_search_program` | societies.len() == 5 → societies.len() == 6 |
| `rust-game/src/msjc08_tests.rs` | `msjc08_tests::msjc08_original_whole_card_and_finite_existing_search_program` | societies.len() == 5 → societies.len() == 6 |
| `rust-game/src/society_msjc09_tests.rs` | `society::printed_tests::msjc09_printed_registry_deck_and_atomic_start_keep_four_seat_privacy` | societies.len() == 5 → societies.len() == 6 |

八处前端修正（只改原图总数断言）：

| 文件 | 测试 | 总数 |
| --- | --- | --- |
| `web/src/game/JC005Scan.test.jsx` | retains Site24 JC005 while adding the isolated JC008 candidate and retains the exact printed spell metadata and original bytes | 94 → 95 |
| `web/src/game/JC008Scan.test.jsx` | admits only JC008 and retains its actual printed metadata and original JPG bytes | 94 → 95 |
| `web/src/game/MSJC01Scan.test.jsx` | retains actual MSJC01 original, metadata, two abilities and two real admitted yellow unique cards | 94 → 95 |
| `web/src/game/MSJC06Scan.test.jsx` | retains actual MSJC06 original, metadata, two abilities and the real admitted white unique card | 94 → 95 |
| `web/src/game/MSJC07Scan.test.jsx` | retains actual MSJC07 original, metadata, two abilities and two real admitted black unique cards | 94 → 95 |
| `web/src/game/MSJC08Scan.test.jsx` | retains actual MSJC08 original, metadata, two abilities and the real admitted purple unique card | 94 → 95 |
| `web/src/game/SocietyScan.test.jsx` | keeps the sole real society unchanged while the local JC008 candidate adds one ordinary card | 94 → 95 |
| `web/src/game/YellowSearchBatchScan.test.jsx` | admits only the two approved original yellow unique cards with their printed numbers and loyalty | 94 → 95 |


## 审阅与复现

`--slice-msjc11` 含所有 32 个新增完整样例（两项构筑、两条自然、28 边界）；这些也确实追加到默认验证，原 401 为前缀。`--msjc11-natural-cases`、`--msjc11-boundary-cases` 用于开发定向检查。审阅包的 `new32-native.json` 可直接使用包内完整正文与 current ABI 比对；默认全套索引引用的旧 401 正文仍完整保存在验长/验 SHA 后归档的私有证据目录。

通常命令：`cargo test --locked -p hegemony-server --no-default-features --lib`；原六项原生特性集成目标；`bash rust-game-wasm/build.sh`；`cargo run --locked -p hegemony-wasm --example native_fixtures -- OUTPUT.json`；`node --expose-gc rust-game-wasm/tests/compare.mjs OUTPUT.json`；`npm --prefix web test -- --maxWorkers=1`；`npm --prefix web run build`。Rust1.90、wasm-bindgen0.2.104、Node24，测试/开发去 debug、去增量、编译两任务；包装附全部实际命令、资源记录及失败修正边界。
