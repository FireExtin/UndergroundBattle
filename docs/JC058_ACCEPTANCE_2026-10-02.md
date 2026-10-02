# JC058 坚毅的刑警：第一张增量整卡原生验收

本批版本为 `hegemony-pdf-v1 / limited-v2.2 / rust-v0.2.3`，私有状态 schema 仍为2。开放定义从29增至30（26玩家牌、4地区牌），只新增 JC058。`keepers` 仍为50张：JC125 从17减至14，加入 JC058×3；其他四套牌组及10张世界牌组成不变。原生默认数据库改为 `rust-game-v2.3.sqlite3`，旧版本状态明确拒绝，不作静默迁移。

## 原稿与声明

实际查看的原卡：[JC058 坚毅的刑警](<../resource/ymsj-fun.github.io/cards/JC058 坚毅的刑警.jpg>)，印刷编号058/135，图像 SHA256 为 `6585e89991049cb05e75db43143320fd12bf6be781847648d551cb433e85119a`。

角色；人类/警察；3费；灰色忠诚×1；国家机构；无魔法领域；永久调查、战斗、势力各1，临时图标全0；防御1；无关键词或独有。原文为“现身触发：消灭本地区的目标暗藏者。”旧 FAQ 的“进场、敌方”措辞不覆盖此新版原图。

原《霸权》印刷P16/物理17“发动”说明操控者可以选择发动行动或触发能力，因此可拒绝这张未写“必须”的触发。该页“来源”亦说明已加入堆叠的能力不因来源被消灭而失效。实际原页：[本地渲染图](/tmp/hegemony-readonly-rule-review/霸权说明书-physical17.png)。暗藏者、付费现身和进入堆叠采用所选《霸权》印刷P11/物理12的既有规则。

生产只增加声明：`Reveal → Board / Hidden / Any / SourceRegion → Destroy(Target(0))`，沿用默认 `Respondable`、空追加费用、单目标和可拒绝选择。未新增 opcode，未改 `resolution.rs`、`model.rs` 或按卡名执行的分支。卡牌的派遣、秘密派遣、现身费用与忠诚均复用原流程。

## 原生规则验证

新增三条复合回归，使用明确的初始布局夹具，之后通过真实 `Action` 执行声明、支付、让过、响应和选择。这些测试不声称布局由自然洗牌发出。

- `detective_only_reveal_triggers_once_can_decline_and_handles_exhausted_or_no_target`：正面派遣不触发；现身先结算牌、再声明一次能力；可拒绝；横置状态保留且不阻止触发；无合法目标时完成，不制造待选。
- `detective_targets_all_sides_same_region_without_private_leaks_or_hidden_death`：2V2同地区己方、队友和敌方暗藏者均可选；其他地区、正面角色及已声明现身而进入堆叠的牌不可选；非法选择完整无变更；公开目标摘要脱敏，队友和敌人看不到选择者的选项；两队四席全部让过只结算堆顶。被消灭的暗藏者进入拥有者墓地并获得新实例；隐藏 XQ12 不发动印刷死亡触发。
- `detective_target_reveal_cancels_without_refund_but_source_death_keeps_effect`：目标响应付费现身时，声明即移走旧隐藏实例，原目标失效，随后刑警能力取消，双方已支付资产不退；另一分支由真实 JC091 谋杀消灭刑警来源，已声明的能力仍正常消灭原隐藏目标。保存并恢复待响应 frame 后结果一致。

现有目录测试同时验证 active catalog 与绑定集合相等、30定义、26玩家牌、五套50张、只有 keepers 含 JC058×3、其 JC125 为14，以及香港仍处于隔离边界。

## SQLite 与房间目录

新增 `detective_reveal_choice_and_bound_frame_restore_without_repayment_or_private_leaks`。一次性初始房间布局在首条命令前安装；后续全部使用认证、版本化命令，不修改中间数据库状态。

覆盖现身的3资产费用、横置保留、触发目标待选恢复、绑定后的未结算 frame 恢复、原现身及选择回执去重、错误座位无法选择、私有选项与公开摘要脱敏、隐藏 XQ12 消灭后不触发死亡能力，以及只读 seed/journal replay audit 的 `matches=true`。待选持久化的是 `Declaration`；待响应持久化的是已绑定 `ResolutionFrame`，其 guard 尚未执行、cursor=0，触发无追加费用。

新增唯一 HTTP 路径 `GET /api/rooms/{roomId}/catalog`，必须提供该房间座位 Bearer token；原生数据库隔离版本，因此返回该原生核的当前 catalog。`room_catalog_requires_a_token_for_that_room_and_returns_current_pool` 验证缺失、错误和其他房间令牌均401，两席有效令牌均200，读取不改变房间版本。多核 Worker 的目录路由由集成线程负责。

## 构建与对照材料

全部使用隔离 `CARGO_TARGET_DIR=/tmp/hegemony-v0.2.3-target`。实际结果：`cargo test --locked -p hegemony-server` 为 **40规则 + 9服务 = 49通过、0失败**；native bins、无 native feature 的 pure lib、`cargo fmt --all -- --check` 与 `git diff --check` 均通过。

证据日志：

- `/tmp/hegemony-v0.2.3-native-tests.log`
- `/tmp/hegemony-v0.2.3-native-build.log`
- `/tmp/hegemony-v0.2.3-pure-build.log`
- `/tmp/hegemony-v0.2.3-native-fixtures.log`
- `/tmp/hegemony-v0.2.3-fmt.log`

原生二进制 SHA256：

- server：`83c5b57fff78146411094d7bde85e9c67e3ab3b82b630bf88d3a15530866e6d9`
- audit：`9fdd87bd621e16533b2d2e7052291f6ec117f6f1e48b56021e1f16bfa4f9923c`

已实际执行原生 oracle，输出 `/tmp/hegemony-v0.2.3-native-fixtures.json`：5场景、417状态步骤、1205本人视图、3个旧版本拒绝状态。新增两条11步轨迹分别为 `detective-target-reveal-invalidates-original-instance` 与 `detective-source-death-independent-hidden-destroy`，包含触发待选、锁定 frame、响应、恢复及最终旧引用拒绝。JS对照继续以 opaque state 字符串传输，不解析或重串行完整私有状态。

本批作者没有构建 WASM、重启服务、提交、推送或发布。上述 oracle 是供根代理执行实际 WASM 对照的原生预期；本记录不将其称为 WASM、浏览器或互联网验收成功。后续集成结果由对应验收证据记录。

## 范围限制

本批没有新增其他原作卡、机制或模式。当前开放池没有在响应中横移 JC058 的牌，本批不裁决该未核定的 `SourceRegion` 场景；也不开放多目标部分失效、强制触发、附件、封印或控制层等新机制。旧房间继续使用固定旧核，不能把新增 keepers 组成应用到旧局。
