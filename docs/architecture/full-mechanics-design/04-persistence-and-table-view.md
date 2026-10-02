# 持久化、网络节奏、牌桌与卡组库

## 1. 每条命令提交一次完整可恢复状态

现有边界基本正确：服务认证座位，查同命令收据，核版本，调用纯 Rust，得到 opaque state 和该座位 view，再提交 state/journal/receipt，最后 ACK。人类选择之间绝不能保持数据库事务；每个中断点必须已经保存。

应持久化的状态除棋盘之外还包括：

- `versions + stateSchema + modeProfile + acceptedPool`、RNG 状态、实体序号、回合与步骤；
- 堆叠顺序、帧的固定 actor/来源快照/目标/已付款/guard/cursor/局部结果；
- 待发生事件、替代处理进度、同时事件批次、死亡波次、触发顺序与已声明项；
- 每个选择组的候选快照、获知权限、各座位提交情况、未公开答案及后续指令；
- 效果/额度的有效期、控制授予、宿主关系；
- 优先权窗口的逻辑 ID、各座位让过/响应意图、下述联网节奏状态。

重新加载不是“重新算当前能触发什么”。已经付款、已经公开、已经使用过的替代、已经完成的 RNG 和已经接受的 guard 都不能再做一次。局部结果只存必要的有界数据，避免把整张状态复制进每个帧。持久化 schema 升级不能静默丢弃未知枚举。

## 2. D1 与 native 服务契约

`sites/src/store.mjs` 的 CAS 更新带 `expectedVersion` 和不存在 receipt 的条件；后续 INSERT SELECT 由本次 `attempt_nonce` 锁定为同一成功写入。房间状态、journal、receipt 置于一个 D1 `batch()`。D1 文档说明 batch 的语句顺序执行，并在任一语句失败时回滚整个批次；当前设计正依赖这个事务边界。[Cloudflare D1Database.batch](https://developers.cloudflare.com/d1/worker-api/d1-database/)

保留该模式，不能改成三次独立请求或 isolate 内 mutex。恢复语义：

| 情形 | 正确结果 |
| --- | --- |
| CAS 成功但 ACK 丢失 | 原 commandId + 原 intent 重试拿原 receipt，不重复付款、RNG 或推进 |
| CAS 失败且命令已存在 | 比较完整规范化意图；一致则返回收据，不一致拒绝 |
| CAS 失败且命令不存在 | 返回新座位 view/version conflict；让人重新确认新的动作，不悄悄改版本重发旧决定 |
| 批次中任一语句失败 | state/journal/receipt 全无；按已有有限瞬态重试策略处理 |
| 已接受的个人同时选择重试 | 返回该选择的原收据，不替换答案或重复公开 |
| 旧命令收据早于当前房间状态 | 返回原结果用于确认已接受；客户端随后拉最新 view，不能拿旧 receipt 覆盖新 UI |

Worker 已比较包含 seat、expectedVersion、规范化 action 的原有 intent hash；不要增加另一套签名或校验链。native `service.rs:344` 仅比较 seat，然后返回同 commandId 原 response。本次离线 probe 已复现改变 action 和 expectedVersion 仍获原收据。应把 native 对齐到同一“相同 ID 必须相同意图”契约，可以比较数据库原有字段/规范化序列化，不必新加密码学。

native 当前 SQLite 事务加进程内房间 mutex 适合单服务进程。它没有 Worker 那样的 SQL version CAS，多进程同时写同一 SQLite 文件不能声称受此设计支持；部署拓扑维持单写服务，或者另补条件更新。数据库不是 GUI 状态，也不能先发布 SSE 再提交。

`storage-errors.mjs` 的存储退避随机是网络行为，可使用宿主随机；它不进入游戏 RNG。错误分类、有限重试及无私密 SQL 日志应保留。本次没有访问在线 D1 或 Sites 凭据。

## 3. 版本隔离与确定性

固定基线当前元组为 `hegemony-pdf-v1 / limited-v2.2 / rust-v0.2.3 / schema=2`；router 还保留 `rust-v0.2.1`、`rust-v0.2.2` 的对应内核。room catalog 必须来自该房间对应 bundle，客户端全局最新 catalog 不能覆盖旧房。

新结构需要新 schema 和新规则/池/引擎身份。路由层先识别版本 envelope，再交给准确 bundle；现在只识别 schema2 的读取路径也需扩展，不能先用当前内核反序列化未来 schema 才决定选哪个内核。旧房继续旧内核；未知元组明确拒绝，提供可读恢复错误，不创建半空房间。无需自动迁移已开局对战；要迁移必须有专门工具、测试和用户可知的政策。

保存完整 RNG 状态与序号，用排序稳定的容器及显式座位/事件顺序。当前 xorshift64 + `% n` 和 Fisher–Yates 可保持旧房兼容；若新版本要消除模偏差，应升级算法/规则身份并添加固定向量。不得无说明更换 RNG 导致旧 journal 分叉。随机目的地必须先构建同序的非空合法集合。

WASM ABI 继续把内部状态当 opaque 字符串；JS 不解析再编码 u64 seed/random，不复制 Rust 规则。新接口的规则核心不读系统时间、数据库、网络或 UI。计时作为显式受信事件进入下节的会话层，并可重放。

本次在固定 SHA 临时副本执行现有 parity 工具，得到 **417 transitions、1205 projections 全匹配**，包含最大 u64 seed。它证明有限夹具的 native/WASM 一致，不证明未来机制或全部 685 记录等价。

## 4. 已批准的 5 秒响应意图倒计时

这是用户明确要求的联网节奏政策，**不是纸面规则**。最新要求覆盖此前默认手动建议：可响应窗口中，持有优先权的玩家有 5 秒决定是否发起连锁；点击连锁后再选择牌/能力/目标；正式提交之前可取消，已提交付款不可撤销。选牌阶段时限尚未决定，不能擅写成另一个 5 秒。

原稿 BQ15 的优先权必须先实现准确：

1. 多数步骤开始及一项堆叠结算后，先手先获得优先权；后手行动步骤例外，后手先得。
2. 持权者可以连续快速行动，直到自己让过才把权交对手。打出牌并不必然立即交权。
3. 双方连续让过才结算顶项；栈空时双方连续让过才结束该窗口/步骤；结算后重新给权。
4. 触发的产生不是“某人响应了”；触发声明后若可响应，才进入相应优先权流程。Immediate、强制选择、付款中途和同时选择屏障不会各自冒出一个 5 秒响应窗。

**计时范围只含有待响应堆叠项的响应决定。** 空栈正常行动阶段、建立资产/选出牌、步骤本身的手动结束、目标/伤害/触发选择都不因为有“优先权”二字就自动限为 5 秒。某项结算后栈仍非空，重新给权时开新的 5 秒；栈已空则返回该阶段正常流程。若将来要给空栈快速行动窗口或整个回合加时限，须作为另一项产品政策明定，当前用户授权不能扩大为全流程超时。

因此 UI 可提供“确认并交出优先权”与“确认并继续连锁”两个明确意图，或等价设置；服务应将声明与紧接的让过作为有序、可验证动作，不能悄悄自动把对手当下一持权人。单纯点“出牌”的具体默认选项交主线程落实，必须在 UI 文案可见。

建议把规则状态与会话节奏放入一个新版本 room envelope，**同一个权威 revision 和同一次 CAS**：

~~~rust
struct RoomEnvelope {
    revision: u64,
    versions: Versions,
    game: GameState,
    pacing: PacingState,
}
struct PriorityWindow {
    id: WindowId,                    // 每次重新给权/动作改变栈后产生新 ID
    holder: TeamId,
    eligible: Vec<SeatId>,
    members: BTreeMap<SeatId, ResponseDecision>,
}
enum ResponseDecision {
    AwaitIntent { deadline_ms: u64 }, // 服务端开窗时间 + 5000
    Composing { intent_id: IntentId, selection_deadline_ms: Option<u64> },
    Passed,
}
enum SessionEvent {
    BeginResponse { window: WindowId, intent: IntentId },
    CancelAndPass { window: WindowId, intent: IntentId },
    SubmitResponse { window: WindowId, intent: IntentId, declaration: DeclarationDraft },
    TimeoutPass { window: WindowId, seat: SeatId, observed_now_ms: u64 },
}
fn reduce_room(envelope: &RoomEnvelope, authorized: AuthorizedEvent)
    -> Result<RoomTransition>;       // 纯函数；时刻是服务端注入并记录的输入
~~~

不必把整套业务变成新大框架；已有 `Game::apply` 继续负责游戏合法性，room reducer 只处理有限节奏状态与一条命令的原子组合。消除新旧重复 version 字段的权威歧义：新 envelope revision 是 CAS/客户端引用的唯一版本，内核原 `Game.version` 应迁移为这一计数或只作为明确内部派生值。旧房 schema2 保持原契约。

关键语义：

- 点击“连锁”只提交响应意图，**不选定具体牌、不付费、不入栈、不消耗能力次数**。该座位进入 Composing，原 5 秒意图时限停止。按钮重复点击靠命令收据/intentId 返回同一结果，不延长时限。
- 点击“取消并让过”清除草稿并执行该座位 pass。关闭目标选择面板或返回重选不是隐式撤回已付牌，也不应被误识别为取消并让过。
- 正式提交时重新运行完整 declaration 校验；费用、目标、忠诚和窗口都以最新服务端状态为准。失败不付费，返回仍允许的操作阶段或明确窗口已过期。
- `windowId` 不等于 phase 名称，也不只等于 turn；同一阶段可能连续开多个窗口。旧窗口的 Begin/Cancel/Submit/Timeout 一律不能影响新窗口。
- 超时当且仅当服务端在该窗口仍为 AwaitIntent 且 `now >= deadline`，提交系统 `TimeoutPass`。客户端倒计时是显示，不能发送伪造时间来获得延长，也不能直接决定结算。
- Begin 与 Timeout 并发由 CAS 形成唯一顺序；过期后到达的点击拒绝并返回新 view。服务端应在每次处理前处理已到期事件，不能接受“客户端说自己 5 秒内点过”。若另一请求已先赢 CAS，失败请求重新检查窗口而不是重置 deadline。
- 断线重连恢复原 deadline/intent/草稿可继续信息，不重新给 5 秒。Composing 的操作时限没有用户决定前，允许保存恢复且手动取消，不私自自动选牌/付费；反复点击不能绕过未来确定的操作时限。
- 无手牌不能判断无响应：场上能力、现身、墓地/核心/手牌能力及 Immediate 都要由内核检查。可以有本人可见的“没有可用响应”提示；不能把对手候选数或隐藏牌身份暴露出去。是否自动跳过无合法响应的本人窗口可设置，但不能改变对方看到的私密能力信息策略。

默认不向其他座位公开“无合法响应”的原因。若启用无响应自动让过，服务端仍可统一在该窗口正常 5 秒到期时提交，让界面早收起本人不可用操作即可，避免其他人从一个独有的立即跳过信号推知隐藏能力可用性。本人主动点击让过属于公开行动。

并发前提分两类，不能一律重写客户端 expectedVersion：正式游戏声明仍严格绑定被确认的状态版本；纯 Begin/Cancel 意图可以定义为 `windowId + seat decision state` 的窄前提，同时选择可定义为 `choiceGroupId + 自己尚未提交`。服务读取最新 row revision 做 CAS，冲突后仅在同一窗口/选择组、同一座位状态且候选快照未变时重试原意图；请求原文和 commandId 不改。这避免队友的同窗点击令另一人的 5 秒请求无谓失效。任何棋盘变化、旧窗口或正式目标/付款变化都必须重新确认。新命令字段必须进入规范化意图比较，不能在现有白名单中被丢弃后误视为同一命令。

处理顺序先查原收据，再判断当前时限：已在到期前接受的意图丢 ACK 后重试仍返回原收据。第一次到达的过期意图不能借此延长；非法正式声明也不重置任何期限。

2V2 中两位存活队友各有自己的 AwaitIntent/Composing/Passed。只有双方都 Passed 才让整个队伍让过；一名队友超时不能代替另一名正在编辑的队友。可以同时编辑，但第一条正式声明被接受后会改变游戏状态、重置按规则需要的 pass 并生成新窗口；另一草稿只能重新 quote，不能自动提交旧目标。两位队友的牌/能力列表彼此不可见，公开提示只表达谁在操作或等待，不能显示“有哪张牌能连锁”。如果产品选择同队轮流编辑，也必须保留另一人的响应机会，不能用互斥锁剥夺规则权限。

Worker 的可靠性边界必须写清：D1 不会因为某行 deadline 到点自动唤醒 Worker；普通请求中的 `setTimeout` 不能充当持久化计时器。可先由现有状态轮询/心跳和任何命令请求驱动 `expire_due_windows(now)`，用相同 CAS 提交系统 pass。在线客户端会在到点后下一轮请求推进；无人在线时不会保证恰好第 5 秒物理写库，重连后先补处理已过期窗口。若以后要求全员离线时也准点推进，再由主实施线程引入可靠调度/队列/持久 alarm；本设计不使用 Sites 凭据或擅增服务。

时间事实（开窗时刻/期限及 TimeoutPass 输入）进入同一 journal，重放使用已记事实而不读当前时钟。新窗口不要无请求连续预演多轮超时直到整盘结束；恢复处理按窗口顺序推进到当前应显示的稳定点，并给新创建窗口新的真实时限。

## 5. 结构化牌桌 view model

用户已批准电脑横屏俯视二维牌桌方向，无需兼顾手机。此处只规定信息和操作，视觉尺寸、透视、卡面密度和具体布局由主线程决定。BQ8 的开场示意体现：每席个人资产/秘社/牌库/墓地/手牌，中央有序地区和世界牌库；它不是后端按钮列表。

~~~typescript
type TableView = {
  revision: number; viewer: SeatId; mode: ModeSummary;
  seats: SeatView[];                   // 固定逻辑 seat，含队伍、公开个人区
  regionOrder: RegionId[]; regions: RegionView[];
  world: { remaining: number };
  private: PrivateSeatView;            // 只有当前身份获准内容
  stack: StackItemSummary[];
  priority: { windowId: string; holder: TeamId; members: MemberStatus[];
              myDecision?: DecisionView; serverNowMs: number };
  choices: ChoiceView[];               // 只属于本人；他人只给安全等待提示
  offers: ActionOffer[];
};
type ActionOffer = {
  id: string; source: VisibleRef; label: string; timing: string;
  responsePolicy: 'respondable' | 'immediate';
  modes: ModeSummary[]; steps: DeclarationStep[];
  costSummary: CostSummary; availability: Availability;
};
~~~

各席视图是 `seatId → screenAnchor` 的呈现映射，不能通过旋转 UI 改变区域索引/距离/队伍。支持以自己为底部看桌面，但“最近三地区”和相邻关系始终用逻辑拓扑；地区移位/新增任务后仍指同一 RegionId。公共实体显示 owner 与 controller 的必要区别，附属呈宿主关系，封印呈可检视容器，私人手牌不和队友混堆。

`KnowledgeGrant { viewer, objectOrSet, scope, expiresAt }` 是被规则授予的看牌权限，不是 UI 的“翻开卡面”布尔。自己的暗藏者可看印刷卡面但有效属性仍是暗藏者；对手不应收到 cardId/text 再靠 CSS 隐藏。检索候选、弃牌手牌、同时选择答案、未来先兆来源和日志均通过同一投影器。调用动作失败的错误也不能告诉无权者“实际是某张牌”。

当前 `frame_view` 假设来源定义公开；增加手牌/背面计划能力时必须明确何时公开来源。不要只隐藏棋盘后遗漏 stack、pending.title、日志、收据、SSE、错误和 catalog 关联字段。

## 6. 人类的声明和中途选择

当前 `legal_actions` 对来源/模式/目标/地区/费用组合先生成，再 clone/apply 验证；`Table.tsx` 将很多完整动作显示成按钮。这在少量单目标牌可工作，完整机制会成组合爆炸，而且“点了目标卡却出现施法按钮”不符合桌面操作。

改为：点击自己的来源 → 选择动作/模式 → 在桌面高亮目标 → 选择费用与可选追加 → 看完整确认摘要 → 提交。中途选择由新 `ChoiceView` 单独引导，不从头选能力。`quote` 随部分输入返回下一步合法候选、费用变化与简短不能执行原因；不产生游戏事件、不改变 RNG，服务端提交仍复核。

~~~text
JC003 护卫 → 发动：横置目标
  选择：一名角色或暗藏者（原图没有“敌方/本地区”限制）
  将支付：2 资源，横置护卫
  确认后：效果进入堆叠，对方可响应

JC042 末日信徒 → 牺牲自己，获得本回合下一次指定类别减费
  将支付：牺牲此角色
  确认后：减费立即生效；相关独立触发另行提示

香港 → 从你的牌库选择附属 → 已提交，等待其余玩家
  所有人完成后：同时展示/入手，再洗牌
~~~

选择组件根据语义渲染：调查是“排序顶部/底部后抓一张”，伤害是拖配数值且显示剩余额度，宿主是在牌桌上选可结附对象，隐秘对抗只展示本人手牌，独有冲突是“保留哪张/牺牲哪张”的明确决策。不要把 `{op, targetId, slot}` 调试字段暴露成产品文案。

响应提示用“轮到你决定是否连锁，剩余 4 秒”“你正在选择连锁动作”“等待队友”“双方让过，结算 X”，区分没有优先权、不可响应、正在解决选择和已经提交。草稿取消不改变棋盘；支付后的动作显示已入栈，避免暗示可撤销。

## 7. 卡组编辑、保存和开局选择

新增用户要求是可编辑/命名保存的个人卡组库，超出当前五套预组选择。先对已验收卡池开放真正的编辑流程，未实现卡可只读浏览但不能出现在可开局卡组中。不能把 685 研究目录直接变成选卡列表。

~~~rust
struct SavedDeck {
    id: DeckId, name: String, revision: u64,
    format: FormatId, rule_pack: RulePackId, pool_version: PoolVersion,
    society: Option<DefinitionId>, entries: Vec<(DefinitionId, u32)>,
}
struct MatchDeckSnapshot {
    format: FormatId, versions: Versions, society: Option<DefinitionId>,
    entries: Vec<(DefinitionId, u32)>, original_deck_revision: Option<u64>,
}
fn validate_deck(deck: &SavedDeck, catalog: &AcceptedCatalog) -> DeckValidation;
~~~

校验分层但只保留一个权威验证器供 native/WASM 共用：

1. 定义/产品版本/正反面身份有效，数量正整数、总量和对象上限合理；生成物、转变面、参考记录不能独立加入。
2. 模式最低张数、同名跨版本上限、唯一构筑及印刷不限张例外。
3. 选用秘社是可选的独立项，不计牌库数量；应用该秘社的颜色/类型/比例限制与开局手牌覆盖；不擅加中立牌例外。
4. 每项必须在该房间规则包/池的显式接受清单中，所有所需机制已验收。研究图已核验、局部支持不等于可选。
5. 加入房间/开局时再次校验并复制不可变 MatchDeckSnapshot，原个人牌组后续改名/编辑不会改对局牌库或 replay。房间使用其固定 catalog，不要求旧卡组强制升级。

现在 `deckId` 只指内置预组；先允许服务器把预组解析为同一 snapshot，再添加自定义 deck 的引用/规范化内容入口。玩家提交的卡表始终由服务器重算，不接受客户端的 `isLegal=true` 或总数。大厅可编辑和显示具体错误；准备后锁定，变更需先取消准备并重新快照，开局后拒绝修改。是否向对手公开完整卡表按模式政策，默认不因为服务端保存就公开。

无 ChatGPT 登录前提下不扩建账号体系：第一阶段本机 IndexedDB 保存卡组库，支持版本化 JSON 导入/导出作为用户可掌控备份；本机身份不能假装跨设备身份。开局用现有房间座位认证保存 match snapshot 到 D1，恢复房间依靠已有会话/恢复凭据；丢失本机卡组可从用户持有的导出文件恢复。

如果主线程需要跨浏览器云端个人卡组库，沿现有匿名凭据模式增加独立 `libraryId + 高熵恢复令牌`、服务端只存令牌散列、每次读写按该库验证；令牌不等于房间座位 token，也不公开给对手。无邮箱找回承诺、无 ChatGPT 身份假设。恢复链接/码必须明确由用户保存，丢失不可冒领；这不是新账号平台。是否采用该云端库是产品范围选择，当前请求的“保存”可由本机库完整满足，但必须清楚标明保存位置与跨设备限制。

卡组编辑不是响应倒计时的一部分，不能用一张未验收卡“先保存合法再进房”。旧版卡组可以保存为待修复草稿，显示版本/缺卡原因；只有当前所选格式通过校验才可开局。未来规则更新不自动重写用户原卡表。
