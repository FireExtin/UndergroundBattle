# 响应牺牲案例与通用规则核审查

当前集中修订提交为`d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0`，版本`hegemony-pdf-v1 / limited-v2.1 / rust-v0.2.1`，私有状态 schema 2；可玩25张玩家牌和4种世界牌，香港已隔离。下文`84a2c59`（v0.1）及`8a16cf3`（v0.2.0、30定义）的验收明确作为历史记录，不能代替末尾的新版结果。类型化解释器只覆盖已开放的有限能力，不是685张注册卡或任意多目标/强制触发的完整实现。

## 已查看的原稿和卡图

原《霸权说明书》印刷 P3、P10、P14、P15、P16、P17 已重新查看扫描页：

- 目标及模式在费用前确定，不能在结算时改选；目标已不存在或不合要求时，该卡牌或能力不再生效，费用不退（P3）。
- 费用动作不能被响应；行动费用在冒号前，效果在后；付款后能力入栈，才等待响应（P10、P14、P16）。
- 快速响应后入先出，双方连续让过才结算最上方一个对象，随后重新分配行动权；后手行动步骤使用后手优先的例外（P14）。
- P14 示例二明确：先发生的袭击消灭原目标后，下方基础测试整体不生效，包括其抓牌部分。此处是该 PDF 的规则例示；当前原注册数据中同名109的文字版本存在差别，未将它作为开放卡实现依据。目标失效与关键词“终止”分别记录。
- 牺牲只能选择自己操控的牌；牺牲不算消灭，消灭不算牺牲，但角色二者均为死亡（P15）。
- 能力入栈后与来源独立，来源离场不撤销能力（P10、P16）。费用产生的死亡触发不是“响应”本身。
- 防御降低至0或以下的角色同样死亡（《霸权》P6第6项；《规则手册》物理12/印刷P10）。牺牲、消灭、潜伏、回手或移动移除光环后，稳定点必须重新检查死亡；批量伤害仍保持同时造成与批量移除。

重新查看《规则手册》物理第7/9页（印刷P5/P7）及《勘误及释疑》物理第3/4页；目标失效及翻面响应的说明与上述一致。也重新查看帷幕之后、间奏、序曲、星光俱乐部四份原说明图；间奏的“闪烁”说明确认先暗藏后现身使原指定效果失去目标。上述资料仍未明确单列“多个已指定目标仅一部分失效”时的规则。不要从其他 TCG 自动引入只结算剩余合法目标的处理。

真实卡图已查看：

| 牌 | 原图关键行为 | 当前开放情况 |
| --- | --- | --- |
| JC042 末日信徒 | 费用1；快速能力牺牲自身作为费用；本回合下一张正面打出的鸣钟教派或血牌减费2 | 当前开放；v2.0历史双人、四人浏览器真实响应已通过，新版结果另记 |
| JC091 谋杀 | 行动阶段快速，消灭目标人类角色 | 当前开放；v2.0历史双人、四人浏览器真实响应已通过，新版结果另记 |
| JC049 深渊细语 | 额外费用牺牲一个角色；快速将牌库底两张放手中 | v2开放；原生真实动作验证费用、归属与少于两张的行为 |
| JZ54 阿塔玛斯奉献仪式 | 快速，目标玩家牺牲一个角色 | v2开放；原生JSON状态恢复与WASM验证效果选择；未作JZ54专项SQLite集成或浏览器待选恢复 |
| JC062 调查档案 | 检视至多两个目标暗藏者，然后抓1 | 未开放；多目标与非目标续效的真实用例，部分失效规则仍待裁决 |
| JC021 狭路相逢 | 行动阶段快速，目标本方和/或敌方角色移动到目标地区 | 未开放；多目标、关系和地区目标的真实用例 |

《勘误及释疑》物理第3页明确：深渊细语从牌库底获得手牌不算抓牌；调查档案（旧编号基础055）即使没有可检视暗藏者，也能仅为抓1张打出。后者只确认零目标用法，没有回答多个已选目标中仅一部分失效的问题。

“牺牲契约”未确定对应卡名；已有 JC090 灵魂契约是角色死亡后放影响标记，不能将它误当成牺牲牌。

## 历史基线：v0.1的覆盖与缺口

在历史基线 `84a2c59` 中，JC042/JC091不在受限卡池，只有 LIFO、团队让过、预付费用、实体引用再校验和持久化基础；这不能证明真实牺牲响应。以下是改造前的审查记录，不能作为v2验收数字。

现有精确证据：

- `mobility_first_team_resolves_before_rear_team_and_responses_resolve_lifo`：实际快速响应及逐对象结算。
- `watchers_target_category_locks_and_shield_terminates`：费用横置先于结算，类别与旧实体失效。
- `local_targets_revalidate_and_events_enter_graveyard_after_effect`：地区条件重校验，事务在效果后入墓。
- `teams_require_all_passes_and_own_action_resets_pass_records`：全队让过和行动清除让过记录。
- `simultaneous_death_triggers_are_declared_first_team_then_rear_team`：同时死亡触发的团队声明顺序。
- `sqlite_reopen_restores_identity_pending_choice_and_dedupe`：恢复与幂等；尚不包含牺牲已付款、触发待声明的专门状态。

`Effect::Funeral`当前只有找到并移回墓地目标后才抓牌，目标消失后不抓牌；此前疑点已排除。新增的两个实际 `Game::apply` 绿回归已通过（共28项原生测试，尚未计入正在实现的新核心）：

- `funeral_target_recovered_in_response_cancels_draw_without_refunding_cost`：对方响应翻出窃尸人并恢复同一墓地牌后，葬礼不再移动目标、不抓牌、不退费用，事务仍入墓。
- `response_hide_and_reveal_reenters_with_new_identity_old_spell_cannot_hit`：警车追捕响应潜伏、再响应现身，重新进场使用新实例；原力场束缚不能横置新实例。

这些测试的初始摆放是明确的单元测试夹具，后续均执行真实合法动作；不能称为真实浏览器全程完成的牌局。

## 通用化的实现与审查入口

改造前生产内核有39处显式牌号引用，涉及19张玩家牌和5张世界牌。v2删除了专属 `Funeral/Police/Chase/WorldMode/Death` 效果及选择续体；牌号只在 `rules.rs` 的绑定表和卡牌数据中出现。检查时按 `#[cfg(test)]` 划分测试，不能把单元测试牌号计作生产分支。

`AbilitySpec` 组合时机、费用、目标槽、操作序列、触发及修饰。解释器不比较具体牌号、不解析中文规则文字，不执行任意脚本。有限 `Op` 不是每张牌一个语义相同的专属枚举：区域、实体、玩家和非目标查询作为参数；多个牌共享相同操作并按不同序列组合。

| 规则组合例子 | 声明 | 共享机制 |
| --- | --- | --- |
| JC042 末日信徒 | `Fast + SacrificeSource + CostReduction(SocietyOrMagic,2)` | 牺牲费用、来源快照、通用条件修饰；减费记录行动者、本回合到期、一次消耗 |
| JC049 深渊细语 | `Fast + SacrificeSelectedControlledCharacter + MoveBottomToHand(Actor,2)` | 费用选择与付款；“移入手牌”与 `Draw` 分别建模，符合勘误 |
| JC091 谋杀 | `ActionFast + Board/Character/人类目标槽 + Destroy(Target(0))` | 统一时机、声明绑定、结算入口检查和离场原因 |
| JZ54 奉献仪式 | `Fast + Player目标槽 + SacrificeChosen(Target(0))` | 结算中途由目标玩家选择；保存已通过的检查和操作游标 |
| XQ49 葬礼 | `Graveyard目标槽 + Move(Target(0),OwnerDeckBottom) + Draw(Actor,1)` | 全对象一次检查；自己移动目标后继续抓牌，入口失效则整段取消 |
| LC24 林中女巫 | `Enter + Forecast(Actor,2) + Draw(Actor,1)` | 与其他效果共用暂停/保存/恢复，恢复后不重复预测 |
| 警车追捕、切尔诺贝利 | `ExhaustMatching(BoardSelector)`、`DamageMatching(BoardSelector,1)` | 同一有限查询结构；非目标查询不误用目标护盾/屏障检查，批量伤害同时造成 |

| 审查内容 | 准确源码入口 |
| --- | --- |
| 声明及29个当前绑定 | `rust-game/src/rules.rs`：`Cost`、`TargetSlotSpec`、`BoardSelector`、`Op`、`AbilitySpec`及`definitions()` |
| 合法动作、目标和原子费用 | `rust-game/src/resolution.rs`：`rule_action_candidates`、`valid_binding`、`bind_action`、`pay_printed`、`pay_ability_costs`；`engine.rs`：`Game::apply`及`apply_inner` |
| 入链与一次目标复核 | `resolution.rs`：`make_frame`、`push_frame`、`resolve_frame`；`engine.rs`：`resolve_stack` |
| 触发与稳定死亡检查 | `resolution.rs`：`emit_event`、`declare_trigger`、`choose_declaration`；`engine.rs`：`remove_dead`、`settle_deaths` |
| 续体与待选恢复 | `model.rs`：`ResolutionFrame`、`Declaration`、`FrameChoice`、`ChoiceResolution`；`resolution.rs`：`choose_frame` |
| 持久化/去重/只读回放 | `service.rs`：`Store::command`、`open_read_only`、`audit_replay`；`model.rs`：`Game::from_persisted` |
| 响应UI及自动让过 | `web/src/game/ResponseWindow.tsx`、`Table.tsx`、`AutoPass.test.tsx` |

- 费用：整个动作在克隆状态上完成目标绑定及付款，任何后续失败均不提交克隆；原子性包括资产、牺牲、随机数、实例ID和降费消费。已支付对象入栈，结算和恢复不再次付款。
- 目标：捕获实例ID、类别、地区和操控关系谓词；整个栈对象结算入口重校验，不能只让每个子效果随意跳过。捕获的行动者与现时操控者分别建模。
- 效果：有限操作序列，以目标槽/来源/行动者引用实体；操作序列中途选择保存游标及已经完成的检查，不因自身移牌后再校验而误跳过后续抓牌。
- 触发：区分 Sacrifice/Destroy 等离场原因，二者对角色都产生死亡；保存最后已知信息，稳定检查点声明，不能因来源已消失而丢触发。
- 修饰：保存作用域、条件、到期和剩余次数，例如本回合下一次符合条件的正面出牌减费；不按具体卡名处理。
- 恢复：持久化目标绑定、已付款状态、触发事件、操作游标和待选。去重返回原确认，不重复牺牲、消费减费或声明触发。

开放边界是零目标，或一个目标槽且该槽恰好一个目标（`min=max=1`），包括模式和触发声明；不是任意多目标引擎。多目标部分失效暂不宣称已定；`resolve_frame`的入口失效整段取消只在上述边界下成立，不能据此决定未来多目标规则。新原语和新增整卡必须分开验收。旧引擎/数据库保留，状态格式及固定版本变化需要显式兼容边界，不能静默把旧局当成新格式。

## v0.2.0的历史验证结果及其边界

最终原生测试38项（31内核、7服务），前端62项（12文件），类型检查、构建及格式检查通过。WASM在Node中实际实例化，对照394次转换、1,159个本人视图，原生/WASM字节一致；测试包括真实JC042/JC091动作、JZ54牺牲选择和LC24预测续接。专项原生测试包括：

- `murder_response_sacrifices_disciple_as_cost_and_independent_reduction_resolves`
- `bottom_to_hand_insufficient_cards_does_not_draw_or_eliminate_and_pays_controlled_cost`
- `invalid_additional_sacrifice_rolls_back_assets_modifier_and_every_state_field`
- `effect_sacrifice_private_choice_restores_accepted_guard_and_current_control`
- `source_leaves_after_angru_trigger_target_exhaust_still_resolves`
- `sacrifice_defense_aura_immediately_kills_zero_defense_character_at_stable_point`
- SQLite `paid_sacrifice_death_trigger_and_accepted_frame_restore_without_repayment`：真实JC049费用牺牲由行动者操控、对手拥有的XQ12；分别重启费用已付/死亡触发待声明状态和XQ12的效果弃牌待选状态；重试原commandId不重复费用牺牲或弃牌。JZ54牺牲待选与LC24预测续体另由内核JSON保存/恢复及WASM验证，不称为这项SQLite测试的直接覆盖。

单元测试初始摆放与SQLite测试的显式初始夹具仍是夹具，不能称为全程UI对局。WASM只是共享内核编译/兼容验证，不证明Workers部署完成。

最终浏览器二进制 SHA-256：`56a98031ef54c49f5b8634310a5e8728b4e38f516387e92e1f0c55fd39afc6f0`。最终响应证据位于私有云端 `/tmp/hegemony-response-v2-2026-10-02/final/`，`response-summary.json`记录该哈希；此前`verified/`是中间二进制，不能冒充最终版本。

| 专项场景 | 合法API准备 | 真UI提交 | 去重API探针 | 浏览器上下文 |
| --- | ---: | ---: | ---: | ---: |
| 双人 | 364 | 7 | 1 | 2 |
| 四人2V2 | 1,504 | 12 | 1 | 4 |

专项场景没有状态注入或seed覆盖，但使用合法API推进到所需手牌/场面，因此不是完整UI对局。两模式都在浏览器真实打出谋杀、发动末日信徒快速能力、全席让过仅结算顶层一次、下层唯一pass自动推进、原目标失效不退款且事务入墓。真实快速行动存在时自动让过不提交命令。付费前、付费后链中分别刷新及重启，全部本人投影精确一致；中途相同commandId重试无数据库变化。`browserErrors=[]`；主动重启造成的SSE截断单列为预期网络现象。JZ54没有自然出现在当时手牌，不追加漫长准备或伪造浏览器验证。

主审查者另用最终 `hegemony-audit` 只读重放最终响应房间：

| 房间 | 版本/日志条目 | 持久化与回放共同摘要 |
| --- | ---: | --- |
| `c5c4d94fdeb33c891007b1a8`（双人） | 372 / 372 | `03e955aaf14f2e384023fca5589bcc6b976934f61868f7788040c5b5566f4f6d` |
| `4bfe0df393551342ba6a08ad`（四人） | 1519 / 1519 | `228cd082a7f4e4d311695181f4bd6795169668ffacc0865e9b7a11ce94bd3a96` |

同一最终二进制的完整UI对局另存 `/tmp/hegemony-cloud-playtest-final-v2-2026-10-02/`，包装器不提交API玩法命令，所有ready/start/出牌/选择/让过均由独立浏览器上下文点击，自动让过默认关闭：双人在第11回合以11–7正常得分结束，590次UI提交、0拒绝、0非预期浏览器错误；四人2V2在第14回合以10–7正常得分结束，2,284次UI提交、0拒绝、0非预期浏览器错误。两局都选择不同牌组且包含responders，不能用专项准备场景替代它们。四人实际覆盖12次调查选择、8次伤害选择、4次检索选择及8次弃牌选择；各席独立操作，选择权由服务端指定。

主审查者最终只读回放完整UI数据库，两房间都`matches=true`，报告保留在同目录`root-readonly-audit.jsonl`：

| 完整UI房间 | 版本/日志条目 | 持久化与回放共同摘要 |
| --- | ---: | --- |
| `ac12372251bbf58200a9b8cc`（双人） | 591 / 591 | `e881339ca8f984221eccb1a636da3b81cd3872d31a9080c13bcc5da759e8e5d1` |
| `2522fdb7e9bac5a16ba38cb4`（四人） | 2287 / 2287 | `5c09ebc7fa065eb310b5c80428c9ece12de685e081a3d2dd5b4ceb9ffeee3749` |

响应窗固定显示栈顶、行动权、公开原目标及失效状态；手机实截图已独立查看。完整卡文抽屉和响应按钮可以在390px宽度打开；暗藏目标不泄露身份。`effectiveCost`保留0值，印刷费用仍可见。自动让过默认关闭，只有服务端合法列表恰好一个pass、无本人/他人待选且连接可靠时才计时；真实快速能力、版本变化及新待选取消旧计时。

旧8090/8091服务和v1数据库保留。新默认数据库为`rust-game-v2.sqlite3`；旧状态在启用SQLite迁移/PRAGMA前明确拒绝，不静默迁移。最终v2另保留在`http://127.0.0.1:8098/`（数据库`/workspace/UndergroundBattle/rust-game-v2.sqlite3`），健康检查、页面和30种卡/五套50张的目录已核对。当前没有已验证的Internet入口；该loopback仅供本云端使用，发布由父线程处理。未核定的多目标部分失效规则继续阻止JC021/JC062开放。

## 独立源码审查后的有界修订

父端通过私有GitHub独立读过`8a16cf3`的声明、解释器、命令及事务路径。以下审查修订随后与整池图像对账合并提交；最终范围为25张玩家牌及4种世界牌，不是全部注册卡通用实现。

1. 注册前检查每个能力及其模式：最多一个目标槽，每槽`min=max=1`；禁止有限解释器不能执行的嵌套玩家遍历。错误包含牌号、能力键和模式键。绑定、触发声明及触发选择入口复用检查，删除候选生成的静默跳过。未来多目标卡必须先核定规则、实现相应绑定和选择流程，不能仅添加一个声明。
2. 整段取消的入口检查仅在已开放零/单目标条件下证明。JC062零目标抓牌的原勘误不等于多个已选目标部分失效的裁决。
3. `SourceRegion`当前明确是宣告来源的地区快照。已证实的内容分别是：《霸权》P3及勘误物理P4–P5红眼航班例要求目标结算时仍满足位置条件；勘误物理P3“终焉的诗寇蒂”明确来源已消灭或移动后仍作用于其进场时的地区；《霸权》P10说明能力入链后与来源分离。**这些证据尚未直接裁决所有本地区单目标能力在来源仍在场但横移后的锚点**，不能把诗寇蒂的非目标整区效果推广为通则。
   最小待裁决例：外科医生和有创伤的角色同在地区A；医生支付两资产并横置，指定后者治疗；响应发动红眼航班，将医生移至B，目标留在A。声明锚点会继续治疗，来源当前位置锚点则取消。本例真实卡图分别为LC19及JC117；红眼航班是结附本方秘社的附属，其行动费用为1加横置、仅行动阶段快速，可移动目标本方角色到目标地区。JC117/JC021等外部横移牌继续隔离；已开放的`MoveOnBoard`唯一声明是JC014仅移动来源自身，四张本地区目标能力来源LC19/LC20/LC23/JC002均不具该能力。通用动作也没有自由移动入口，JC063潜伏、JC092墓地召回是换实例/离场，不是相同来源在场横移。因此当前合法有限池不开放该未核定响应组合；增加外部移动牌前必须先裁决，而不能把当前快照推广为完整原游戏通则。
4. **进入地区与进场是不同事件。**《规则手册》物理P7–P8/印刷P5–P6分别列定义；《霸权》P15的进入地区方式和P17的移动说明亦只将移动称为进入该地区。吸血蝙蝠群的原图文字是“进入一个地区”，林中女巫是“进场触发”。修订为真进场/现身发出`Enter`和`EnterRegion`，场上移动只发出`EnterRegion`并保留实例/状态，不重复独有检查；同地区移动不发事件。当前JC014是唯一使用移动操作的已开放牌，且无进场能力。新增测试为有限操作单元夹具，不伪称LC20拥有移动整卡能力。
5. 《霸权》P10明确普通触发可选择是否发动，标“强制”的必须发动；P9赢取流程第5条明确世界赢取触发可拒绝。当前13张触发相关卡原图（LC20/23/24、JC002/056/086、XQ12、JC014及五种世界牌）已逐张核对，均无“强制”。因此当前可选声明成立，但不支持所有原游戏的强制触发；真实反例为霸权灭世魔和亡嚎食尸鬼，继续留在未开放池。发动触发后，效果内必须抓牌/弃牌等不能因此任意拒绝。独有牌强制牺牲和护盾强制消耗由其他已实现的规则路径处理，不混为可选声明。
6. 重新查看LC19原卡图发现治疗费用确为两资产加横置。公开关键词原来已在绑定表尾设置，没有旧局非法秘密派遣；修订将其放回LC19声明旁便于审查，补足`ExhaustSource`及展示文字。旧完整UI双人只有一次JC003发动，四人无主动发动，均未发动LC19。费用修正仍是实际语义变化，升为`rust-v0.2.1 / limited-v2.1`，schema仍2、默认独立`rust-game-v2.1.sqlite3`；旧v2.0状态由固定版本拒绝，不让旧回放静默漂移。新版原生/WASM及完整UI结果待下一验收段。

依父端要求随后集中逐张对账全部30张开放原图，按费用/符号、时机、目标范围、可选/强制、关键词、触发及顺序核对，没有只用JSON牌文代替图像。除了LC19横置费用，另一项差异为DQJC116香港：原图要求各玩家检索附属牌，同时展示并入手，然后洗牌；旧`Search`先洗牌且逐人公开。当前没有附属牌，空检索不证明完整行为。因此新版隔离香港，从可玩注册表、声明绑定和世界牌堆撤下，不添加未验证的同时检索框架。其原图校验保留用于后续复核；其余29定义未发现新的原图/声明差异。

`limited-v2.1`可玩范围为25张玩家牌及4种世界牌，五套50张玩家牌组不变；10张自组世界牌堆为沉没的废墟×3、纽约×3、切尔诺贝利×2、上海×2，并不称为官方预组世界牌堆。香港、外部横移牌、多目标和强制触发卡均不通过“先开放后猜测”补齐。

LC19专门回归先在隔离的历史`8a16cf3`源码上运行，确实红灯：一项测试失败于`assert!(g.board(&doctor).unwrap().1.exhausted)`；公开限制及正常派遣此前检查均通过。不是凭新测试的存在声称见过失败。集中修订提交后，该同一回归纳入最终整套原生检查，验收结果另记。

## 固定 d94baaf 的 v0.2.1 最终验证

本段只记录集中修订后的固定源码`d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0`。在全新独立`CARGO_TARGET_DIR=/tmp/hegemony-d94baaf-final-target`构建，避免历史红测试的共享target缓存；原生42项全部通过（35规则、7服务），其中LC19原有失败回归转绿。纯核无native feature、native binaries及格式检查通过；前端未变，其62项测试/12文件及类型构建结果沿用本次已有检查，没有重复跑完整浏览器局冒充新增前端覆盖。

Node实际实例化新版WASM，与原生逐步比较397次转换和1,165个玩家视图，状态及投影全部一致，保留opaque state、最大u64种子精度和旧patch拒绝。明确的合成初始布局夹具包括LC19治疗付款/横置、JC042/JC091响应、JZ54牺牲待选及LC24预测续体，不能称为无注入浏览器对局。

| 最终产物 | SHA-256 |
| --- | --- |
| `/tmp/hegemony-d94baaf-final-target/debug/hegemony-server` | `148d703b10fb20978b5e4f05345825c5e71a1d22c9c6b3b64080bc3e09d0f2cc` |
| 同目录`hegemony-audit` | `e71937782d62dab5d7cdf055af63b51f9e296368520152b0afc10de0e5e76409` |
| `rust-game-wasm/pkg/hegemony_wasm_bg.wasm`（1,040,341字节） | `a6366ef6bc7df03ba666b33286d3a01c412ac6bf779d6396fcbf6affcdc43966` |

证据保留在私有云端`/tmp/hegemony-d94baaf-final-{native-tests,pure-build,native-build,wasm-build,wasm-parity}.log`及`/tmp/hegemony-d94baaf-final-wasm-fixtures.json`。主审查者独立核对三份产物哈希，并使用最终服务另外启动持久预览`http://127.0.0.1:8101/`，数据库`/workspace/UndergroundBattle/rust-game-v2.1.sqlite3`；`/api/health`确认rust-v0.2.1，`/api/catalog`确认29定义、香港不可用、五套各50张。旧8090/8091的rust-v0.1.0、8098的rust-v0.2.0仍健康，旧DB未迁移。该入口仅在云端loopback可访问，尚无已验证的Internet试玩入口。

### 新版完整 UI 对局

最终完整UI验收每模式只跑一次，固定binary与上述哈希一致，源码记录为d94baaf；自动让过保持默认关闭。包装器仅管理自有8099服务并记录证据，游戏API准备命令为0，创建后join、ready/start、出牌、所有待选及让过均由真实浏览器操作。两个模式均正常得分结束：

| 新版完整UI局 | 牌组 | 终局 | UI接受命令 | 拒绝/非预期浏览器错误 |
| --- | --- | --- | ---: | --- |
| 双人，2个独立上下文 | watchers / responders | 第10回合，11–7，目标8 | 545（542游戏循环） | 0 / 0 |
| 四人2V2，4个独立上下文 | watchers / hunters / keepers / responders | 第12回合，团队0为3、团队1为11，目标10；个人3/0/3/8 | 1,873（1,868游戏循环） | 0 / 0 |

每局故意断线恢复的预期网络错误各1次单列。四人实际选择包含7次伤害、6次调查、6次回牌排序、4次检索、7次弃牌及5次触发，另有10次先手特权和1次主动能力；双人执行了调查、检索、弃牌、回牌排序及触发。两局均在再调度待选时实际刷新/断网恢复。服务正常退出码0后只停止自有8099，没有重跑长局。

主审查者在服务停止后使用最终audit，只读重放完整DB，两个终局均`matches=true`，种子+journal精确等于持久状态；日志条数包含加入房间等记录，不等同UI游戏命令数。实际查看手机双人终局及四人终局截图，结束/得分反馈明确。

| 完整UI房间 | 版本/日志条目 | 持久化与回放共同摘要 |
| --- | ---: | --- |
| `73c0fa78cc67e4c827633047`（双人） | 546 / 546 | `e5bd8cec4323a57e310fbb33212a84c4c90a2122f637a698acf3b03ff11b0fe4` |
| `da677bd1327c6e8e844bd761`（四人） | 1876 / 1876 | `b9a7c19d29588b287546170fedaffd24d18ef456461294f1cfb15b66f5628b40` |

证据在`/tmp/hegemony-cloud-playtest-v2.1-2026-10-02/`：`binary-and-service.json`、`source-build.json`、两份summary、全部UI动作trace、手机/桌面截图和`root-readonly-audit.jsonl`。这是脚本通过真实UI的云端验收，不能称为用户或其女朋友的真人新手体验。

### 新版响应专项：保留通过和失败边界

新双人专项按约定在准备超过15回合时停止：753个合法API命令、0个响应UI命令。双方已有JC042/JC091，但准备策略给缺色保留资产槽，双人一方不能建第三资产，另一方等待对手资源后才派遣，无法形成预期场面。诊断和失败DB保留，不改seed、补牌、改策略再盲目重跑；这不是双人响应验收通过，也没有引擎死锁证据。历史v2.0的双人响应通过记录仍单独保留。

新四人专项一次通过，四个独立浏览器上下文：1,652个合法API准备命令、12个真实响应UI命令、1个命令去重API探针，均无状态注入或seed覆盖。真实链验证手机快速响应可达且自动让过不抢走该行动、费用牺牲后原目标已离场、四席让过只结算顶层、下层唯一pass自动推进、谋杀整段失效但3资产不退且入墓。链前及支付后分别刷新与重启，全部本人视图精确一致；非预期浏览器错误0，8次重启SSE截断明确单列预期。JZ54浏览器效果待选本次仍未覆盖，不扩大声明。

主审查者使用上述最终audit二进制，分别只读重放已停止的专项DB，结果均`matches=true`。第一行只证明失败准备的状态与日志一致：

| 新版专项房间 | 版本/日志条目 | 持久化与回放共同摘要 |
| --- | ---: | --- |
| `e1b481876b9b5d15f1020876`（双人准备未达场景） | 754 / 754 | `f8341e4ca186d285c7c213ed7a630bdceeea74c22cf47f60a5a602aaaa743e2c` |
| `3c8b2d5970ccd1ff08f2e174`（四人响应通过） | 1667 / 1667 | `430033db63181d8b73d7fa5658fd1eeb22d24fcbd615df61f4ac1179a19ebeff` |

报告分别位于`/tmp/hegemony-response-v2.1-2026-10-02/root-readonly-audit.jsonl`及子目录`teams/root-readonly-audit.jsonl`。主审查者也实际查看新版手机快速响应抽屉与原目标失效截图，关键响应提示和按钮在可见视口内。

父端随后明确授权测试专用固定seed及逐步合法命令序列，代替随机长准备。新工具`response_fixed_seed.py`及`wasm_new_lobby.mjs`使用已验证的同一WASM `Game::new`生成全新host空lobby；在自有服务停止且该室无join、hand、deck、board、commands或journal时，仅替换初始seed/random并保留HTTP创建的真实凭证。没有改生产随机接口、规则或中途牌面。该测试明确`testOnlyFixedSeedLobbyBootstrap=true / seedOverride=true / intermediateStateInjection=false / countsAsCompleteUiGame=false`，不是无夹具完整局。

固定seed来自只读历史成功双人初局；其364条准备prefix在新版实际HTTP逐条通过，合法动作/choiceId及每次before/after版本都吻合，至第8回合形成真实谋杀→末日信徒局面。随后2个独立浏览器执行7次UI命令及1个去重API探针，前述双人响应、付款前/后刷新和服务重启、原目标失效不退款均通过；非预期浏览器错误0，4次重启SSE截断单列预期。这是一次定向补验，没有随机重跑完整长局，原失败记录未覆盖。

主审查者使用最终audit再只读检查这份已停止的DB：房间`244e2793513b55bd4b515f3e`，版本372 / journal372，`matches=true`，共同摘要`9e74f35c600c5627696e533cc3fe26298f47b907dec0248e872539a4d6950119`。报告、364行逐步校验、实际UI trace和手机截图位于`/tmp/hegemony-response-v2.1-fixedseed-2026-10-02/`；主审查者实际查看快速响应抽屉和原目标失效两图。测试私有seed文件不入Git，不更换生产公平洗牌。新版双/四响应专项由此均有通过证据，但准备方式分别如实标注。

### Sites 部署能力调查结论

本环境的Sites原生工具已发现且只读`list_sites(role=owner,limit=50)`成功：返回0个拥有的Site，无后续游标；本项目没有`.openai/hosting.json`或Worker房间适配器。尚未注册、保存或发布Site，没有生产URL。调查并非工具缺失或连接失败。`SITES_MANAGED_LINUX_CONTAINER`未设置，按Sites说明属于portable环境。

当前Rust原生Axum/SQLite服务不能直接作为Sites服务器上传：Sites托管服务器要求Cloudflare Workers兼容输出。已通过的共享WASM核是可用的适配基础，不能把本机WASM parity说成已完成托管部署。仍须实现Worker房间HTTP/认证、版本与命令去重、跨实例串行/并发保护、提交后确认的持久化，以及订阅/恢复；持久数据需映射Sites支持的存储，不能把当前内存锁或本地SQLite文件搬过去假定语义不变。之后由Site所有者注册一个私有Site、推送对应源码、保存构建产物并发布，部署状态成功返回URL后才有外网入口。

此结论来自已读取的Sites `sites-hosting/SKILL.md`、`sites-building/references/project-setup/portable.md`及`persistence-and-storage.md`，以及成功的只读连接调用；证据位于`/tmp/hegemony-sites-investigation-2026-10-02.json`。本次责任边界仍为云端实现和验收，发布由父线程持有，不因调查擅自创建另一Site。

父端进一步询问DO绑定。准确状态为**Sites当前未暴露Durable Objects绑定支持**：已读manifest允许字段和存储说明列D1/R2，当前工具没有DO配置入口；这不证明平台内部绝不支持DO。未获明确支持前，不把DO当作可用的Sites部署前提。

| 有界路线，尚未实现 | 最小适配 | 一致性与体验取舍 |
| --- | --- | --- |
| Sites Worker + 同一Rust WASM + D1 | 薄HTTP/认证层；不透明状态、房间版本、命令确认和journal持久化；按版本轮询 | 单房间版本CAS作为裁决点，替代本机房间锁；前台约1–2秒轮询、提交后即刻更新，后台降频。对手反馈有短延迟，不宣称WebSocket/SSE推送；多个房间会共享D1负载，须实测冷启动、限额及延迟 |
| Sites前端 + 独立Cloudflare DO/WASM后端 | 同一WASM；DO房间串行、持久化与推送层；跨域及认证配置 | 房间串行与推送更直接，但需要额外Cloudflare账号、凭据及部署权，当前未获这些资源；仍需提交后确认、幂等、断点恢复和真实部署验证，不能仅靠DO类别保证正确 |

D1路线是待验证的可行机制，不是已通过的部署实现。Worker读取版本v，调用同一WASM计算候选；在一个batch中条件更新`rooms WHERE version=v`并保存候选状态及提交标识，后续journal/原确认的条件插入必须只在该提交标识与新版本匹配时发生。CAS零行不会自动报错或让batch回滚，不能直接无条件INSERT journal/commands；失败候选必须丢弃。命令去重先于版本冲突，使用房间/身份/commandId唯一键，重复返回原确认；落盘失败不能确认。入座等房间变更也要通过同一并发协议。规则计算和PRNG都复用WASM，不重写规则。

官方[D1 batch说明](https://developers.cloudflare.com/d1/worker-api/d1-database/)确认批量语句作为事务，语句失败回滚整体；[读取复制及Sessions说明](https://developers.cloudflare.com/d1/best-practices/read-replication/)说明副本可能滞后、bookmark提供顺序一致性。首版可使用主库读取，或正确传递Sessions/bookmark，客户端也只接受不倒退的版本。上述CAS条件旁写、同commandId并发、失败事务、四席竞态和刷新恢复需要在真实D1/Worker中验收，不能用当前SQLite/WASM结果替代。父端将与用户选路线，当前未同时实现两套。
