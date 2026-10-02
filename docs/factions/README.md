# 官方颜色派系、秘社与完整待研目录核验（第一阶段）

审计日期：2026-10-02。源码基线：`286ad83ef5a54bc0caed26199ca4cfea269314bd`，来自当时最新的 `origin/codex/hegemony-playable`。独立工作分支：`codex/faction-coverage-audit`。本次只新增此目录、两份独立数据和 `tools/factions/`；没有修改生产卡池、规则核、React、房间或发布配置。

归档说明书明确列出 **八种颜色派系，另有褐色中立**。现有五副 50 张牌是受限卡池的自组预组，不能作为官方派系导航。完整归档的 **685 条记录已全部纳入待研索引**，但这不是 685 张可玩牌，也不是 685 张已核清规则的牌。

本阶段完成全部颜色、归档秘社记录、组牌概念和当前源码边界的核验。截至 B09，163 条记录至少查看过原图，其中 141 条完成锁定观察版本的全部牌面游戏字段，另 1 条先手标志完成非卡牌来源参考，21 条仍属部分/阻塞核验；509 条有原图但尚未逐张核验，13 条缺少同 ID 独立原图。完整来源记录共 142 条，尚有 543 条未完成。完整字段核验与原作机制/引擎/策略试玩验收分别计数，没有声称完成全卡规则转写或策略试玩。

## 输入、证据与输出

规则事实以原始 PDF 的渲染页面、原始卡图和四张扩展说明图为准。`resource/ymsj-fun.github.io/cards/cards.json` 的 685 条数据仅用于定位和建立待研目录；其中 `set`、`set-id`、类型、关键词、颜色、规则文本不能自动升级为核准值。抽取文字也只辅助定位。

输入：

- [霸权说明书](../../resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf)：24 个 PDF 页面；正文印刷页码通常比 PDF 页面小 1。
- [隐秘世界玩家指南](../../resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf)：29 个 PDF 页面；正文印刷页码通常比 PDF 页面小 2。
- [隐秘世界规则手册](../../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf)：21 个 PDF 页面；正文印刷页码通常比 PDF 页面小 2。
- [隐秘世界勘误及释疑](../../resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf)：本次引用的印刷页码与 PDF 页面一致。
- 四张扩展原图：[序曲](../../resource/ymsj-fun.github.io/public/docs/序曲说明书.jpg)、[间奏](../../resource/ymsj-fun.github.io/public/docs/间奏说明书.jpg)、[帷幕之后](../../resource/ymsj-fun.github.io/public/docs/帷幕之后说明书.jpg)、[星光俱乐部](../../resource/ymsj-fun.github.io/public/docs/星光俱乐部说明书.jpg)。
- 原始卡图在 `resource/ymsj-fun.github.io/cards/`，优先使用同 ID 的完整 JPG，没有用压缩图或另一版同名图代替缺失证据。

输出：

- [faction-coverage.json](../../rust-game/data/faction-coverage.json)：八色及中立、38 条秘社记录、组牌规则、模式、53 项共享机制、原始证据清单及源码锚点。
- [card-reviews.json](card-reviews.json)：人工原图核验的部分字段、能力概述、确认的机制及剩余核验项。
- [card-research-index.json](../../rust-game/data/card-research-index.json)：完整 685 条待研索引，保留所有版本/形态；原始定位字段与核验字段分开。
- [card-specifications.json](card-specifications.json) 与 [B01-evidence-package.md](B01-evidence-package.md)：原图锁定版本的全游戏字段、能力边界、未解疑点和待执行验收场景。累计141条牌面完整、1条非卡牌参考完整、5条未清字段阻塞，分批原稿和验收边界另见各批evidence-package文档。
- [B02-evidence-package.md](B02-evidence-package.md)：JC058、私密检视、墓地正面打出、声望赋予、操控/结附等20条原图规格，包含19条完整和1条blocked。
- [B03-evidence-package.md](B03-evidence-package.md)：剩余19条现有牌池完整原图规格；校正JC059友方范围及护卫伤害分配语义。
- [B04-evidence-package.md](B04-evidence-package.md)：14条基础秘社完整规格，包含构筑例外、额外费用、双目标移动和底抓语义。
- [B05-evidence-package.md](B05-evidence-package.md)：霸权与帷幕八条秘社规格；七条完整，梦魇主母的额外8符号未明而blocked。
- [B06-evidence-package.md](B06-evidence-package.md)：15条共享机制邻接牌；护盾生成、先兆、警犬、沙漏冲突、闪烁与卡牌终止边界。
- [B07-evidence-package.md](B07-evidence-package.md)：17条生成指示物与白紫邻接牌；解开沙漏地区标志计数，修正触手目录ID，保留TK007非卡面标志图的独立边界。
- [B08-evidence-package.md](B08-evidence-package.md)：剩余13条扩展秘社/转变面；38条秘社中35条来源完整，3条保留既有阻塞。
- [B09-evidence-package.md](B09-evidence-package.md)：世界、任务、遭遇与其他转变面的29条规格；新增武陵源副名称阻塞，先手标志按非卡牌类型独立完成。
- [source-recovery.json](source-recovery.json)：缺图补证搜索、墨尔本/核心区 PDF 旁证、网络访问结果及13条隔离名单。
- [implementation-plan.md](implementation-plan.md) 和 [acceptance-groups.json](acceptance-groups.json)：共享机制依赖、小步补全顺序和未来验收门槛。
- [tools/factions](../../tools/factions)：可复现生成器、独立验证器与防止误解锁的回归测试。它们不读取规则文本来执行游戏，也不写生产卡池。

`faction-coverage.json.evidence` 为每个被引用的原始文件记录路径、SHA-256，以及 PDF 的**印刷页码和从 1 起算的 PDF 页面**。卡牌的 `evidenceId` 关联同一清单。原图核验的判定是人工判断；自动校验只能验证关联、状态和哈希，不能证明人工阅读正确。研究索引的 `fullCardVerification` 与 `fullSpecRef` 另行跟踪整卡状态，不能用 `primaryImageReviewed` 数量冒充整卡完成数。`sourceRecordVerification`/`sourceSpecRef`另兼容非卡牌参考；`completeSourceSpecifications`只计牌面，`completeNonCardReferences`只计非卡牌，`completeSourceRecords`才是两者合计。

## 官方颜色及实际覆盖

名称/颜色的共同原始证据：霸权说明书印刷 P1 / PDF 第 2 页、玩家指南印刷 P1 / PDF 第 3 页；规则手册 P7 / PDF 第 9 页明确说中立牌没有派系。下列策略概述来自典型牌，**不是颜色独占能力，也不是固定组牌限制**。

| 颜色 | 官方名称 | 典型原图证据 ID | 当前玩家牌定义数 | 源码覆盖结论 |
| --- | --- | --- | ---: | --- |
| 黄 | 帷幕守望 | JC002、JC003、JC006、MSJC10 | 3 | partial：有限调查/横置；领域控制、秘社等未齐 |
| 绿 | 猎魔人 | JC014、JC016、JZ08、MSJZ01 | 3 | partial：有限机动/杀伤；装备/秘社等未齐 |
| 蓝 | 王座会 | XQ12、JC028、JC032、MSJC12 | 1 | partial：只有一个玩家定义；鲜血、奴仆体系等未齐 |
| 红 | 鸣钟教派 | JC042、JC049、MSJC13、MSJZ03 | 2 | partial：有限牺牲/降费；时间、毁灭等未齐 |
| 灰 | 国家机构 | JC056、JC059、JC063、MSJC14 | 3 | partial：有限公开/防守/暗藏者针对；锁定等未齐 |
| 白 | 圣贤 | JC070、JC076、JC077、MSJC15 | 0 | unimplemented：声望、灵体等尚未开放 |
| 黑 | 方碑序列 | JC086、JC092、JC085、MSJC16 | 4 | partial：有限墓地回收；墓地直接打出等未齐 |
| 紫 | 梦境行者 | JC099、JC100、JC108、MSJC17 | 0 | unimplemented：封印、梦境/梦魇体系等尚未开放 |
| 褐 | 中立（无派系） | JC125、LC19、XQ49、MSJC09 | 9 | partial：有限通用牌；不是第九个颜色派系 |

这些原图路径可在数据的 `evidence["card-<ID>"]` 查到。不能把 Lobby 中 `watchers` 的蓝色、`hunters` 的红色、`keepers` 的金色、`reclaimers` 的紫色主题映射成官方颜色；它们是 UI 主题。

**五种魔法领域**是心灵、神圣、星辰、死亡、血脉（规则手册 P7 / PDF 第 9 页），与八种颜色分属不同概念。当前有限代码/数据中有“血”“鲜血”的领域别名，展示官方名称时应注明映射，不能把蓝色或红色直接等同于血脉。

卡图中的其他组织徽记、卡牌子类别及定位数据的 `society` 标签也不能新增顶层颜色。仙灵、昆仑、炼金师学会、独立、破茧者等应保留为待核的组织/归属检索词。举例：绿卡 BQ016、红卡 BQ043、黄秘社 MSCQ01、红秘社 MSJZ04 上的徽记与基础八色徽记不完全相同；颜色、组织徽记和秘社牌须分别展示。仅凭 locator 的标签还不能核准所有组织归属。

## 混搭、构筑和官方预组

1. **基础版快速组牌**：任选八色中的两个不同派系，把基础版各自完整的 25 张半副洗混成 50 张。原证据：玩家指南 P6 / PDF 第 8 页、规则手册 P6 / PDF 第 8 页。28 种无序两色组合是这条规则的算术推论，不能当作已实现的 28 副数字牌组。仅持有一套实体基础版时，两位玩家不能同时取用同一套派系牌；不能把这个实体库存限制额外施加到数字游戏。
2. **一般自定义构筑**：至少 50 张玩家牌；同名通常最多 3 张；可以使用任意数量颜色，之后再应用所选秘社的具体限制及卡面例外。不是“一律只能两色”。秘社可选且另放，不洗入牌库、不计入 50 张；起始手牌使用秘社牌数字。原证据：玩家指南 P12 / PDF 第 14 页、P22 / PDF 第 24 页；霸权说明书 P13 / PDF 第 14 页。JC125 无知路人原图提供同名数量例外。同名不同版本的组牌数量仍按名字处理，但版本不能因此被合并成同一规则定义。
3. **霸权官方预组**：说明书 P13 / PDF 第 14 页列出四套：貘组 MSJC10、龙之欢宴 MSBQ01、圣甲虫药业 MSBQ03、修格斯 MSBQ02。MSBQ04 天闻山庄不是该页的第五套官方预组。

当前 `Game::new` 只接受运行时 catalog 中的牌组 ID，没有自定义卡牌列表或完整秘社构筑检查入口。固定 50 张牌组的测试不能证明任意两色、三色、秘社限制或官方四套牌表可玩。

| 现有牌组 ID | 名称 | 实际 50 张构成（按颜色计复制数） |
| --- | --- | --- |
| watchers | 帷幕调查 | 黄 9 + 中立 41 |
| hunters | 公路猎手 | 绿 9 + 中立 41 |
| keepers | 城市守卫 | 灰 9 + 中立 41 |
| reclaimers | 墓地回声 | 黑 6 + 蓝 3 + 中立 41 |
| responders | 牺牲与响应 | 红 5 + 黑 4 + 中立 41 |

## 秘社记录的边界

本归档核准的是 **38 条秘社牌记录**：37 个 `MS…` 记录和盛夏王廷的转变面 `ZHMSJZ02`。其中 MSJZ03 末日钟明确不能作为初始秘社；ZHMSJZ02 凛冬王廷是转变面。其余卡面允许初始使用，不代表其所属产品/模式在当前引擎适用或已实现。当前所有秘社都 `unimplemented`，没有秘社区域/选择/起始手牌覆盖或完整能力入口。

下面表格的名字、颜色、起始手牌、构筑限制及能力概述均检查过该行原图。概述用于研究导航，不能直接当可执行规则；代价图标、目标范围、强制/可选、完整数值和版次仍需逐项核准。MSBQ03 第一行动原图费用为 1，定位 JSON 为 2，已经记录差异，禁止从 locator 生成能力。

<!-- Society inventory follows, derived only from manually reviewed coverage. -->

| ID / 原图 | 名称 | 颜色 | 起手 | 角色 | 构筑限制（概述） | 能力（概述） |
| --- | --- | --- | ---: | --- | --- | --- |
| [MSBQ01](<../../resource/ymsj-fun.github.io/cards/MSBQ01 龙之欢宴.jpg>) | 龙之欢宴 | 蓝 | 6 | 卡面允许初始 | 蓝/绿/中立；鲜血不限同名数量。 | 初始检索鲜血封印秘社；角色离场时鲜血结附改封印。 |
| [MSBQ02](<../../resource/ymsj-fun.github.io/cards/MSBQ02 修格斯.jpg>) | 修格斯 | 灰 | 5 | 卡面允许初始 | 灰/红/中立；选修格斯核心；至少12张命令。 | 失控/名誉点降命令费用；赢三张任务另获胜。 |
| [MSBQ03](<../../resource/ymsj-fun.github.io/cards/MSBQ03 圣甲虫药业.jpg>) | 圣甲虫药业 | 黑 | 6 | 卡面允许初始 | 黑/白/中立；无名尸体不限同名数量。 | 墓地尸体封印产业附属并赋势力图标；每局一次检索产业。 |
| [MSBQ04](<../../resource/ymsj-fun.github.io/cards/MSBQ04 天闻山庄.jpg>) | 天闻山庄 | 中立 | 6 | 卡面允许初始 | 任意一个颜色派系和中立。 | 持先手加隐秘展示值；检视手牌、暗藏者或秘密计划。 |
| [MSCQ01](<../../resource/ymsj-fun.github.io/cards/MSCQ01 水月.jpg>) | 水月 | 黄 | 6 | 卡面允许初始 | 黄/中立，另可含其他色法师或法术。 | 卜卦标志累积，法术预测与隐秘对抗修正。 |
| [MSCQ02](<../../resource/ymsj-fun.github.io/cards/MSCQ02 诺尔维隐修会.jpg>) | 诺尔维隐修会 | 红 | 5 | 卡面允许初始 | 红/中立，另可含灾难、时间和命运牌。 | 初始毁灭地区3，四时间毁灭，赢毁灭地区抓牌。 |
| [MSJC01](<../../resource/ymsj-fun.github.io/cards/MSJC01 帷幕守望.jpg>) | 帷幕守望 | 黄 | 6 | 卡面允许初始 | 至少25张黄牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC02](<../../resource/ymsj-fun.github.io/cards/MSJC02 猎魔人.jpg>) | 猎魔人 | 绿 | 6 | 卡面允许初始 | 至少25张绿牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC03](<../../resource/ymsj-fun.github.io/cards/MSJC03 王座会.jpg>) | 王座会 | 蓝 | 6 | 卡面允许初始 | 至少25张蓝牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC04](<../../resource/ymsj-fun.github.io/cards/MSJC04 鸣钟教派.jpg>) | 鸣钟教派 | 红 | 6 | 卡面允许初始 | 至少25张红牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC05](<../../resource/ymsj-fun.github.io/cards/MSJC05 国家机构.jpg>) | 国家机构 | 灰 | 6 | 卡面允许初始 | 至少25张灰牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC06](<../../resource/ymsj-fun.github.io/cards/MSJC06 圣贤.jpg>) | 圣贤 | 白 | 6 | 卡面允许初始 | 至少25张白牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC07](<../../resource/ymsj-fun.github.io/cards/MSJC07 方碑序列.jpg>) | 方碑序列 | 黑 | 6 | 卡面允许初始 | 至少25张黑牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC08](<../../resource/ymsj-fun.github.io/cards/MSJC08 梦境行者.jpg>) | 梦境行者 | 紫 | 6 | 卡面允许初始 | 至少25张紫牌。 | 持先手时付费抓牌；每局一次检索相应派系独有牌。 |
| [MSJC09](<../../resource/ymsj-fun.github.io/cards/MSJC09 秘社.jpg>) | 秘社 | 中立 | 6 | 卡面允许初始 | 无专属构筑条款；仍遵守一般构筑。 | 持先手时付费抓牌。 |
| [MSJC10](<../../resource/ymsj-fun.github.io/cards/MSJC10 貘组.jpg>) | 貘组 | 黄 | 6 | 卡面允许初始 | 黄/中立，另可含其他色星辰领域牌。 | 手牌封印、用封印换灵体或角色正面进场。 |
| [MSJC11](<../../resource/ymsj-fun.github.io/cards/MSJC11 S．P．T执行部.jpg>) | S．P．T执行部 | 绿 | 6 | 卡面允许初始 | 绿/中立，另可含带永久战斗图标的人类角色。 | 赋杀伤1；同地区本方带永久战斗图标角色赋撤回。 |
| [MSJC12](<../../resource/ymsj-fun.github.io/cards/MSJC12 爱弗罗德家族.jpg>) | 爱弗罗德家族 | 蓝 | 6 | 卡面允许初始 | 蓝/中立，另可含吸血鬼或奴仆角色。 | 低费本方人类赋奴仆；横置族群抓牌/令敌方弃牌。 |
| [MSJC13](<../../resource/ymsj-fun.github.io/cards/MSJC13 计时者.jpg>) | 计时者 | 红 | 6 | 卡面允许初始 | 非红且非中立至多12张。 | 放时间标志；消灭达到三标志的角色/附属后抓牌。 |
| [MSJC14](<../../resource/ymsj-fun.github.io/cards/MSJC14 哨兵.jpg>) | 哨兵 | 灰 | 6 | 卡面允许初始 | 灰/中立，另可含无魔法领域图标的其他色牌。 | 锁定敵方暗藏者；本方角色移动到有锁定目标所在地区。 |
| [MSJC15](<../../resource/ymsj-fun.github.io/cards/MSJC15 隐士.jpg>) | 隐士 | 白 | 6 | 卡面允许初始 | 非白且非中立至多12张。 | 初始隐秘标志；手牌≥4时展示角色并暗藏者进场。 |
| [MSJC16](<../../resource/ymsj-fun.github.io/cards/MSJC16 颂亡者.jpg>) | 颂亡者 | 黑 | 5 | 卡面允许初始 | 黑/中立，另可含死亡领域牌。 | 牌库置墓；封印7墓地牌作成本，墓地死亡领域角色正面进场。 |
| [MSJC17](<../../resource/ymsj-fun.github.io/cards/MSJC17 梦魇族群.jpg>) | 梦魇族群 | 紫 | 6 | 卡面允许初始 | 紫/中立，另可含灵体角色或梦境牌。 | 从牌库底抓牌；封印顶牌；墓地灵体回牌库顶。 |
| [MSJZ01](<../../resource/ymsj-fun.github.io/cards/MSJZ01 猛禽战术小组.jpg>) | 猛禽战术小组 | 绿 | 6 | 卡面允许初始 | 至少15张副名“猛禽”战术小组牌。 | 牺牲资产检索猛禽；装备结附猛禽角色降费。 |
| [MSJZ02](<../../resource/ymsj-fun.github.io/cards/MSJZ02 盛夏王廷.jpg>) | 盛夏王廷 | 绿 | 6 | 卡面允许初始 | 绿牌和仙灵牌；不默认允许非仙灵中立。 | 生成仙灵斗士；控制四个仙灵时强制转变。 |
| [MSJZ03](<../../resource/ymsj-fun.github.io/cards/MSJZ03 末日钟.jpg>) | 末日钟 | 红 | — | 不能初始 | 不能作为初始秘社牌。 | 非灾难不能打出或付费翻面；不能获分，三毁灭标志获胜；时间/毁灭与生成钟摆祭司。 |
| [MSJZ04](<../../resource/ymsj-fun.github.io/cards/MSJZ04 破茧者教团.jpg>) | 破茧者教团 | 红 | 6 | 卡面允许初始 | 红/紫/中立。 | 起始检索至多五张不同名梦魇并封印；宿主死亡放梦魇暗藏者。 |
| [MSLC01](<../../resource/ymsj-fun.github.io/cards/MSLC01 联席会议观察员.jpg>) | 联席会议观察员 | 黄 | 6 | 卡面允许初始 | 黄/中立。 | 检视危机方手牌顶；预测；检索防暴构装体部署。 |
| [MSLC02](<../../resource/ymsj-fun.github.io/cards/MSLC02 加利福尼亚猎团.jpg>) | 加利福尼亚猎团 | 绿 | 6 | 卡面允许初始 | 绿/中立。 | 额外支线剧情；完成支线/赢区后付费抓牌。 |
| [MSLC03](<../../resource/ymsj-fun.github.io/cards/MSLC03 洛杉矶警察局.jpg>) | 洛杉矶警察局 | 灰 | 6 | 卡面允许初始 | 灰/中立。 | 名誉点、洗支援牌库、空本方角色城市生成警察。 |
| [MSLC04](<../../resource/ymsj-fun.github.io/cards/MSLC04 圣公会.jpg>) | 圣公会 | 白 | 7 | 卡面允许初始 | 白/中立。 | 威胁进度减1；横置僧侣于无势力地区放标志。 |
| [MSLC05](<../../resource/ymsj-fun.github.io/cards/MSLC05 劣地长老.jpg>) | 劣地长老 | 白 | 6 | 卡面允许初始 | 白/中立，另可含其他色星辰领域牌。 | 生成黄昏界域；手牌更多时黄昏地区略过战斗。 |
| [MSLC06](<../../resource/ymsj-fun.github.io/cards/MSLC06 居民武装.jpg>) | 居民武装 | 中立 | 6 | 卡面允许初始 | 仅中立。 | 检索无知路人进入城市；无领域人类赋战斗/防御标志。 |
| [MSWM01](<../../resource/ymsj-fun.github.io/cards/MSWM01 曼纽家族.jpg>) | 曼纽家族 | 蓝 | 6 | 卡面允许初始 | 蓝/黄/中立，另可含吸血鬼或阴魂。 | 阴魂隐秘修正；本方吸血鬼/法师额外阴魂；未暴露时横置阴魂抓牌。 |
| [MSWM02](<../../resource/ymsj-fun.github.io/cards/MSWM02 公众安全委员会.jpg>) | 公众安全委员会 | 灰 | 6 | 卡面允许初始 | 灰/黄/中立，另可含特工、媒体或互联网。 | 一次移除目标秘社所有隐秘/暴露；弃媒体/互联网作隐秘对抗修正。 |
| [MSWM03](<../../resource/ymsj-fun.github.io/cards/MSWM03 冷山帮.jpg>) | 冷山帮 | 黑 | 5 | 卡面允许初始 | 非黑且非中立至多12张。 | 起始检索黑市；弃牌横置罪犯抓牌，弃罪犯可使该角色潜伏。 |
| [MSWM04](<../../resource/ymsj-fun.github.io/cards/MSWM04 梦魇主母.jpg>) | 梦魇主母 | 紫 | 2 | 卡面允许初始 | 仅紫牌。 | 忽略紫忠诚；复苏标志允许有限免付费打出/现身；检索贝兰丹娜。 |
| [ZHMSJZ02](<../../resource/ymsj-fun.github.io/cards/ZHMSJZ02 凛冬王廷.jpg>) | 凛冬王廷 | 绿 | — | 转变面 | 盛夏王廷的转变形态；不独立构筑。 | 准备步骤强制每玩家牺牲角色；场上无角色则转变。 |

## 685 条记录的目录边界

以下是定位字段建立的待研分类计数，除已核记录外仍需原图确认；不是可玩池规模。

| 待研角色 | 条数 |
| --- | ---: |
| 核心区参考 | 1 |
| 生成指示物候选 | 9 |
| 标志参考 | 1 |
| 一般玩家牌候选 | 591 |
| 角色/附属双类型候选 | 4 |
| 任务/地区候选 | 7 |
| 秘社记录 | 37 |
| 转变面（不能另构筑） | 4 |
| 世界遭遇候选 | 2 |
| 世界地区候选 | 29 |

定位产品标签计数：基础 161、霸权 129、帷幕 109、间奏 82、序曲 55、星光 48、传奇 44、洛城 37、TK 11、测试 9。归档之外产品不在本次声明范围内。

四条转变面明确关联原牌：ZHJZ65 → JZ65、ZHMSJZ02 → MSJZ02、ZHWM026 → WM026、ZHWM048 → WM048。ZHMSJZ02 在 locator 被标为 token，原图证明应保留为形态。

同名集合只提示复核，不自动去重：鲜血 BQ040 / XQ14；洛杉矶 DQBQ01 / DQWM005；钟摆祭司 JC045 / TK009；阿瓦隆隐士 JC071 / LC12。正式版、扩展版、生成指示物或别的印次须逐张判定。

测试标签中的六条原图有 TEST 代码：TEST027、TEST033、TEST034、TEST042、TEST043、TEST052。另三条 BQ010、BQ105、TEST015 原图没有相应 TEST 标记或编号与 locator 冲突，标为 uncertain。不能只按定位标签判为“确定测试牌”，也不能当正式牌直接开放。

13 条缺原图：DQBQ01、DQBQ02、DQBQ03、DQBQ04、DQBQ05、DQBQ06、DQBQ07、DQBQ08、DQBQ09、DQBQ10、TK004、TK010、TK011。

## 源码核实与实现状态

`implemented` 表示明确列出的有限行为在此基线的源码及既有测试声明中存在；`partial` 表示有部分原语但有明确缺项；`unimplemented` 表示没有完整状态/声明/入口；`uncertain` 表示来源或完整规则不明。未运行的测试不能用作新通过证据。

实际审计了 catalog、手写规则注册表、engine、resolution、model、service 与 Lobby。生产卡池只有 25 个玩家定义 + 4 个世界定义（世界牌库含 10 张复制）。`rules::definition` 手写登记的 ID 与这 29 个定义相符；引擎不执行归档 JSON 的规则文本。`catalog.supported` 默认值本身不是机制实现证据。研究索引对现有 29 个定义标为卡级 partial（有有限实现，完整原作等价未验收），没有用登记存在直接宣称完整牌规则已实现。

机制状态、原始页码和源码/既有测试锚点详见数据。以下列表便于交接检索；partial 的明确限制仍以数据为准。

| 共享机制 ID | 名称 | 状态 |
| --- | --- | --- |
| loyalty | 派系/领域忠诚与资产支付 | implemented |
| hidden | 秘密派遣、暗藏者、付费现身和潜伏 | implemented |
| priority | 响应堆叠、付费与可恢复选择 | partial |
| confrontation | 调查、战斗、势力与赢区 | implemented |
| public | 公开 | implemented |
| exhaust | 横置与重置 | implemented |
| mobility | 机动与场上移动 | partial |
| combatTraits | 杀伤、护卫、撤回 | implemented |
| barrier | 屏障 | implemented |
| shield | 护盾 | partial |
| graveRecovery | 墓地回手/牌库底/暗藏者进场 | implemented |
| gravePlay | 墓地/其他区域直接打出 | unimplemented |
| sacrifice | 牺牲费用与牺牲效果 | implemented |
| costReduction | 降费 | partial |
| deckChoice | 抓牌、弃牌、预测、检索及排序 | partial |
| continuous | 持续数值修正与类型/能力赋予 | partial |
| healWounds | 移除创伤 | partial |
| society | 秘社牌与秘社区 | unimplemented |
| attachment | 附属/结附、回收与双类型牌 | unimplemented |
| seal | 封印与载体生命周期 | unimplemented |
| renown | 声望 | unimplemented |
| spirit | 灵体 | unimplemented |
| lock | 锁定与检视暗藏者 | unimplemented |
| timeDestroy | 时间标志与毁灭地区 | unimplemented |
| damagePrevention | 通用防止/伤害替代 | unimplemented |
| control | 操控权变化 | unimplemented |
| transform | 转变与多形态 | unimplemented |
| blink | 闪烁 | unimplemented |
| nonAsset | 非资产 | unimplemented |
| slow | 迟缓 | unimplemented |
| leader | 领袖与唯一构筑 | unimplemented |
| token | 生成指示物角色/附属 | unimplemented |
| mysticContest | 隐秘对抗、隐秘/暴露与隐秘等级 | unimplemented |
| plan | 秘密计划与推进 | unimplemented |
| rumor | 传闻 | unimplemented |
| blood | 鲜血附属与饮血 | unimplemented |
| task | 任务地区 | unimplemented |
| core | 核心角色、核心区与命令 | unimplemented |
| encounter | 遭遇角色、利用与恐怖 | unimplemented |
| starlightInfluence | 附属上的地区势力 | unimplemented |
| combatKeywords | 袭击、创伤、遏制、威名等战斗关键词 | unimplemented |
| hiddenOnly | 隐匿 | unimplemented |
| cancel | 终止/取消与替代效果 | unimplemented |
| handAbility | 手牌能力、神秘学博弈等 | unimplemented |
| crisis | 洛城危机/支线剧情/威胁与支援 | uncertain |
| chaos | 混乱 | unimplemented |
| herald | 先兆手牌行动 | unimplemented |
| reincarnation | 转生 | unimplemented |
| eternal | 永恒 | uncertain |
| temporaryIcons | 临时能力图标 | partial |
| uniqueInPlay | 独有（金色名称） | implemented |
| privatePeek | 私密检视与持续检视权限 | unimplemented |

几个容易被误判的边界：

- 护盾消耗有状态字段和夹具测试，但所有生产实例护盾初值为 0，没有赋予护盾的开放牌/操作，故 partial。
- LC19/20 可移除创伤，防御会减创伤；没有生产牌赋创伤的操作，不能据此声称创伤体系完整。
- JC086 回墓地角色到手、JC092 墓地角色暗藏者进场、XQ49 回拥有者牌库底已经登记；JC085 的墓地直接打出是另一流程，未实现。
- JC042 的降费只限下一张正面打出的鸣钟教派或血脉牌，忠诚/追加费用仍须检查，不能推广到所有降费效果。
- 封印、声望、秘社、附属、锁定、通用控制变化等没有完整生产入口；卡图和关键词存在不等于引擎支持。
- 堆叠与选择只覆盖有限零目标或单个固定目标槽；任意多目标、通用强制触发、取消和替代仍需补全。

## 模式区别

| 模式 | 原始证据 | 当前状态 |
| --- | --- | --- |
| 霸权 1V1 | 霸权说明书 P6 / PDF 7 等设置页 | partial：三地区、8 分，有限卡池 |
| 霸权 2V2 | 霸权 P21 / PDF 22 | partial：同侧队伍、五地区、10 分、距离与共同步骤；隐秘对抗缺失 |
| 经典基础版 2V2 | 玩家指南 P19 / PDF 21 | unimplemented：对坐、12 分、顺时针个人行动步骤，不是当前 teams |
| 统治者 1V2 / 1V3 | 玩家指南 P20–21 / PDF 22–23 | unimplemented：分别四/五地区，10/12 分，另有 30 张指令牌库及谋划 |
| 洛城危机/剧情 | MSLC01–04 原图 | uncertain：归档缺完整模式手册，当前无此模式入口 |

源码中队伍由 `seat / 2` 得出（0/1 同队、2/3 同队）；`seat % 2` 区分同侧队伍内左右位置。位置和队伍不能混为一谈。洛城 LC19–24 中立牌已进入有限池，不证明洛城专用模式已实现。统治者“指令”与修格斯“命令”也不能共用同一规则标签。

## UI 集成约束

官方颜色导航展示八色及单独的中立项，支持只读查看证据、覆盖数量和缺少的机制。官方秘社、基础两色配对与当前自组预组应使用不同标题和数据来源。所有官方导航/秘社/任意配对选择在本数据中禁用；现有五副预组仍由运行时 `/api/catalog.decks` 提供，覆盖数据不得扩展开局命令。

适合客户端使用的是较小的 faction coverage；完整研究索引面向开发审核。没有必要将全部原始定位文本送入游戏页面。缺少数据、未知状态或源码漂移时，显示“待复核”且保持新内容禁用。机制状态与卡级状态分别展示，不能把“一个操作已实现”改写为“整种颜色可玩”。

## 生成、验证与局限

在仓库根目录执行：

```sh
python tools/factions/build_research_index.py --check
python tools/factions/validate_coverage.py
python -m unittest discover -s tools/factions -p 'test_*.py' -v
```

更新人工 reviews、coverage、完整规格或缺图补证后，先检查原始证据，再运行 `build_research_index.py` 生成研究索引。生成器只取人工记录提供核验字段，只把原始 JSON 放在 locator 区；当前 catalog 哈希变化时拒绝继续生成旧实现标签。全卡规格是独立人工输入，不能由 locator 文本自动填成完成。

严格验证会检查原始文件哈希、证据关联、八色映射、秘社记录完整性、禁用策略、组牌/模式概念、当前 29 个注册 ID、复制数、机制依赖无环、源码/既有测试锚点和索引可复现。`--allow-code-drift` 只把源码快照/锚点变化报告为警告，仍不解锁卡；不得当作重新审计完成。

截至 B02，实际运行的是上述独立 Python 校验与 26 项回归测试，全部通过。Rust 既有测试仅核对声明存在，没有在此环境运行；当前环境没有 cargo。也没有执行 UI/策略试玩、线上房间回归或发布验证。未完成的规则研究和未来验收要求见 implementation-plan.md。
