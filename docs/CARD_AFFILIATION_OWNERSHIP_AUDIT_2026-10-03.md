# 当前48定义阵营与归属定点语义闭环

本次逐张实际查看48张固定原图，对照说明书实读P15/P16/P17/P18/P21、实际编译的有限绑定、目标声明/结算复验、离场及回收目的地。没有增加卡、改预组、泛化重构、变更域名、访问任何公网房间、推送或发布。

**线上39没有发现新增的自然可达“本方扩队友”或owner/controller归属混用问题。** 本地9张中此前XQ16错误已经由53e47ca修正；当前48未发现需要新增生产修复的问题。此结论是当前有限卡池的语义审计，仍不是完整规则或自然UI验收。JC016＋BQ022赢区撤回/回收优先关系继续待裁定，不能计为闭环。

## 判定依据和容易误改的例外

- P15：拥有权不随控制改变，牺牲仅可选本人当前操控牌；P16：本方是本人、友方含队友、你指该牌当前controller。
- JC059原图写友方，持续加防御应覆盖当前controller同队的其他正面角色，不是只覆盖本人。
- JC084原图写本方/敌方势力标志，但P21明确组队玩家**共用势力标志**；按controller选择队伍标志正确，不能机械拆成个人标志。
- BQ022原图回收括注写“你的手中”，P17的**回收专门定义明确回附属拥有者手牌**；OwnerHand正确，不应换成controller。P18撤回亦明确回拥有者。
- JC086/JC092的“你的墓地”与Actor私有区相符，OwnedByActor限定其拥有牌。当前全部准入效果保持手牌/牌库/墓地的owner容器不变量；没有夺取控制或把牌转入其他玩家私有区的准入卡。
- XQ49、JC063、JC006、JC075中明写拥有者的目的地按owner；发动者本人支付、检视、抓牌和私密选择则按Actor/current controller或各自Context。死亡触发归离场前controller，消灭后的卡归owner墓地。
- 每位玩家的世界效果分席Context执行，伦敦按controller＋地区混暗藏者，不把同队/同owner合组。

## 与当前发布源码的实际对照

当前卡链8963fcd的48定义与已发布Site15源码074def9的39定义，均从各自源码实际编译导出完整rules::definitions，而非从前端summary猜目标规则。后者仅将已发布源码的两crate最小副本放本地、加入同一只读导出example，复用既有编译cache；未联网读取站点或房间。

39旧牌的relation、PlayerRef、Destination、modifiers、host_leaves以及各既有有限操作在对照中全部相同。比较前明确排除五个后加schema guard字段，并完整列出非零项，**没有把完整定义逐字相等当作事实**：BQ022新增equipment_host=true仅用于已审本地JC001“不能装备”，所有线上39宿主的cannot_be_equipped都为false。此项是先前的猫头鹰增量，非本次阵营/归属修复。可重复导出工具是 `rust-game/examples/semantic_scope_audit.rs` 与 `tools/research/audit_card_affiliation_scope.py`。

本地新增9张为 BQ083、JC001、JC006、JC007、JC047、JC075、JC088、JC104、XQ16，均逐卡列在下表。仅XQ16有本轮引用的历史纠正；53e47ca未进入公开39池，其他8张没有发现同类错误。

## 新增边界测试与变异

新增三项有行为判别力的原生测试：跨owner/controller的友方防御、敌方图标及屏障；附属owner≠controller时按P17回收owner；JC086/JC092本人墓地与队友/敌方墓地隔离、非法选择原子拒绝。需要控制不同的初始布局均明确为本地primitive，不冒充当前池自然可达，也不修改不可变owner。

`cargo test --workspace`最终132项通过（100库＋32集成），三项新增测试全绿。独立最小变异均实际编译后断言失败：XQ16本人→队友；墓地OwnedByActor→FriendlyTeam；回收owner→controller。每个日志必须含对应变异源码编译路径，复用同一target时先触摸变异源码防止旧产物误命中。原53e47ca红绿及本次变异均有精简差异材料。

142个生产/原图/界面文件按SHA保留，engine.rs在cfg(test)前生产部分与8963fcd逐字相同；无新增生产修复或机制。实际WASM保持1619119字节、SHA256 `6bbd2ce9ffdefdb3c24b7c0ca1786440ce087e4be34cff6af7f6c4af5e7bbc51`。上一批168前端、22Worker/D1、默认26WASM场景的证据对应完全相同的生产代码和字节，本轮未重复声称重新跑过这些检查，新三项边界测试只声称原生验证。

## 53e47ca精简审阅入口

- 完整生产diff：[53e47ca-production.diff](evidence/semantic-scope-audit-2026-10-03/53e47ca-production.diff)，仅rules.rs一处FriendlyTeam→ControlledByActor。
- 完整scope相关源码diff：[53e47ca-scope-tests.diff](evidence/semantic-scope-audit-2026-10-03/53e47ca-scope-tests.diff)，含原生新增“响应期间转为队友控制则取消”、本人跨owner候选与队友拒绝、四席隐私、默认oracle与Worker/D1对应修改。
- 逐卡完整编译定义/实读原图SHA：[48-card-audit.json](evidence/semantic-scope-audit-2026-10-03/48-card-audit.json)。

历史CONTROL_PAIR及SEVEN_CARD审计里XQ16可作用队友的旧文字应按53e47ca纠正，不再作为规则依据；历史红绿原记录保持其原时点。完整大源码包另有8963fcd Library审阅材料，本次提供易读的定点补充，不要求父线程重读整个14MB文件。

## 逐卡结论

| ID | 名称 | 范围 | 本次定点结论 |
| --- | --- | --- | --- |
| JC125 | 无知路人 | 线上39 | 无阵营限制；同名构筑例外不涉及归属。 |
| LC19 | 外科医生 | 线上39 | 本地区任意正面角色，不限本方；费用及横置由发动者本人承担。 |
| LC20 | 心理咨询师 | 线上39 | 进场源当前controller发动，本地区任意正面角色；无本方限制。 |
| LC21 | 称职的保镖 | 线上39 | 护卫1按对抗中的敌方/队伍分配；拥有者不定义敌我。 |
| LC22 | 退役军人 | 线上39 | 护卫1按敌我队伍；公开不限制其他效果翻暗。 |
| LC23 | 安格鲁，“荆棘” | 线上39 | 自称只指该instance，另一本地区目标不限阵营；发动者是controller。 |
| LC24 | 林中女巫 | 线上39 | 预测2及抓1均由进场源controller本人进行；不共享队友牌库。 |
| XQ49 | 葬礼 | 线上39 | 任意玩家墓地目标；OwnerDeckBottom按拥有者，后续Draw(Actor)按发动者，不能混用。 |
| JC118 | 停战协议 | 线上39 | 目标地区Any，无角色阵营限制；地区效果归原frame。 |
| JC002 | 巨石阵看护人 | 线上39 | 本地区任意正面角色，目标复验含屏障的当前controller敌我。 |
| JC003 | 帷幕护卫 | 线上39 | 目标角色或暗藏者Any；费用2及横置自身按发动者，不能由队友代付。 |
| XQ03 | 力场束缚 | 线上39 | 目标正面角色Any，敌方屏障由通用目标守卫判定。 |
| JC014 | 公路骑士 | 线上39 | 你=当前controller；移动该instance而非同名或队友对象，2V2机动仅邻区。 |
| JC016 | 执行部精锐 | 线上39 | 敌方按当前controller的对手队伍；队友及暗藏者不算敌方角色。撤回按P18回owner；与BQ022赢区优先仍待裁定。 |
| JZ08 | 幸运新人 | 线上39 | 屏障阻止敌方卡/能力指定目标，按目标当前controller团队；允许友方且不阻止非指定群伤。 |
| JC056 | 地区警员 | 线上39 | 本地区所有暗藏者，非指定集合Any，含双方，不错误限制本人。 |
| JC058 | 坚毅的刑警 | 线上39 | 本地区暗藏者Any；现身中栈实体不可再作场上目标，死亡归owner。 |
| JC059 | 安全保卫部门 | 线上39 | 原图友方，按controller同队，包括队友，排除来源自身/暗藏者；不能改成本人限定。 |
| JC063 | 警车追捕 | 线上39 | 两模式目标均Any；潜伏保留owner/controller并换instance；回手明确OwnerHand。 |
| JC086 | 窃尸人 | 线上39 | 你的墓地为本人；OwnedByActor限定其拥有牌，ActorHand入本人手，当前私有区owner不变量使二者相符。 |
| XQ12 | 暴躁血仆 | 线上39 | 死亡触发属于离场前controller；目标玩家Any，私密弃牌在目标玩家本人手中。 |
| JC092 | 死灵召唤 | 线上39 | 你的墓地角色为本人；OwnedByActor、HiddenInChosenRegion，换instance后owner/controller归本人。 |
| DQJC107 | 沉没的废墟 | 线上39 | 每位玩家分别Search(Context)，检索与洗牌仅操作各自牌库，不按赢区者或团队汇总。 |
| DQJC112 | 纽约 | 线上39 | 各Context玩家独立私密弃牌并等量抓牌，不共享手牌。 |
| DQJC113 | 切尔诺贝利 | 线上39 | 场上所有正面角色Any，直接群伤，不以owner/controller筛掉队友或自己。 |
| DQJC114 | 上海 | 线上39 | 模式决定后逐位Context抓/弃自己的牌，目标群体为每位玩家。 |
| JC042 | 末日信徒 | 线上39 | 自称牺牲来源instance；后续降费绑定发动者Actor，仅其下次正面出牌，owner/队友不代领。 |
| JC049 | 深渊细语 | 线上39 | 额外牺牲只能本人controller角色（P15），不是队友；MoveBottomToHand(Actor)仅本人牌库。 |
| JC091 | 谋杀 | 线上39 | 人类目标Any；行动阶段限定与屏障守卫独立于拥有者。 |
| JZ54 | 阿塔玛斯奉献仪式 | 线上39 | 目标玩家Any，包括自己/队友/敌人；由该目标玩家选择其本人controller角色牺牲，墓地回owner。 |
| DQJC108 | 佛罗伦萨 | 线上39 | 自赢区者Actor起逐位Context，仅各自controller的暗藏者可选；免费/忠诚沿用已批准版本FAQ。 |
| DQJC109 | 京都 | 线上39 | 所有非人类正面角色Any翻暗，保留owner/controller；普通结附清理仍回收owner。 |
| DQJC110 | 伦敦 | 线上39 | 所有角色翻暗；混牌按每位controller及地区分组，不能按队伍或owner合组。 |
| DQJC111 | 莫斯科 | 线上39 | 自赢区者起逐位Context，各人牺牲本人controller角色、抓自己的牌；墓地归被牺牲牌owner，防御快照沿用版本裁定。 |
| DQJC115 | 死者之城 | 线上39 | 逐位Context各取本人墓地角色，正面进入同一已选地区；不从队友墓地选牌。 |
| DQJC116 | 香港 | 线上39 | 每人检索自己牌库的附属并入各自手，独立私密承诺，全部选完才共同展示/洗牌。 |
| BQ022 | 合金指虎 | 线上39 | 宿主Any，人类/吸血鬼，不继承宿主controller。卡括注你的手中须按P17回收专门定义回附属owner；赢区组合优先未裁定。 |
| JC084 | 街头混混 | 线上39 | 本方/敌方势力标志由controller选择队伍；P21明确同队共用标志，此处不能机械改为个人。 |
| JC085 | 复生行尸 | 线上39 | 你的资产按controller本人，不共享队友领域；墓地正面出牌仅本人墓地，普通费用/忠诚与instance更新。 |
| JC088 | 职业杀手 | 本地9 | 现身目标Any，同区/印刷费<=2；自潜伏只该instance且本人付费，保留owner/controller，死亡归owner。 |
| JC001 | 通灵猫头鹰 | 本地9 | 你的资产区心灵2按controller本人，owner/队友资产不贡献；不能被装备结附无阵营限制。 |
| BQ083 | 索命骷髅 | 本地9 | 本地区正面目标Any，包括自己/队友/敌方，屏障按目标controller复验；死亡归owner。 |
| JC006 | 逆转异界之门 | 本地9 | 有领域正面目标Any，返回OwnerHand而非ActorHand，普通离场回收按各附属owner。 |
| XQ16 | 法务部律师团 | 本地9 | 53e47ca已将旧FriendlyTeam修为ControlledByActor；另一张本人controller角色，排除源；响应后转为队友控制则原目标失效。 |
| JC047 | 烈火重围 | 本地9 | 地区Any、角色集合Any；群伤无阵营筛选，死亡各回owner。 |
| JC007 | 常春藤派学者 | 本地9 | 你的牌库/手牌均Actor本人；按印刷费用与类别检索，展示后入本人手并洗牌。 |
| JC075 | 雪地救援队 | 本地9 | 本方=ControlledByActor而非OwnedByActor/FriendlyTeam；另一个正面角色返回其OwnerHand，费用2与横置由Actor承担。 |
| JC104 | 占卜师 | 本地9 | 你的牌库=Actor本人，私密Forecast3仅本人排序，既不共享队友牌库也不抓牌。 |

本次过程中父线程原公网权限暂停后，用户明确批准临时访问原已发布站点，父线程已恢复自然UI亲测；本执行者继续仅本地审计，不入其房间、不改域名、不扩大公开范围。此前拒绝记录保留历史边界，不能当作现时仍未授权。

来源：仓库原图、固定原图规格、已发布074def9源码、当前8963fcd卡链源码与上述说明书页；输出是本说明、逐卡JSON、只读导出工具、3项测试、3项实际编译变异及精简53e47ca差异。唯一规则未决仍是JC016＋BQ022的赢区交互；父线程自然UI覆盖另行负责。
