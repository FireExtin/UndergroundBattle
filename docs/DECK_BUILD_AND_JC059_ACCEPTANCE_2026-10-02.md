# 自组卡组、JC059友方防御与地区公共图标总数：原生验收

本批版本为 `hegemony-pdf-v1 / limited-v2.2 / rust-v0.2.4`，私有状态 schema 仍为2。30个开放定义、五套50张预组和10张世界牌组成不变。`cards.json` 只改根 `engineVersion` 及 JC059 显示文字“其他本方角色”→“其他友方角色”。旧3及更早状态明确拒绝，不静默迁移；默认原生数据库为 `rust-game-v2.4.sqlite3`。旧核、进程及真实QA房间未修改。

## 构筑与冻结

新增有限校验模块 `rust-game/src/deck.rs`。原构筑要求为至少50张；通常按印刷名称跨ID合计最多3张；印刷“唯一”最多1张；场上“独有”不改变构筑上限。无限数量例外来自 typed `rule_traits.unlimited_copies`，“唯一”来自已确认 `keywords`，生产执行路径不按卡号、卡名或文本解析作例外。任意派系组合允许，没有新增两派限制。

秘社尚未实现，`societyId:null` 允许，非null明确拒绝。未知、未开放、地区及不属于玩家牌种类的条目拒绝；数量必须为正整数，合计使用 checked arithmetic。2048张及2048条目的容量护栏属于服务限制，错误文字明确说明它不是原作最大牌数。

`DeckDraft` 合同：

```json
{
  "id": "saved-deck-id",
  "name": "公开卡组名称",
  "description": "说明",
  "societyId": null,
  "cards": [{"cardId": "JC125", "count": 50}],
  "rulesVersion": "hegemony-pdf-v1",
  "cardPoolVersion": "limited-v2.2",
  "engineVersion": "rust-v0.2.4",
  "updatedAt": "2026-10-02T00:00:00Z"
}
```

三项版本必须存在且精确匹配当前核。canonical snapshot 合并同ID条目并按ID排序，裁剪id/name首尾空白；接受后拥有独立副本，保存稿后续编辑不能改变局内牌组。`Player.deck_snapshot` 为私有持久字段，有 `serde(default)`。自组创建/加入即冻结；开始时从 snapshot 实例化，预组在开始时也冻结；重开沿用该 snapshot。大厅换预组清除旧 snapshot，换自组建立新 snapshot，二者均取消 ready；开始后换组拒绝，整个状态保持不变。

原生 create/join 增加可选 `deckDraft`，原 `deckId` 继续可用；大厅命令为 `Action{kind:"deck",deckDraft}` 或旧 `Action{kind:"deck",option:deckId}`。加入日志为 `{"JoinWithDeck":{"name":"…","deck_draft":{…DeckDraft…}}}`，原 `Join` 不变。原始加入spec存入journal，重放重新验证并得到相同 canonical snapshot。

目录新增 `deckBuildRules`，包含 `minimumCards:50`、`usualNameCopyLimit:3`、`uniqueNameCopyLimit:1`、`copyLimitByPrintedName:true`、`factionLimit:null`、`societySupported:false`、`serviceCardCapacity:2048`、`capacityIsServiceLimit:true`；每卡 `deckCopyLimit` 为数字或null。对手仅获公开 `players[].deckName` 和通用 `deckId:"custom"`；`View.yourDeck` 只含本人spec。目录不含玩家snapshot，卡组名称本身为公开信息。`View.worldDeckCount` 是当前世界牌库实际剩余数量。

WASM源接口保留 `newGame`/`joinGame`，增加 `newGameWithDeck(...,deckJson,seedDecimal)`、`joinGameWithDeck(state,name,deckJson)` 与 `roomCatalog(state,seat)`。完整私有state仍是opaque字符串，seed仍为十进制u64字符串。这些接口的实际WASM构建与多版本路由由集成线程负责。

## JC059与公共总数

实际核对原图：[JC059 安全保卫部门](<../resource/ymsj-fun.github.io/cards/JC059 安全保卫部门.jpg>)。原文“所有其他友方角色”包含同队其他玩家。将 typed modifier 从 `OtherControlledCharactersDefense` 改为 `OtherFriendlyCharactersDefense`，执行按当前controller所属team判断，保留同地区、正面角色、来源不自加限制。没有卡号执行分支；费用、图标、卡池和其他声明不变。

`RegionView.iconsByTeam:[Icons;2]` 分别复用 `contest_counts(index,0/1/2)`，显示各队当前有效调查、战斗、势力。它包含重置暗藏者基础势力1、横置0、先手临时图标及已有持续修正。累计控制标志仍为独立 `influence` 字段。各席公共总数一致，暗藏者的身份和印刷字段投影规则未改变。

`RegionView.skipConfrontation:boolean` 直接公开现有 `Region.skip` 当轮状态，供界面提示跳过比较/收益。它不通过卡号、文本或日志推断；四席投影测试同时验证true/false一致性。

只读优先权核查实际查看《霸权》印刷P14/物理15：[原页渲染图](/tmp/hegemony-readonly-rule-review/霸权说明书-physical15.png)。当前 `push_stack` / `dispatch_frame` 不自动移交出牌者优先权；`pass` 才按两队让过移交或结算；`priority_default` 保留先手与后手行动步骤的规则。此次不改 `resolution.rs` 或优先权逻辑。

## 实际测试与构建

JC059先运行红测试：`friendly_defense_includes_teammate_but_not_enemy_other_region_hidden_or_source` 在旧过滤条件下队友防御得到1、预期2，0通过/1失败。修复后完整原生轮为 **40规则 + 9既有服务 + 5构筑/冻结 + 2友方/总数 = 56通过、0失败**。

其后按审查授权修正既有原生回执边界：先运行 `command_id_reuse_requires_same_complete_typed_intent_before_and_after_reopen`，相同座位/commandId改expectedVersion后错误收到旧ACK，实际红灯0通过/1失败。修复读取现有commands的seat/expected_version/action/response，解码完整typed Action进行相等判断；异座位、异版本或异动作返回409 `command_id_conflict`，不写状态、回执或journal。JSON键顺序、可选字段null/遗漏在typed decoding后规范化，同意图即使当前房间版本已推进或重启，仍返回原ACK。不引入hash表或schema迁移。该回归覆盖版本、动作kind和新增deckDraft字段差异，检查完整view、journal计数及audit不变。

这两项小修之后执行 **10服务 + 5构筑/冻结 + 2友方/总数 = 17定向测试通过、0失败**，重编native bins并刷新native oracle。包括前轮的40规则，本批累计57项不同测试通过；没有重复未改变的40项长测或WASM运行。

新增测试：

- `minimum_unlimited_multifaction_and_canonical_duplicate_entries`：无限数量、多派系、合并重复条目，独有与构筑唯一区分。
- `printed_name_groups_versions_and_unique_is_distinct_from_in_play_unique`：同印刷名称跨ID合计、唯一合计限制。额外印刷版本为明确的typed注册夹具，不向Game开放新卡。
- `malformed_disabled_overflow_versions_and_society_are_rejected`：少于50、0、未知、地区、unsupported、溢出、容量、版本缺失/不匹配及秘社。
- `frozen_snapshot_private_views_lobby_changes_and_started_rejections`：稿件编辑与局内隔离、不同组成的对手隐私、大厅换组/ready、开始后完整no-op、持久恢复、实际世界余量；重开入口使用明确的finished夹具，不声称自然全局胜负。
- `custom_deck_join_change_receipt_reopen_and_replay_keep_frozen_private_specs`：三项错版本加入不占席、不升版本；SQLite加入spec、换组、ACK回执去重、关闭重开、开始、隐私及seed/journal只读audit一致。
- `friendly_defense_includes_teammate_but_not_enemy_other_region_hidden_or_source`：四席同队增益，敌方/不同地区/暗藏/来源排除。
- `public_region_totals_include_team_temporary_hidden_ready_and_static_icons_without_identity`：四席公开总数一致，双队加总、临时图标、横置与暗藏、无敌方角色持续修正，暗牌投影保持过滤。

所有新构建均使用 `CARGO_TARGET_DIR=/workspace/.private-validation/hegemony-v024-target`，`TMPDIR=/workspace/.private-validation/hegemony-v024-temp`。纯lib、native bins、WASM ABI native check、`cargo fmt --all --check` 与 `git diff --check` 通过。

证据目录 `/workspace/.private-validation/hegemony-v024-evidence` 保留 `jc059-red.log`、`deck-and-aura-green.log`、`native-tests.log`、`pure-build.log`、`native-bins.log`、`wasm-abi-native-check.log`、`native-fixtures.log`、`fmt.log`，以及最终小修的 `command-intent-red.log`、`final-service-and-view-tests.log`、`final-native-bins.log`、`native-fixtures-final.log`、`fmt-final.log`。首次目标构建因/tmp空间不足未完成；仅迁移本批新目标到workspace。首次全量编译另有本批补可选字段时遗漏逗号的错误，修正后完整执行56项；失败日志均保留，没有将未执行轮称为通过。

最终原生SHA256：

- server `39d6774b6e628320d0db40cd436d5bf6346cf5c15392cd45b0edee6d1573d01a`
- audit `a9958ab02243860f5099667e4fb3d58d11d9da4ce341dd4c9c4facd04b1fde74`

已实际执行native oracle，生成该证据目录内 `native-fixtures.json`：**7场景、432状态步骤、1259本人视图、4个旧版本拒绝状态**。新增14步自组创建/加入/大厅冻结/开始轨迹，以及明确初始布局的四席友方防御与公共总数投影场景。保留原duel、teams、真实响应续体及两条JC058轨迹。完整私有state只用于内部对照，不应作为浏览器响应。

本批未构建wasm32、修改生成pkg、执行浏览器或互联网验收、重启服务、提交、推送或发布。上述native oracle供根代理实际WASM对照，不代表WASM已经通过。未扩展其他原作卡、秘社、隐藏控制权规则或计时政策。
