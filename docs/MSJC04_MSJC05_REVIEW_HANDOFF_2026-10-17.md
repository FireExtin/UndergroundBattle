# MSJC04 / MSJC05 独立批次：WIP

本批基线为既有进度分支 `codex/stage-pass-progress-20261016` 的 `000c0846bd90dc23310813b953551550e850124e`。ABI82/v64 的发布回执已经保存，本批没有保存新 Sites 版本、部署、合入 main 或迁移旧桌。候选身份为 ABI83 / pool75；完整回归与父串行独审尚未完成。

## 已实现的候选范围

- MSJC04 鸣钟教派：红、金名独有、秘密教派、副题“末日钟的侍奉者”；MSJC05 国家机构：灰、金名独有、机构、副题“政府权力”。两张均无印刷费用、起手6、本色普通牌至少25，秘社额外设置、不计玩家牌组50张及本色25张。
- 两张的完整行动程序：支付3资产并横置，结算时本人队伍具有先手则抓1；支付4并横置，检索本人牌库一张相应印刷颜色且金名独有的牌，展示、换新 instance 入手、洗牌。第二能力每局次数在成功付费发动时消费，空搜、取消、回合重置、换 instance、恢复均不返还，重开新局清空。能力资格不扫描私有牌库命中情况。
- 受限卡池真实红色独有目标为 JC052「终焉的诗寇蒂」。当前14张灰色普通卡均非独有；灰色没有伪造目标，依照既有检索程序正常空搜、洗牌并消费支付与次数。
- 入席页可直接选择已准入秘社，展示构筑失败的实际原因；选择后使用完整牌组快照，既有预组不足25本色时不能入席，需在收藏与组牌中完成合法构筑。保留全部既有预组，不擅改其组成。
- 沿用既有命名/改名、本浏览器 localStorage 保存、sessionStorage 标签内编辑恢复和完整牌组入桌快照；新增显式“载入当前入席快照”，存在未保存编辑时先保存，再载入，避免静默覆盖。恢复旧席保留桌内秘社。
- 原始图片、客户端 URL 和扫描清单均已接入。普通147定义、世界牌库、五套预组及原8秘社保持原值，仅新增2秘社。全目录为137玩家卡＋10地区＋10秘社＝157定义。

## 来源及必须明确复审的解释范围

已再次查看两张原图：MSJC04 SHA-256 `a6ba66c7fb73acdbe4cab001e6330d0f339c46e12b6d3e714307196e9fd23cf3`；MSJC05 `d71e7c3df37a7c857cfb74c9988261ad722c5f3012ca49c75151fd407dab59f3`。已查看《霸权说明书》PDF物理第5页/印刷P4的秘社设置、独有和起手规则。

来源研究档案 `docs/factions/card-specifications.json` **仍原样保留**两张的 `openQuestionIds:["conditionalActivation"]`、`implementationDesignStatus:needsRuling`、`gameAcceptance:notAccepted`、`strategyUiPlaytest:notRun`。没有把候选测试改写成来源验收。

[MSJC09 历史候选说明](MSJC09_SOCIETY_PRIVATE_CANDIDATE_2026-10-03.md)明确写过父裁定“仅用于MSJC09，不泛化其他条件句”；其后已准入的 MSJC01/02/03/06/07/08 均复用同一条印刷条件及结算程序。本批**提请父串行复审明确批准**将相同有限项目解释用于这两张逐字同义的行动：冒号前为费用；先手是结算条件，后手可付费但不抓、不退款。此处是项目候选解释，绝非声称存在官方FAQ或已解除档案疑问；不向其他条件句扩散。复审前不发布本批。

## 旧桌与原始见证

修改 Rust 之前已核对并冻结真正发布于 v64 的 ABI82 原字节，且和官方 provider source `74b1e8f2e6a0c4feb403520554a9275f52be82fd` 的 Worker payload 相符：WASM 3,165,174 bytes / `1479c3d2ac98fa0f3c0aefff4eba409f2f84a28f437833cbb0a654332b752d37`，JS `fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb`。原 Native82 fixture及128编译输入绑定未重签，历史检查改用只读冻结输入。ABI77–82全部保留，候选独用83身份；禁止旧桌静默迁移。

## 当前验证状态

精确机器状态见 [checkpoint-status.json](evidence/msjc0405-2026-10-17/checkpoint-status.json)，日志索引包含首次测试错误及修正后结果，不合并累计：

- 聚焦 Native18通过；Native83/WASM83六场景39命令（6拒绝）、180四座投影、129编译输入的完整opaque状态、journal、结果及所有视角一致。
- 冻结ABI82与七核路由10通过；来源55通过；聚焦前端4文件35通过；两套TypeScript检查通过。
- 完整Native库回归正在运行；完整前端/HTTP回归、七核production build、实际production GameApp本地浏览器流程和父独审未完成，不能称全绿或可发布。
- 首轮失败为新增冻结测试的fixture文件名错误，以及Vitest静态改写Node URL；均仅改测试加载路径。新增来源测试曾错误要求无档案疑问，已改为明确保留 `conditionalActivation`。未重写原始Native82或Native83观察。

公网仅已部署ABI82/v64。12:25UTC普通浏览器仍为 `ERR_CERT_AUTHORITY_INVALID`，没有绕过证书、进入真实牌桌或声称本批线上试玩。卡组保存的现有范围另见 [存储说明](DECK_NAMING_AND_STORAGE_STATUS_2026-10-17.md)。
