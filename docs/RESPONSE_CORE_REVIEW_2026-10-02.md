# 响应牺牲案例与通用规则核审查

历史基线为 `84a2c59`（v0.1）；本次结果为 `hegemony-pdf-v1 / limited-v2 / rust-v0.2.0`，私有状态 schema 2。生产代码已经完成通用化，开放25张玩家牌和5种世界牌；不是685张注册卡的完整实现。本文件区分原稿事实、真实动作、浏览器场景与尚未核定的语义。

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
| JC042 末日信徒 | 费用1；快速能力牺牲自身作为费用；本回合下一张正面打出的鸣钟教派或血牌减费2 | v2开放；已通过双人、四人浏览器真实响应 |
| JC091 谋杀 | 行动阶段快速，消灭目标人类角色 | v2开放；已通过双人、四人浏览器真实响应 |
| JC049 深渊细语 | 额外费用牺牲一个角色；快速将牌库底两张放手中 | v2开放；原生真实动作验证费用、归属与少于两张的行为 |
| JZ54 阿塔玛斯奉献仪式 | 快速，目标玩家牺牲一个角色 | v2开放；原生、SQLite恢复与WASM验证效果选择；未在浏览器自然抽到并完成该专门场景 |
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
| 声明及30个绑定 | `rust-game/src/rules.rs`：`Cost`、`TargetSlotSpec`、`BoardSelector`、`Op`、`AbilitySpec`及`definitions()` |
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

多目标部分失效暂不宣称已定；新原语和新增整卡必须分开验收。旧引擎/数据库保留，状态格式及固定版本变化需要显式兼容边界，不能静默把旧局当成新格式。

## 验证结果及其边界

最终原生测试38项（31内核、7服务），前端62项（12文件），类型检查、构建及格式检查通过。WASM在Node中实际实例化，对照394次转换、1,159个本人视图，原生/WASM字节一致；测试包括真实JC042/JC091动作、JZ54牺牲选择和LC24预测续接。专项原生测试包括：

- `murder_response_sacrifices_disciple_as_cost_and_independent_reduction_resolves`
- `bottom_to_hand_insufficient_cards_does_not_draw_or_eliminate_and_pays_controlled_cost`
- `invalid_additional_sacrifice_rolls_back_assets_modifier_and_every_state_field`
- `effect_sacrifice_private_choice_restores_accepted_guard_and_current_control`
- `source_leaves_after_angru_trigger_target_exhaust_still_resolves`
- `sacrifice_defense_aura_immediately_kills_zero_defense_character_at_stable_point`
- SQLite `paid_sacrifice_death_trigger_and_accepted_frame_restore_without_repayment`：分别重启费用已付/死亡触发待声明状态、效果牺牲待选状态及已经通过检查的预测续体；重试原commandId不再扣费、牺牲或消费降费。

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

同一最终二进制的完整UI对局另存 `/tmp/hegemony-cloud-playtest-final-v2-2026-10-02/`，包装器不提交API玩法命令，所有ready/start/出牌/选择/让过均由独立浏览器上下文点击，自动让过默认关闭：双人已在第11回合以11–7正常得分结束，590次UI提交、0拒绝、0浏览器错误；四人完整局仍在进行，完成后单独补录结果。两局都选择不同牌组且包含responders，不能用专项准备场景替代它们。

响应窗固定显示栈顶、行动权、公开原目标及失效状态；手机实截图已独立查看。完整卡文抽屉和响应按钮可以在390px宽度打开；暗藏目标不泄露身份。`effectiveCost`保留0值，印刷费用仍可见。自动让过默认关闭，只有服务端合法列表恰好一个pass、无本人/他人待选且连接可靠时才计时；真实快速能力、版本变化及新待选取消旧计时。

旧8090/8091服务和v1数据库保留。新默认数据库为`rust-game-v2.sqlite3`；旧状态在启用SQLite迁移/PRAGMA前明确拒绝，不静默迁移。当前没有已验证的Internet入口；loopback仅供本云端浏览器验收，发布由父线程处理。未核定的多目标部分失效规则继续阻止JC021/JC062开放。
