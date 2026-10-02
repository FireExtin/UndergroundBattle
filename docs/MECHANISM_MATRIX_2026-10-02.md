# 原稿机制与有限 Rust 卡池只读对照（2026-10-02）

本次只读查看原 PDF 页面、扩展说明原图及下列原卡图，并对照现有源码；未运行测试、创建房间、调用 Sites 或修改仓库文件。源码读取时 HEAD 为 d3d88c5d21784b7543627090858dad567361a62d，内核标识 rust-v0.2.1 / limited-v2.1 / hegemony-pdf-v1。结论是源码及原稿审查，不是新增行为验收。

**先前报告的更正：不能把“非角色手牌不能秘密派遣”定为实现缺陷。**我先前孤立使用规则手册P7的“一张手牌”泛称，推成任意牌型；原[霸权P11/物理12](/tmp/hegemony-readonly-rule-review/霸权说明书-physical12.png)明确写“手牌中的一张角色牌”，玩家指南P9同样明确。FAQ[物理P5](/tmp/hegemony-mechanism-readonly-review/隐秘世界勘误及释疑-physical5.png)花园地精像只证明效果造成的非角色暗藏者不能付费现身，不证明可秘密派遣非角色。当前character门禁符合所选霸权基准。

## 1. 先区分原游戏、正式预组与当前受限预组

当前 registry 有29个开放定义：25玩家牌、4地区。玩家牌按不同定义计：中立9、黄3、绿3、灰3、黑4、蓝1、红2、白0、紫0。数量是定义数，不是某套牌的张数。白、紫尚无可玩玩家牌；其余派系也没有完整的官方基础预组。原素材库包含更多卡图或文本，不代表这些卡已经由规则引擎实现。

五套50张牌的 description 均明确为非官方受限预组：watchers=黄+中立；hunters=绿+中立；keepers=灰+中立；reclaimers=黑+蓝+中立；responders=红+黑+中立。它们不能冒称完整派系、官方双派系预组或完整构筑能力。

原稿构筑依据：

| 规则 | 已查看的原页/原卡 | 对当前实现的含义 |
|---|---|---|
| 八派系：帷幕守望黄、猎魔人绿、王座会蓝、鸣钟教派红、国家机构灰、圣贤白、方碑序列黑、梦境行者紫；中立显示褐色但没有派系 | 玩家指南印刷P1/物理3 | 中立不能计成第九派系；不能按五套demo推导派系玩法覆盖 |
| 快速组牌：任取两个基础派系，各含25张，混为50张 | 玩家指南P6/物理8 | 是新手组牌方法，不是通用“最多两派系”规则 |
| 正式自组：至少50张；同名最多3张；同名不同版本合并计数；可以添加任意种类派系 | 玩家指南P12/物理14 | 没有核实到通用派系数量上限；也没有此处规定的牌组最大张数 |
| 秘社可选一张、或不用；不计入50张；必须符合其构筑限制 | 玩家指南P22/物理24 | 不能把秘社派系等同于全牌组只能单色，也不能忽略秘社限制 |
| 帷幕守望要求至少25张黄牌 | MSJC01原图，印刷Society01/17 | 是数量下限，不是禁绝其他颜色 |
| 颂亡者允许黑、中立及其他派系死亡领域牌 | MSJC16原图，印刷Society16/17 | 构筑限制需要可组合的颜色/魔法领域谓词 |
| 无知路人数量不限 | JC125原图125/135 | 同名3张规则有印刷例外，不能无条件截断为3 |
| “唯一”构筑只能一张；“领袖”限制同一玩家同时控制一个领袖在场 | 霸权P18/物理19 | 与“独有”角色同名进场规则不同，不能复用成同一个标志 |
| 打出/付费现身须满足全部忠诚图标；已横置资产仍算忠诚；支付费用不必用满足忠诚的那些资产；有魔法领域忠诚 | 规则手册P11/物理13；FAQ P5阿斯旺 | 当前 loyalty()统计全部资产颜色和魔法，方向符合；完整构筑校验和秘社支持尚缺 |

代码依据：[catalog.rs](/workspace/UndergroundBattle/rust-game/src/catalog.rs:79)仅提供有限deckId表；[loyalty()](/workspace/UndergroundBattle/rust-game/src/engine.rs:168)；[traits](/workspace/UndergroundBattle/rust-game/src/rules.rs:183)。当前不是自组牌提交接口。已有测试检查五套固定50张和JC125例外，不等于已经实现通用“按印刷同名合并、秘社限制、唯一”构筑验证器。

## 2. 核心机制矩阵

“已实现”只表示下述有限用途存在；“部分”表示有字段/基础操作，不能据此宣称机制完整。

| 机制 | 原稿核实 | 当前状态及证据 | 缺口 |
|---|---|---|---|
| 封印 | 霸权P15/物理16：被封印牌不在场、公开可查、牌面空白；承载者离场或翻为暗藏者时回拥有者手上。规则手册P4/物理6还说明被封印离场触发与失效时的墓地去向 | 未实现。model无sealed区/承载关系，Zone/Destination/Op无Seal | 需独立封印区、host实例、公开投影、离场/翻面释放、封印来源/承载目标失效处理；不能用墓地或附属字段冒充 |
| 声望 | 霸权P18/物理19；JC070原图070/135：一个地区所有对抗结束后，比对手有更多未横置声望角色者可加1本方势力，不能叠加 | 未实现。Traits无renown，Event无地区所有对抗结束，Op无通用放置势力 | 需动态trait、对抗末尾事件、数目比较、可选触发及1次上限；旧规则手册P8/物理10使用“参与此次对抗”措辞，不应混成最新版 |
| 墓地与死亡 | 规则手册P7/物理9：任何区域进入墓地均为拥有者墓地；P8/物理10：角色从场上进墓地算死亡。霸权P15明确牺牲不等于消灭 | 已实现有限墓地移动及Death。remove_dead区分原因、按owner入墓；SourceSnapshot保留死亡能力来源 | 尚无所有区域的通用Move、离场/置墓事件与替代事件体系 |
| 墓地回收/再部署 | JC086窃尸人；JC092死灵召唤的原稿已由之前集中图审确认 | 已实现。rules.rs JC086为墓地角色->ActorHand；JC092为墓地角色->HiddenInChosenRegion | 不等于墓地正面打出、任意跨墓地操控进场、任意复生。此处“复活”是效果族；本次原页未见独立统一的“复活”关键词 |
| 墓地行动/正面打出 | JC085原图085/135：墓地正面打出，仍付费并满足忠诚；FAQ P4/物理4“复生行尸（基础074）”明确墓地不能秘密派遣 | 未实现。普通打出/派遣从hand取牌，AbilitySpec无可发动源区域许可 | 需PlayFromZone许可重用普通支付/忠诚/堆叠/进场流程；不可把“置于进场”（不视作打出）与此混同 |
| 拥有者/操控者 | 霸权P15/物理16：owner不变，controller可变；“本方/你的”按当前controller；牺牲只能选自己操控的牌 | 部分。Card有owner/controller，目标有OwnedByActor/ControlledByActor，支付牺牲检查控制权，死亡进owner墓地 | 无GainControl或控制效果层；拥有字段不代表实际支持夺取/到期返还 |
| 临时控制权 | JC129原图129/135：行动，获目标敌方角色操控权直到回合结束 | 未实现 | 需带来源、持续期、顺序的控制效果，恢复前一个有效控制层；跨回合清理、失效、存档/重放不能靠卡名分支 |
| 附属持续控制 | JC036初拥原图036/135：结附印刷人类，夺取控制、把人类改成吸血鬼；FAQ P3/物理3：附件被消灭或角色潜伏后交回owner | 未实现。当前无可玩附属/附件图，也无类型变更层 | 需Attachment(host)+WhileAttached控制/类型修正；附件操控者不随host操控者自动变化 |
| 横置/潜伏控制 | JC063原稿；霸权P11/物理12、P17–18 | 已实现有限Exhaust、Hide、现身及新实例锁定；现身可触发Enter/Reveal；目标离场/变类型会使原锁定失效 | 不完整支持潜伏/离场触发、附件/封印清理、任意持续修正；无通用潜伏事件 |
| 角色/暗藏者目标 | 规则手册P1/物理3、霸权P11：暗藏者非角色，印刷属性不生效、伤害无效；FAQ P3刑警/警车追捕要求结算时仍是原暗藏者 | 已实现有限EntityKind::Character/Hidden/CharacterOrHidden；valid_binding再查实例、类别、关系、范围；damage忽略face_down；icons对暗藏者只给1势力 | 敌方手牌/暗藏者的规则检视授权、来源许可与持续检视不完整；原卡JC057不是UI放大读取自己卡牌 |
| 秘密派遣可用牌型（重要更正） | 玩家指南P9/物理11与霸权P11/物理12都明确“一张角色牌”；规则手册P7的一张手牌是泛称。FAQ P5/物理5花园地精像仅说明非角色经效果潜伏后不能付费现身 | 当前conceal要求character，符合本项目霸权基准 | **不能把“非角色手牌不能秘密派遣”报告成确定缺陷，也不建议放宽。**完整模型仍需处理非角色因效果潜伏后的不可付费现身；复生行尸墓地也不能秘密派遣 |
| 检视隐藏信息 | JC057监控员原图057/135；FAQ P3/物理3：未横置时可检视所在地区敌方暗藏者；现身检视目标手牌 | 未实现敌方手牌/暗藏者的完整规则授权操作；现有Forecast为自己牌库待选 | 需PrivateKnowledge/Inspect许可，限制可见对象、范围和有效期，避免把内容公开给队友/所有人 |
| 目标/费用/独立来源 | 规则手册P7/物理9，霸权P16/物理17：先锁定目标后支付，目标失效费用不退，能力来源离场不取消独立能力 | 有通用Frame、Guard、成本账本、游标、SourceSnapshot；真实JC042/JC091旧目标失效响应已有已存证据 | 当前validate_ability明示仅0或1个target slot，且min=max=1；不是多目标通用语言。Zone也没有Stack/Asset/Hand/Sealed |
| 终止堆叠对象 | 霸权P17/物理18：终止作用于待结算牌，付费现身牌也可终止，已在场牌不能这样终止 | 未实现通用Terminate/Stack目标。现有Guard取消只覆盖失效目标或护盾 | 需StackRef与终止操作，不能把取消非法命令、潜伏规避或护盾终止称作通用反制 |

关键源码：[rules.rs AST](/workspace/UndergroundBattle/rust-game/src/rules.rs:115)、[源区域/目标限制](/workspace/UndergroundBattle/rust-game/src/rules.rs:207)、[model Card/Player](/workspace/UndergroundBattle/rust-game/src/model.rs:242)、[目标再校验](/workspace/UndergroundBattle/rust-game/src/resolution.rs:419)、[Move/Hide](/workspace/UndergroundBattle/rust-game/src/resolution.rs:920)、[死亡](/workspace/UndergroundBattle/rust-game/src/resolution.rs:838)、[隐藏投影](/workspace/UndergroundBattle/rust-game/src/engine.rs:1594)。现有测试名只能作为已有源码证据，本次没有运行它们，也没有新增线上验证。

## 3. 其他正式关键词的覆盖范围

以下霸权词条均实际查看P17–18/物理18–19。四扩说明原图在前次只读模式研究中已经实际查看，本次复用该证据。

| 关键词/机制 | 当前覆盖 |
|---|---|
| 公开、屏障、机动、护卫X、杀伤X、预测X | 有实际有限卡声明和对应执行流程；公开只限制秘密派遣，并不禁止效果潜伏；屏障限制敌方指定目标，并非对非目标效果免疫 |
| 潜伏/现身、撤回 | 有有限基础流程；撤回角色地区被赢取回owner手中。无完整附件/封印/所有翻面事件。霸权把撤回写持续，旧手册写触发，也需按指定规则版本处理 |
| 创伤X | 有wounds字段、降低防御和HealWounds；无完整产生创伤的操作/战斗奖励替换。LC19/20文本涉及治创伤不等于这两张有“创伤X”进攻能力；metadata关键词命中不能充作已支持 |
| 护盾X | 有shield字段和敌方目标结算时消耗、终止当前效果逻辑及已有单测；没有当前可玩卡产生护盾的声明/操作，属于部分支架 |
| 声望、威名、灵体、领袖、隐匿、混乱、饮血X、传闻、回收、通用标志X | 未有完整通用执行。领袖不能拿同名独有检查代替；隐匿应禁止正面打出；灵体要求地区魔法图标比较与防伤；威名需要赢得战斗且参与的触发上下文 |
| 唯一/不限副本、独有 | JC125不限副本标志及固定牌表例外存在；同名独有进场选择存在。完整构筑“唯一1张/按印刷同名最多3张”与不同名称领袖上限尚无通用校验 |
| 非资产 | 无当前trait/建资产门禁；版本不同：间奏原图禁止任何效果使其成为资产，霸权P18却明确其他效果建资产不受该关键词影响；应固定版本，不能融合成自创规则 |
| 闪烁 | Hide与另一次付费现身不足以自动覆盖原缩写；原间奏说明及霸权P18要求潜伏后立即现身、离场再进场并使旧目标失效。需要一次效果内的完整生命周期 |
| 毁灭、迟缓、转变 | 四扩原图正式机制；当前无通用对应效果或状态。毁灭计分-1不能用角色伤害替代 |
| 遭遇角色/利用、恐怖X | 序曲原图正式机制；当前无独立世界角色中立控制、利用许可及恐怖时机 |
| 隐秘对抗、隐秘/暴露标志、秘密计划/可推进、非地区势力标志 | 帷幕之后/星光俱乐部原图正式机制；当前无完整流程。存在单个keyword标签或字段不代表整个扩展可用 |

## 4. 少乃至零运行时卡名特判的原语建议

现有rules.rs的逐卡注册是合法的“印刷牌->声明”绑定位置；真正应避免的是engine/resolution到处按卡名/ID决定行为。原文指定某个名称时，声明中的Name谓词是规则数据，不是引擎特判。

1. **共用筛选器和条件表达式。**实体区域、印刷/当前类别、控制/拥有关系、颜色、魔法、费用、横置、来源地区、排除自身、数量比较；供目标、费用、检索、构筑与静态能力共用。新增Relation::FriendlyTeam，不能把本方与友方混为一谈。
2. **共用跨区移动与生命周期。**区分Play（付费用/忠诚/堆叠）和PutIntoPlay（效果放置）、MoveOnBoard（进入地区但不进场）、Hide/Reveal/Blink（离场及新实体实例）；产生Enter/EnterRegion/Reveal/Leave/Death等可声明事件，不按复活卡名分支。
3. **通用持续修正。**字段包括图标、防御、魔法、类别、关键词、controller；持续期为回合末、来源在场、结附有效等；保存来源/时间顺序和过期条件。控制返还恢复上一个有效层。单个owner/controller字段不足以处理持续附件和临时夺取叠加。
4. **封印与附件关系图。**Seal(card, host)、Attach(card, host)、Release(host)；封印牌公开、不在场、不能普通指定；host翻面/离场统一释放。复用Frame成本/游标/PrivateChoice恢复，不能把被封印牌塞墓地后靠卡名找回。
5. **事件与资源标志。**AfterRegionConfrontations、CombatWon、AboutToDamage、Sealed等；TriggerSpec声明可选/强制、频次及上下文；AdjustMarker/Influence与阈值检查共用。声望/威名是事件+条件+效果，避免各写独立卡名程序。
6. **动作源区域许可。**PlayPermission(from=Graveyard, face=Up)直接进入普通付费与忠诚流程；反制需要Stack目标和Terminate，私密检视需要有时效的查看许可。
7. **独立构筑验证。**MinDeckSize(50)、按印刷名称合并的副本上限、唯一/不限覆盖、秘社谓词。通用规则不新增“最多两个派系”。这些扩展引入新持久状态时需显式版本迁移/新规则版本，不能让旧房间静默按新能力解释。

## 5. 最小扩卡候选，不声称构成完整派系

以下均实际查看原卡图；成本与忠诚必须按原图录入，不能为方便演示改低费用。候选用于证明通用原语覆盖，**不是**新增实现授权，也不是平衡预组设计。

| 顺序 | 原卡/编号 | 最少依赖与验证价值 |
|---|---|---|
| 0，可先不加opcode | JC058坚毅的刑警，058/135 | 原图为**现身**触发，消灭本地区目标暗藏者（未限定敌方）。现有Event::Reveal + Hidden/Any/SourceRegion + Destroy即可绑定；响应付费现身后旧目标必须失效。不能拿旧FAQ“进场”措辞覆盖新原图 |
| 1，填白色机制 | JC070虔心修士，070/135 | 公开+声望；只需通用声望trait/地区对抗末事件/加势力，不引入附件、复杂目标或额外卡名程序 |
| 1，声望与已有触发组合 | JC076悟道僧人，076/135 | 声望+现身抓1；复用Draw和Reveal。与JC070任选其一作为第一张，并非两者都必需 |
| 2，墓地动作 | JC085复生行尸，085/135 | 墓地正面打出许可重用付费/忠诚；资产死亡魔法满足时加临时势力图标，需要条件修正。不能从墓地秘密派遣，不能省略印刷费用 |
| 2，临时控制 | JC129操控心智，129/135 | 3费事务，目标敌方角色控制到回合末；最小ControlModifier/expiry案例，不需附件层 |
| 3，填紫色封印案例 | XQ40催眠术表演者，40/52 | 行动2+横置，抓1，然后选一手牌封印目标角色。Draw/Exhaust已有；新增通用手牌选择与Seal，能验证私密待选恢复、host离场回owner手上 |
| 后续控制附件案例 | JC036初拥，036/135 | Attachment + WhileAttached控制/类别层；原图限定**印刷人类**。须验证附件消灭、host潜伏、控制叠加后返还，不能简单修改一次controller |

JC066封魔瓶瑟琳娜（066/135）适合作为封印完成后的第二个案例：它还需要从其上所有被封印角色读取**印刷能力图标**的聚合修正；FAQ明确不能复制那些牌通过其他效果获得的图标。因此它不是最低成本的首张封印牌。JC057监控员（057/135）可独立验证受限敌方隐藏信息检视，不应让其混入所有人公开View。

## 6. 证据与未知边界

原PDF目录：/workspace/UndergroundBattle/resource/ymsj-fun.github.io/public/docs/。本次新页面渲染在同目录本笔记旁：/tmp/hegemony-mechanism-readonly-review/。霸权P15–18原页渲染另在/tmp/hegemony-readonly-rule-review/霸权说明书-physical16..19.png；P11在physical12.png。印刷页与物理页已逐图确认。

已看原卡路径统一为/workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/{编号} {名称}.jpg。JCxxx是本地新版编号（例如JC058原图058/135）；FAQ仍用旧基础051，二者需按名称和原文对应，不能把编号当同一版本。

正式规则版本差异已明确列出：声望参与措辞、非资产、秘密派遣泛称、撤回触发/持续、刑警进场/现身。仅凭旧整理文本或FAQ无法解决所有跨版本一致性问题；新增卡应以选定规则版本+对应实际原卡为准。多重控制层冲突的官方顺序细则未从本次材料核实，建议通用模型预留顺序与来源，并另取正式解释再开放涉及冲突的牌。没有把推断当作原稿已规定。

总体建议：先建立正确的公开能力/构筑范围声明和通用选择、移动、修正、封印模型，再逐牌以声明扩展。即使加入上述候选，也只是机制覆盖增量，不能宣称八派系或各25张官方预组已经完整。

## 7. 已核实的统治者/战役模式结论（复用本次先前原图审查）

统治者正式规则来自[玩家指南PDF](/workspace/UndergroundBattle/resource/ymsj-fun.github.io/public/docs/隐秘世界玩家指南.pdf)印刷P20–21/物理22–23，实际渲染图：[P20](/tmp/hegemony-mode-readonly-review/玩家指南-physical22.png)、[P21](/tmp/hegemony-mode-readonly-review/玩家指南-physical23.png)。3/4人，一名统治者对抗2/3名组队玩家；统治者初始先手。1V2四地区、任一方总分10即胜；1V3五地区、总分12即胜。任一玩家将抓牌而库空，其整方立即失败，不能沿用当前2V2的队友继续规则。

统治者起始手牌+1；1V3首次回合前可把牌库顶建额外资产；每回合多一次秘密派遣，第二次2费。必须另有30张指令牌（不是“30种”，指南示例编号01/10；完整组成未定位），基础版不含。每次一张、1V3可依次两张；能力可选、无堆叠、不能响应；结算后牌底，谋划则占一个秘社位置、结束步骤强制触发且普通效果不能使其离场。组队共享势力、共同行动全员让过、奖励只给一人、先手特权全队每回合一次；队友不能检视彼此手牌或暗藏者。当前mode只有duel/teams，尚未实现统治者优势、指令区/谋划、非对称人数及其整方空库失败。

战役准确名称“洛城惊变”有本地卡FAQ证据（organized_content/cards/秘/cards.json 与事/cards.json），不是由其他桌游推测。实际看过以下原卡：

- [MSLC01 联席会议观察员](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/MSLC01 联席会议观察员.jpg>)：危机方手牌区顶牌。
- [MSLC02 加利福尼亚猎团](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/MSLC02 加利福尼亚猎团.jpg>)：额外支线剧情/完成触发。
- [MSLC03 洛杉矶警察局](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/MSLC03 洛杉矶警察局.jpg>)：名誉点与支援牌库，每局开始名誉+1、放弃一局-1。
- [MSLC04 圣公会](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/MSLC04 圣公会.jpg>)、[LC13 远古先祖的回应](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/LC13 远古先祖的回应.jpg>)：威胁进度-1。
- [LC29 组织营救](</workspace/UndergroundBattle/resource/ymsj-fun.github.io/cards/LC29 组织营救.jpg>)：名誉点+2。

已检查四份原PDF与四张扩展说明原图，未找到完整战役手册。因此战役人数、阵营控制、先手、胜负、章节顺序、跨局状态继承及专用卡池组成未核实，不能从这六张牌补出正式规则。名誉/威胁/剧情等是已确认概念，具体持久进度规则仍未知。当前无这些模式状态/牌库。先补齐统治者指令素材可形成相对明确增量；战役须先取得完整规则及剧情、危机、支援材料。这里只记录依赖，不新增实现或部署任务。

## 8. 快速原图索引

- [八派系颜色：指南P1](/tmp/hegemony-mechanism-readonly-review/隐秘世界玩家指南-physical3.png)
- [快速双派系25+25：指南P6](/tmp/hegemony-mechanism-readonly-review/隐秘世界玩家指南-physical8.png)
- [任意派系、至少50、同名3：指南P12](/tmp/hegemony-mechanism-readonly-review/隐秘世界玩家指南-physical14.png)
- [秘社限制：指南P22](/tmp/hegemony-mechanism-readonly-review/隐秘世界玩家指南-physical24.png)
- [忠诚与魔法：手册P11](/tmp/hegemony-mechanism-readonly-review/隐秘世界规则手册-physical13.png)
- [封印细则：手册P4](/tmp/hegemony-mechanism-readonly-review/隐秘世界规则手册-physical6.png)
- [拥有者/操控者、封印：霸权P15](/tmp/hegemony-readonly-rule-review/霸权说明书-physical16.png)
- [声望及关键词：霸权P18](/tmp/hegemony-readonly-rule-review/霸权说明书-physical19.png)
- [初拥、暗藏目标失效FAQ：P3](/tmp/hegemony-mechanism-readonly-review/隐秘世界勘误及释疑-physical3.png)
- [复生行尸墓地不能秘密派遣：FAQ P4](/tmp/hegemony-mechanism-readonly-review/隐秘世界勘误及释疑-physical4.png)
- [非角色暗藏者不能付费现身：FAQ P5](/tmp/hegemony-mechanism-readonly-review/隐秘世界勘误及释疑-physical5.png)

新增原卡逐图样本：JC125/JC129/JC036/JC085/JC066/XQ40/JC057/JC065/JC070/JC076/JC058/MSJC01/MSJC16，模式样本MSLC01–04/LC13/LC29；此前集中有限卡图审的JC086/JC092等复用已确认结论。这不是全685张逐图审，不声称完整资料核验或完整派系实现。
