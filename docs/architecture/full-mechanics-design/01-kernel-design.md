# 内核边界、数据模型与执行管线

本文是新规则版本的设计，不是已提交的运行实现。`K1`–`K7` 是共享职责，不要求恰好七个 Rust 文件，更不是七个服务。

## 1. 七个共享职责

| 边界 | 负责 | 不负责 |
| --- | --- | --- |
| K1 实体与生命周期 | 身份、区域、朝向、形态、宿主、拥有者、基础操控权、转移结果 | 优先权、卡文解释 |
| K2 有效属性与查询 | 颜色/领域/类型/图标/能力、持续修正、控制修正、构筑与状态约束 | 主动修改游戏状态、发触发 |
| K3 声明、费用与许可 | 时机、来源区域、目标、模式、X、忠诚、付款、次数、响应政策 | 中途效果选择的 UI 排版 |
| K4 调度、事件与续体 | 堆叠、事件批次、将发生事件、替代/预防、稳定化、触发排序、效果游标 | 网络、数据库、客户端时钟 |
| K5 选择与知情权限 | 个人选择、排序、同时选择、检视/展示许可、按座位投影 | 修改一般规则或决定卡牌合法性 |
| K6 模式与阶段 | 开局、队伍/距离、步骤、对抗、奖励、核心/任务/遭遇协议、胜负 | 把某套预组等同于官方派系 |
| K7 边界与版本 | 存档、命令/收据、恢复、native/WASM 适配、Worker CAS、视图 DTO | 第二份 JavaScript 游戏规则 |

51 项的逐一落点见 [矩阵](02-mechanism-matrix.md)。一张鲜血牌可能经过 K1/K2/K3/K4；“横置”和“迟缓”共享状态，却不共享发生时机。模块数少不代表把语义差异抹平。

## 2. 现有基础值得保留

- `Game::apply`（`engine.rs:321`）复制状态后校验、执行和推进；失败时整个命令不产生任何变化，包括 RNG、序号、费用与日志。继续保持此事务性质。
- `AbilitySpec` 分离 `Timing` 与 `ResponsePolicy`（`rules.rs:7–19,177`）；JC042 证明二者不能合并。明确的 `Immediate` 不是“没有触发”。
- `ResolutionFrame`（`model.rs:407`）保存固定 actor、来源快照、目标实例、已付费用、guard、cursor 和 steps。中途选择后从已接受 guard/下一条指令恢复，不能重收费用或因效果自己移动目标而再次取消。
- `Cost / TargetSlotSpec / Op` 保持有限、类型化、可静态检查。当前禁止多个目标槽及嵌套遍历是已声明的能力边界；在规则与测试齐备前继续拒绝，不能改成静默尽力执行。
- shared Rust crate + WASM ABI、每房间版本元组、Worker CAS/receipt/journal、客户端原请求重试均继续保留。

现在主要问题不是缺少一个能执行任意卡文的解释器，而是这些边界仍夹在 `engine.rs` 的局部分支中，新增牌会跳过同一规则的其他入口。

## 3. 实体模型：一张物理牌与一次在场身份分离

以下为接口伪代码；名称可随仓库风格调整。

~~~rust
struct Object {
    key: PhysicalKey,                 // 私有内部键，不作为隐藏区可追踪的网络 ID
    definition: DefinitionId,
    incarnation: IncarnationId,       // 目标绑定使用；离场/潜伏/现身按规则更新
    owner: Owner,                    // Player(SeatId) 或 World；控制变化不改它
    base_controller: Controller,      // Player(SeatId) 或 Independent
    form: FormId,                     // 正常形态 / 转变形态；不是 face_down
    posture: Posture,                 // Ready / Exhausted
    presentation: Presentation,       // FaceUp / Concealed / SecretPlan
    counters: Counters,
    damage: u32,
    location: Location,
}
enum Location {
    Hand(SeatId), Deck(SeatId), Graveyard(SeatId), Asset(SeatId),
    Region(RegionId), SocietyArea(SeatId), CoreArea(SeatId),
    AttachedTo(HostRef),               // 在场；宿主可为角色/地区/秘社区等
    SealedTo(HostRef),                 // 不在场，通常公开可检视
    Stack(StackItemId), WorldDeck, ScoredBy(SeatId),
}
struct Board {
    objects: BTreeMap<PhysicalKey, Object>,
    order: ZoneOrder,                 // 只存键；牌库/手牌/堆叠/地区顺序
    region_slots: Vec<RegionId>,      // 空间顺序与地区实例身份分开
}
~~~

这是单一权威位置模型，不要求全面 ECS。初次迁移也可以保留当前 Vec 容器，通过 `ZoneStore` 封装读写；但在附属/封印上线前必须消除同一对象由多处容器拥有、或绕过生命周期直接 `remove/push` 的通路。

具体要求：

1. 场上移动保留 incarnation、横置、标志；潜伏/现身会使旧目标失效，并按规则清理附属/封印。转变仅换有效形态，不能假装潜伏或新进场。原文依据：BQ12、BQ16、BQ19、间奏、FAQ5。
2. 宿主关系是有类型的无环图，不能只在牌上加 `attached_to: String`。附属在场，封印不在场；二者在宿主离场/潜伏时分别消灭与回拥有者手。附属操控者不自动等于宿主操控者。角色/附属双类型仍是一个对象。
3. 地区的空间位置与实体身份分离。当前 `region:r` 和 `usize` 在补区后可能指向另一地区；将来会移动/增加任务地区，不能让旧地区目标自动命中新补上的牌。
4. 世界牌不应假定 `owner=0`。赢区后的计分持有者也不是对原始拥有权的改写；遭遇独立状态不属于任意玩家，不能被 `Enemy = controller != actor` 错判。
5. 私有内部 PhysicalKey 不下发给无权观察的座位。离开公开区进入未知顺序的隐藏区后，客户端不能靠恒定 ID 追踪同一张牌。公开历史不撤销，但当前未知位置要重新遮蔽。
6. 未选择秘社也有秘社区，可容纳规则允许的标志/附属（指南 PDF24）。秘社本体不进 50 张，不允许一般移区效果把它移走（BQ5）。

## 4. 生命周期不是一个通用 Move

~~~rust
enum Transition {
    Enter { object: ObjectRef, destination: InPlayLocation, posture: EnterPosture },
    MoveOnBoard { object: ObjectRef, destination: RegionId },
    Hide { object: ObjectRef },
    Reveal { object: ObjectRef, via: RevealMethod },
    Transform { object: ObjectRef, to: FormId },
    Attach { object: ObjectRef, host: HostRef, profile: AttachmentProfile },
    Seal { object: ObjectRef, host: HostRef },
    Leave { object: ObjectRef, destination: OutOfPlayLocation, cause: RemovalCause },
}
struct TransitionResult {
    before: LastKnownObject,
    after: Option<ObjectRef>,
    emitted: Vec<DomainEvent>,
    cleanup: Vec<Transition>,
}
fn prepare_transition(state: &State, intent: Transition) -> Result<ProspectiveEvent>;
fn commit_transition(state: &mut State, approved: ApprovedEvent) -> TransitionResult;
~~~

每一种转移明确：是否在场、是否新身份、是否进场/进入地区/现身/潜伏/死亡、哪些标志清除、横置是否保留、宿主如何清理、触发读取何种快照。规则不同就显式列不同转换，避免越来越多布尔参数。

同时移动一批对象时先制定整个 `TransitionBatch` 的目的地，再计算宿主清理。霸权赢区明确把附属也送各拥有者库底；不能先让宿主离场而把本批本来要回底的附属销毁。未纳入该在场回底批次的封印牌仍按封印宿主规则处理。状态约束在批次后的稳定点求值，不能因为先从 Vec 删除光环来源就在同批其他牌回底之前误判其死亡。

牺牲、消灭、赢区回底、弃牌、封印、伤害死亡是不同原因。统一的是事件与转移机制，不是将它们全部标成 Death。`TransitionResult.after` 同时解决 JC028：费用牺牲后的新墓地实例可被后续效果精确引用；不能搜索同名牌，也不能从 `SourceSnapshot` 复制一张复活。

~~~rust
enum PaidCost {
    Resources(ResourcePayment), Exhausted(ObjectRef), Revealed(ObjectRef),
    Transferred { before: ObjectRef, after: Option<ObjectRef>, cause: CostCause },
}
enum EntityRef { SourceAtDeclaration, Target(Slot, Index), PaidTransfer(CostIndex), Result(OpIndex) }
~~~

`PaidTransfer` 仍需检查 after 当前位置/身份；若在响应中被再次移动，后续效果按经核验的目标规则处理，不能无条件追回。是否支付被替代后仍算完成牺牲，原文未给一般裁定，相关费用组合禁用至裁定。

## 5. 有效属性查询：少量固定层，不复制别家 TCG 的规则层

印刷定义使用 `CardTypes` 集合、`Faction`、`SocietyTags`、`DomainCounts[5]`、永久/临时图标、忠诚需求列表。当前 `kind: String`、单 `magic: String` / `MagicIcon::{None,Blood,Mind,Other}` 不能表示角色附属、五领域 LC15、领域移除及类型变化。

~~~rust
enum IconPurpose { Confrontation, EffectCount, TerrorComparison }
struct EffectiveQuery<'a> { state: &'a State, rules: &'a RulePack }
impl EffectiveQuery<'_> {
    fn characteristics(&self, object: ObjectRef) -> EffectiveCharacteristics;
    fn controller(&self, object: ObjectRef) -> Controller;
    fn icons(&self, object: ObjectRef, purpose: IconPurpose) -> Icons;
    fn can_target(&self, actor: SeatId, spec: &TargetSpec, target: TargetRef) -> bool;
    fn contributes(&self, object: ObjectRef, contest: ContestContext) -> bool;
}
~~~

建议求值依赖顺序：当前形态的印刷值 → 区域/朝向带来的覆盖（资产、暗藏者、封印）→ 类型/领域/能力/操控权修正 → 数值修正 → 当前用途的参与资格与图标口径。它是工程依赖安排，**不是已获原文支持的通用冲突优先级**。

- `Confrontation`：横置者不参与，先手方才计临时图标，暗藏者仅 1 势力。
- `EffectCount`：手册 PDF8 明确效果计算图标包含临时图标，不能调用当前忽略后手临时图标的 `icons()`。横置是否影响某项效果，应由该效果的计数条件决定。
- `TerrorComparison`：序曲专门要求根据被比较角色的操控者是否先手计算临时图标，是前项的印刷例外。
- `defense`、领域与操控权都走同一只读视图，不能在目标验证、灵体和 UI 分别写三套算法。
- 资产的有效名称/类型是资产，文本为空，保留颜色和领域；印刷图像可公开展示不代表印刷能力仍生效。

持续效果描述使用封闭枚举（AddIcons、ModifyDefense、GrantTrait、RemoveDomains、SetTypes、BlankText、ControlGrant 等）和有限选择器。存限时来源、有效区域、过期点及按规则需要的优先级。不支持动态脚本、字符串表达式或任意递归。先不缓存；规模证明必要后，只缓存一次稳定状态版本内的纯查询。

遇到互相授予/取消能力的循环、多个控制授予的冲突、相互设定值的顺序，不能用哈希遍历顺序或“最后写入”自创裁定。规则包需有已核准的冲突表；没有的组合不能开放。查询器检测依赖循环，返回可定位的规则错误，在命令原子边界回滚。

## 6. 声明、付款和次数

声明步骤必须与执行阶段分开，UI 草稿不等于游戏行动。

~~~rust
struct DeclarationDraft {
    offer: OfferId, mode: Option<ModeId>, x: Option<u32>,
    targets: TargetBindings, optional_costs: CostChoices, payment: PaymentSelection,
}
fn quote(state: &State, actor: SeatId, draft: &DeclarationDraft) -> Quote;
fn plan_declaration(state: &State, actor: SeatId, draft: DeclarationDraft)
    -> Result<DeclarationPlan>;
fn commit_declaration(state: &mut State, plan: DeclarationPlan)
    -> Result<FrameAndCostEvents>;
~~~

`plan` 只读检查：来源区域/形态和窗口许可 → 模式、X、合法目标 → 费用和可选追加费用 → 有效费用修改 → 忠诚 → 完整支付方案和次数额度。目标在付款前绑定（BQ4/BQ17），实际提交仍统一复核状态版本。选完目标后才横置资产；不能先付款、再让用户发现没有目标。

`commit` 原子执行全部费用、使用额度、已支付记录、来源转移和入栈/Immediate；费用产生的独立事件保留。忠诚不是被支付/消耗的资源。JC042 减的是后续符合条件的正面打出费用，不替代忠诚、不覆盖追加费用、不覆盖现身。

新增 `ResourcePool` 只在核验到真正产生资源的牌时引入，按步骤清空（BQ12）。若资产具有不同附带状态，应允许玩家选哪张支付；现有自动取前 N 个未横置资产可留作 UI 推荐，不能成为唯一合法付款顺序。数量须用检查运算，X/负费用按已核规则截断而非整数溢出。

次数用 `UsageLedger<UsageKey, Counter>`，不能继续每加一种能力就加 `foo_used: bool`：

~~~rust
struct UsageKey { ability: AbilityKey, scope: UsageScope, period: PeriodKey }
enum UsageScope { Player(SeatId), Physical(PhysicalKey), Instance(IncarnationId), Region(RegionId) }
enum PeriodKey { Turn(u32), Step(StepId), Game }
~~~

建立资产、秘密派遣、特权的个人/团队范围按模式明定；秘社“本盘一次”和先兆“每回合一次”单独设置。原文没说明同名多张/潜伏后是否共享额度时，先做问题记录，不一律按 instance 归零。消耗点默认来自卡级规格中的 `onDeclaration`/`onSuccess`，不能通用假定“被终止就退次数”。

效果中“你可以支付…”是 `CostDecision` 续体，不重新把整张牌声明一次。单次接受付款应原子提交，但此前已完成的效果、已公开信息不能退回。若费用自身的替代需要人选，进入 `PaymentPreparation` 阶段，在真正付款前完成；不支持的会泄密/递归付款组合先隔离。

## 7. 调度状态与执行顺序

~~~rust
enum RuntimeCursor {
    Priority(PriorityWindow), Resolving(FrameId),
    PreparingEvent(ProspectiveEventId), Stabilizing(StabilityBatchId),
    OrderingTriggers(TriggerBatchId), WaitingChoice(ChoiceGroupId),
    AdvancingPhase(PhaseCursor), Finished,
}
struct ResolutionFrame {
    actor: SeatId, source: SourceSnapshot, targets: TargetBindings,
    paid: Vec<PaidCost>, guard: GuardState, program: ProgramId,
    cursor: usize, locals: BoundedResults, parent: Option<FrameId>,
}
fn advance_until_external_input(state: &mut State) -> Result<AdvanceOutcome>;
~~~

通用一次命令管线：

1. K7 验身份、规则元组、收据/版本；复制当前状态。
2. K3 接受一个完整声明，或 K5 接受当前选择。检查不依赖客户端隐藏字段。
3. 对每个待执行原子事件：形成意图 → 收集适用的替代/预防或明确“将发生”触发 → 必要时暂停 → 提交该事件批次。
4. 在定义好的稳定点重新求值，处理致死、无效宿主、独有/领袖等约束，记录事件快照；每波次保留边界。
5. 收集事件观察者，按触发政策形成 `TriggerBatch`；选择是否发动、合法目标、费用及同一玩家的排序；需要时暂停。
6. 帧完成或一项堆叠结算结束后恢复对应优先权；全部让过才结算下一个顶项。阶段不是因为一个 Op 完成就自动推进。
7. 推进到一个可持久化的外部输入点，构建各座位投影，K7 原子提交 state/journal/receipt 后 ACK。

“事件原子性”与“一个命令推进到稳定输入点”不同：一个命令可能包含多个原子事件、多个死亡波次和待选触发；不能把整张卡效果并成一个无内部顺序的大事件，也不能在每次底层 Vec 修改之间让玩家响应。

当前 `emit_event` 仅查事件来源本身的定义；需改为按 `EventKind + active source zone` 查询观察者，支持“另一角色死亡”“本方抓牌”“成为目标”等。一个能力规格要明确：可选/强制、观察区域、事件匹配、目标何时选择、`ResponsePolicy` 和排序组。

普通事后触发不打断一个正在执行的效果；收集并在规则稳定点声明。例外是 JC077 明写“将要受到伤害”的触发，必须挂住待发生事件；是否响应及多个该类触发如何排序仍需规则包裁定，不能偷换成无响应的连续转移。细节见 [语义文档](03-rule-semantics.md)。

稳定化每波先确定适用对象集合，再执行这一波转移，下一波重新求值。这样可以表示失去 JC059 光环后的连锁死亡。死亡观察者是否使用离场前整批快照、濒死旁观者是否也触发，未有通则原文就不得开放依赖该假设的卡。

## 7.1 开局与回合的模式游标

ModeProfile 应固定具体版本的阶段图，不能只存字符串 mode 再散落判断：

| 阶段 | 霸权顺序与选择点 | 原文 |
| --- | --- | --- |
| Setup | 验证并锁定牌组快照 → 世界牌库设置 → 玩家牌库洗牌 → 可选秘社背面放置、同时翻开 → 翻出有序地区 → 按秘社或默认6张起手 → 一次再调度（所换牌先隔离，抓等量，再洗回隔离牌）→ 随机先手 | BQ7；2V2地区数用BQ22 |
| Start | 重置场上牌 → 准备步骤处理相应触发及快速行动 → 抓牌步骤每存活玩家抓1并有霸权允许的快速行动窗 | BQ9；reset引出的触发推迟到准备见RM4 |
| Action | 先手方行动步骤 → 后手方行动步骤；正常行动空栈且有行动权，快速行动服从各优先权窗 | BQ9/BQ15 |
| Confrontation | 阶段开始机动等 → 依地区顺序处理调查、战斗、势力，各自开始/结束窗口、奖励、特权 → 地区全对抗后的声望等 → 处理赢区流程 | BQ9/10/19；追加任务须稳定RegionId |
| RegionWin | 赢区后的快速行动机会 → 所有相关拥有者提交回底排序/合法撤回计划 → 一批移区 → 得到地区计分 → 补区 → 若对局未结束，地区赢取触发 | BQ10；胜利检查遵BQ4 |
| End | 结束步骤触发与快速行动 → 恢复步骤同时移除回合伤害并结束“直到回合结束”效果 → 按有效手牌上限弃多余牌 → 交换先手标志 → 下一回合 | BQ10/BQ20 |

上述是原文顺序；现有有限实现未必含所有节点。每个 StepEnd 有资源池清空等明确钩子。回合末的移除伤害与临时效果结束必须同一清理批次，避免先降防而在伤害清零前误杀。默认手牌上限7还需叠加已核隐秘/暴露等修正，不能在多个选择组件硬编码。

胜利作为 K6 的明确检查点，不能等全部栈清完才检查。霸权达标时立即按分值判定，同达标同分尚未分出胜负，不强行宣布平局。任务替代胜利和淘汰有独立规则；胜负已定后清理/触发是否继续按该模式规范停止。多种同时达成胜利的通用优先次序如无原文，不自动创造规则。

## 8. 有限程序的边界

保留直线 `Vec<Op>`，允许有限 `If`、`ForEach`（冻结合法集合，显式顺序）以及类型化结果引用。增加 `OpResult::{Succeeded,Count,Objects,...}` / `IfSucceeded(op)`，表达“若如此做”而不是用当前状态猜前一步是否成功。

`Draw`、`MoveDeckEndToHand`、`Search`、`Reveal`、`Shuffle` 是不同动作。可复用 `SearchProtocol`、`MysticContestProtocol`、`CoreReturnProtocol`、`SimultaneousChoiceGroup` 等少量封闭协议；每个协议的阶段枚举都序列化。禁止任意跳转、循环、脚本执行和从转写文本自动生成生产逻辑。嵌套遍历仅在确有卡例、固定上界、续体可恢复且验证器覆盖后开放。

卡名/ID 应出现于有限 registry 的数据绑定：LC15 指定江莲/江月、两面牌的 FormId、生成模板、印刷特例。这些是原文要求，不能为了“零特判”改成错误的一般规则。运行执行器不以 `if card_id == ...` 选择付款、伤害、投影或时机。实在独特的新协议要说明原文、输入/输出、恢复点与测试，优先由共享操作组合，禁止为每卡复制整个 `resolve`。

版本内执行预算限制操作数、宿主深度和稳定化波次；两个目标 native/WASM 相同。超限返回显式规则错误且不提交部分状态，保留脱敏诊断供离线复现；不得偷偷终止为平局、忽略触发或续跑不完整状态。

## 9. 具体迁移位置

| 现有位置（固定 SHA） | 保留 | 迁移/替换，何时删除旧路径 |
| --- | --- | --- |
| `model.rs:242–293` Card/Player/Region | owner/controller 区分、公开/私有区思想 | K1 增加世界归属、形态、多类型、稳定地区 ID；先封装容器，附属上线前统一位置写入 |
| `model.rs:364–486` Stack/Frame/Pending | 目标实例、来源快照、已付款、Accepted guard、cursor | 续体和选择组类型化；增加事件/批次/付款结果；新 schema 才替换旧序列化 |
| `rules.rs:28–259` Cost/TargetSlot/Op/validator | 封闭枚举、注册时拒绝不支持程序 | 添明确 source-zone、usage、event subscription、result refs；每扩一个能力同步扩验证器 |
| `rules.rs:308–850` 定义绑定 | 有限显式 registry | 可分主题文件；合法卡名过滤保留为数据，生产登记必须显式接受 |
| `engine.rs:192–320` loyalty/pay/icons/defense/targetable | 已核忠诚与屏障行为 | 支付到 K3，纯查询到 K2，护盾从 guard 内扣标志移到成为目标事件，现有旧房内核不修改 |
| `engine.rs:336–699` action/deploy/reveal/ability | actor 固定、来源快照、非法命令 no-op | 所有来源入口调用 K3/K1；删各自重复费用、取牌/进场分支前先做行为等价回归 |
| `engine.rs:700` pass；`drive/close_window/reward` | 每座位 pass 与单项 LIFO；霸权顺序 | K4/K6 管优先权、Mobility 例外及 ModeProfile；不导入经典 2V2 顺时针模式 |
| `engine.rs:1369–1442` damage/deaths | 同时分配与多波致死思想 | K4 DamageIntent/批次/预防/观察者；删除直接护盾/致死捷径须有恢复与顺序测试 |
| `resolution.rs:11–884` selectors/Op/guard | guard 只接受一次、选择后续接 | K2 查询、K1 转移、K4 事件；来源地区引用从无条件 snapshot 改明确 anchor |
| `resolution.rs:885` take_entity 及 engine 多处 remove/push | 无独立规则可保留 | 收口至 K1，之后仅 ZoneStore 内可修改位置容器 |
| `engine.rs:1677,1831` view/legal_actions | seat 投影、服务端裁决 | K5 TableView/ActionOffer/Quote，逐步移除完整笛卡尔动作枚举 |
| `catalog.rs:11,98` | 当前池与旧房对应 catalog | 多类型/领域/接受清单；未知 ID 拒绝，不从 685 研究记录批量加载 |
| `service.rs`、WASM ABI、`sites/src/*` | 事务、opaque state、CAS、旧内核路由 | 对齐原意图收据契约；schema3 reader/routing 独立支持，旧房仍用旧 bundle |

迁移采用新规则元组下的小批实现，避免并行运行两套当前规则。旧元组内核只做必须的兼容/安全修复；任何会改变对局结果的修改需明确新版本及旧房策略。
