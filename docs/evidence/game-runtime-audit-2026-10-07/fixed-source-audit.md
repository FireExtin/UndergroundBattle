# 游戏运行数据保留、隔离与可恢复清理审计

结论：旧桌无需为用户长期兼容，但当前代码会一直保留其运行数据；退出大厅和忘记浏览器座位不会清服务器。Sites已按roomId、seat令牌及每桌版本CAS隔离对局，没有应用级“所有牌桌轮流执行”的global scheduler；所有桌仍共享同一个D1、Worker执行资源。native每桌有独立锁，但数据库查询/事务共用一个连接锁。现有证据能支持一致性和单桌并发正确性，不能给出多桌容量或吞吐承诺。

最小建议是先增加房主可恢复的“移入回收站/恢复”入口，以及升级时由站点运维执行的“停写→一致备份→事务清五张运行表→验证→恢复服务”。回收站本身不释放数据空间；升级重置移走热库数据才减少热库保留量。永久销毁备份或不可恢复删除继续按父指定当次确认。本次只做设计，没有触碰线上数据。

审计范围：2026-10-07 15:42 UTC，既有普通clone `/tmp/undergroundbattle-shared-cloud`，固定HEAD `fff81ec77bd06106450bb83cdaf1ea73928a3eee`，固定核45/pool42。所有产品读取用 `git show fff81ec:<path>` / 固定树搜索；未提交core A WIP及未跟踪新机制全部排除。没有clone/worktree/index/commit/push，没有执行测试、D1清理、浏览器或运行服务；可用旧本地QA文件仅读取计数。线上D1实际schema、行数、套餐/容量没有查询。下列“D1表”指固定源码定义的应用运行表；平台迁移/内部元数据另属平台，不在清理范围。

## 1. 持久化数据与键

源码证据：`sites/db/schema.ts:3–36`、`sites/drizzle/0000_numerous_lucky_pierre.sql:1–46`、`sites/src/store.mjs:1–49`，均绑定固定fff81ec。`sites/wrangler.jsonc:6`只声明一个DB绑定，所有room使用它。

| 应用表 | 键/索引 | 每桌内容 |
| --- | --- | --- |
| rooms | PK id；唯一索引 rooms_invite_unique(invite) | 每桌1行：id、邀请码、initial_state、当前state、version、attempt_nonce。初态与当前态都是完整RoomEnvelope opaque文本。当前态在成功CAS时整体覆盖；不是每版本另存一个完整snapshot。 |
| seats | PK(room_id,seat)；唯一索引 seats_token(room_id,token_hash) | 2人桌最多2席、2v2最多4席。只保存座位令牌SHA256。 |
| commands | PK(room_id,command_id) | 每个接受的唯一命令保存seat、完整意图hash、完整actor view回执response、该次version；重复相同命令不新增行。 |
| journal | PK(room_id,version) | 每次join或有状态变化的事务1行，entry为Join/JoinWithDeck或SessionEvents；SessionEvents可包含Tick和Command。用于从initial_state完整重放。 |
| entry_receipts | PK(request_hash)，requestHash在整个DB唯一 | create/join稳定请求ID的hash、intentHash、room_id及完整response。response包含原座位的明文token以恢复丢失ACK。因此数据库和备份是私有凭据资料，不能下发给房主下载其他席位数据。 |

commands/journal/seats的复合键以room_id起始；entry_receipts没有源码声明的room_id索引。后续按桌归档/删除宜加 `entry_receipts(room_id)` 索引以避免该表全量筛查。这里仅陈述源码索引，未执行生产PRAGMA或性能测量。

RoomEnvelope包含schema、revision、versions、Game、Pacing（`rust-game/src/room.rs:80–85`）。Game含完整玩家手牌/牌库/墓地/分区、真实实例、seed/PRNG、回合/比分、声明/堆栈/选择/效果队列/控制及修正；Pacing含窗口、每席decision、最后server time（`model.rs:706–745`、`room.rs:64–85`）。这些数据按桌嵌在state内，牌/堆栈/选择没有另建D1表；跨房可重复的小实体ID由各自room state限定作用域。浏览器只收鉴权席位projection，服务器完整状态留在D1。

Native存储不同：`rust-game/src/service.rs:214–218`定义rooms/seats/commands/journal四表，没有D1的entry_receipts；rooms列叫revision，没有attempt_nonce；commands保存seat、expected_version、action及response，没有D1的version/intent_hash列；native token_hash有全表UNIQUE。不要把两种表结构混用。

## 2. 数据隔离与并发范围

Sites每个HTTP请求创建RoomService（`sites/src/index.mjs:25–26`）；RoomStore无authoritative内存房间缓存、无isolate-local mutex（`store.mjs:1`）。读取按room.id/invite；鉴权查询 `(room_id,token_hash)`，hash来自严格格式Bearer令牌，actor由查询得到，客户端不能指定另一个seat（`service.mjs:154–159`）。invite只准入大厅，不授予已有席位操作权限。命令意图hash绑定seat、expectedVersion和normalized完整action（`:180–188`），回执查找也带roomId。

同一桌写由D1批次中的 `UPDATE ... WHERE id=? AND version=?` CAS仲裁，后续journal/receipt INSERT再查该id、成功version及唯一attempt_nonce。join、command、system tick都是这一结构；CAS失败不旁写。不同roomId互不争同一rooms行/version键，相同commandId在不同桌可有独立命令记录。entry_receipts的request_hash全局幂等键要求客户端每次新入席/建桌用新随机ID；同一键异意图409，不能当成跨桌共享的座位凭据。

没有源码级global调度器、所有桌共用的游戏行动队列或某一桌等待选择时锁住其他桌的逻辑。WASM调用同步解析该次输入state，局部RoomEnvelope做转换再返回（`rust-game-wasm/src/lib.rs:22–43,163–206`）；shared的lazy ABI/目录初始化不是共享对局状态（`sites/src/lazy-kernel.mjs:4–20`）。同一Worker isolate的同步WASM计算和所有桌共用DB的资源会形成共享执行负担；这是源码结构的推论，不是远端平台调度或吞吐测量。不能从“无global游戏队列”推出“任意多桌完全没有资源竞争”。

Sites响应5秒超时在该桌的state/command/quote请求时评估并持久化（`service.mjs:73–85,211–244,250–259`，`room.rs:377–414,565–613`）；无后台全局timer/cron扫描房间。无请求时旧桌数据不会因5秒期限被清走。此期限是响应席位规则，和存储TTL不同。

Native有 `HashMap<roomId,Arc<Room>>` 及每桌AsyncMutex、watch sender（`service.rs:49–55,191–196`）；同桌join/state/command拿该桌game锁。全Store共享 `Mutex<Connection>`，查询和事务持锁时其他桌的DB部分也等待（`:254–259,409–465,471–489,511–553`）；rooms map还有短暂全局锁。Tokio并发不是多数据库并行。native启动会读/解析全部房间并把当前RoomEnvelope常驻map；任一旧版本不兼容可能使Store::open失败，不能把它等同Sites按旧桌返回410（`:199–241`）。

客户端也有隔离：API返回roomId必须匹配当前session；切桌abort旧轮询并用isActive过滤迟到结果（`api.ts:112–115,154–161`、`useGame.ts:68–130`）。普通凭据在localStorage，显式独立玩家会话以sessionStorage命名空间隔离；namespace只控制浏览器存储，不是服务器身份（`playerStorage.ts:1–22`）。这些措施不替代服务端token鉴权。

## 3. 保留、增长、已有事实

固定生产代码中没有room/journal/commands/entry_receipts DELETE、truncate、room级TTL、finished清理、created_at/updated_at/last_activity_at，或到期垃圾回收。无法按可靠“建桌时间很久”筛旧桌；version和当前server time不是建桌时间。用户授权后可按全体运行表或明确roomId范围清理，先记录范围与备份。

新桌增加rooms+seat0+entry_receipt；join增加seat+entry_receipt+journal。接受的新命令覆盖rooms.state，并新增command回执和journal；system tick覆盖state并新增journal，不增加成功用户回执；相同重复命令只读旧回执；未改变状态的poll不新增journal。有期限tick时拒绝的用户操作也可能只提交系统journal。Game可见日志上限150条（`engine.rs:154–160`）只限state/view里的文字log，commands.response和journal历史仍保留。

没有周期checkpoint后截断旧journal的实现。重放从initial_state读到末条journal（`service.rs:639–673`），当前state供正常请求直接转换而不是每次重放全部日志。随命令积累，回执完整view及journal记录继续增加；桌数增长也增加初态/当前态及座位。native还有常驻房间state内存。不能声称只存当前snapshot所以数据大小恒定。

可核事实而非容量估算：既有本地engine44自然聚焦final-snapshot文件只读计数为rooms1、seats2、commands120、journal121、entry_receipts2；initial_state UTF-8 1,620B，当前state18,026B，command response文本合计757,901B，journal entry文本26,431B，entry response文本4,832B。仅这些文本合计808,810B；不含其他列、SQLite页面/索引、WAL、平台开销，不能当数据库文件大小、上限或每桌典型均值。未查询线上D1。

固定仓内engine45独立摘要记录一个自然双席桌273唯一命令、274journal、277成功命令HTTP回执含4次重复，turn6/playing（`docs/evidence/jc089-combined-2026-10-07/natural-ui-integrity-independent.json`）；旧engine43一个自然四席合作终局1063唯一命令、1066journal（`docs/evidence/jz48-combined-2026-10-07/engine43-terminal-summary.json`）。两者证明这些本地流程的回执/重放，不证明同时多桌运行容量。未从这些数字外推可支持人数或容量。

## 4. 当前删除/API权限与关联问题

Sites路由仅create/join、state/catalog/commands/quote；events返回410；没有DELETE/reset/restore/list-all rooms/admin入口（`sites/src/index.mjs:27–40`）。Native路由也没有删除（`service.rs:752–768`）。公开站点的seat0是房间创建者席位，不是站点运维管理员；已有代码没有“站点owner账号授权全库重置”的服务端权限检查。

“返回大厅/新建牌桌”明确保留座位（`GameApp.tsx:12–14,27–28`）。forgetSavedSeat只改当前浏览器的seats/session键（`api.ts:69–75`），通常因明确invalid token被调用，不删seats或其他D1行。current-only路由410文案明确旧桌数据仍保留（`kernel-router.mjs:3–5`）。故服务端旧桌会积累，这是实际缺失的生命周期管理，不是Library配额结论。

现有外键全部ON DELETE NO ACTION，不是cascade。直接删rooms在启用外键时会因子表失败；即使关闭约束也会留下孤儿。完整归档/清理至少包含该room的entry_receipts、commands、journal、seats、rooms，子表先于rooms，并在同一原子操作中验证范围。若新增回收站/维护控制表，清理清单也要显式包含或保留其必要控制/操作回执元数据。不要DROP数据库、schema、迁移记录、Site配置、原图、源码或浏览器牌组库来清游戏运行数据。

## 5. 测试能说明什么

本次只读审计不重跑测试。固定tree的实际Worker日志25通过、0失败/跳过；日志明确 `localWorkerdD1Only:true`、provider emulation only，remote Cloudflare D1 acceptance still required（`docs/evidence/jc089-combined-2026-10-07/combined-worker-tests.log`）。607 Web、571 native记录也是已保存的前一固定产品验证，不是本审计新执行或容量压测。

| 现有测试 | 已覆盖 | 未据此证明 |
| --- | --- | --- |
| Sites integration.test.mjs:53–91,174–205 | 同请求并发建房回同一receipt；同桌3个并发join分不同seat；同桌重复命令仅执行一次；同version两ready一成功一409；末席重复join；CAS零行与事务回滚；多种桌共存后重开不串数据。 | 多桌同时完整对战、A桌阻塞时B桌持续进展、负载/latency/RPS、真实远端DB容量。Promise.all主要针对同一桌或同一入席意图。 |
| native session_service.rs:377,424,519 | 同一桌两个Store实例的Begin/timeout竞态、join CAS、队友同窗口Begin并发及重放。 | 多room并发吞吐与DB共享锁影响。 |
| native service.rs:842–904 | 另一个room token不能读取本room catalog；正确本room席位可读取。 | 尚无一套跨房GET/CMD/quote/delete全权限矩阵，delete本来不存在。 |
| PlayerSessions/useGame前端测试 | mocked API下不同会话storage、不确认命令锁定、跨房迟到响应/目录忽略、旧桌恢复/切桌。 | 真实多桌后台负载与服务端并行容量。 |
| storage-retry.test.mjs | 模拟断线/未知commit回执、固定错误分类、有限重试；quota/约束拒绝不盲重试。 | 触发过真实D1 quota或量到账号剩余空间。 |

固定测试树未找到专门多room同库同时发合法动作并校验彼此独立推进的压力/容量测试。构筑的serviceCardCapacity限制是单副牌张数，不能拿来回答房间数或DB容量。当前源代码和记录不足以给“支持N桌”“不会拥塞”结论。

## 6. 最小可恢复delete/reset方案

### 用户房间入口

第一版使用回收站：在rooms加入deleted_at及轻量生命周期修订值；界面提供“仅移除此浏览器座位”和“房主移入回收站”两种明确操作。前者仍只处理本地凭据；后者要求该room的真实seat0 Bearer鉴权，传稳定operationId、当前room版本/生命周期修订值，变更deleted_at并原子记录操作结果。可供同房主恢复。普通成员不能删整桌；invite、猜roomId、另桌token都不授权。

管理操作按seats与room元数据鉴权，不要求旧Game能被当前核解释，故可移除已停止支持的旧桌，无需加载旧核。所有读/加入/command/system tick/entry receipt恢复路径在进入kernel或返回旧receipt前统一检查生命周期；回收站桌返回明确room_deleted，停轮询/停止原命令重试，保留可恢复管理标识。写batch同时校验active状态和生命周期修订值，避免delete/restore与在途写错序。恢复后先重新同步，明确清除作废的未确认动作，旧代次请求不跨恢复生效。全库暂停与此字段均是运维控制，不能改变游戏实体ID、规则队列或历史规则路由。

此阶段保留5类原数据，不打破initial_state→journal重放和原回执幂等；因此回收站不省D1空间。若需要按单桌释放热库，应在停写该桌后把完整5类关联行归档到受控私有备份，再事务移出热库、保留轻量tombstone/操作回执供重复操作与恢复；不要先删token鉴权行后丢失确认/恢复能力。这可以作为下一步，不必第一版引入完整通用归档系统。

### 升级后的站点运维reset

推荐先做受站点owner/运维授权的操作流程，不暴露一个任何seat0都能调用的全库清空按钮。具体备份/恢复工具需先核实当前平台支持与权限；公开Site内的房间Bearer不是站点owner凭证。

1. 展示明确目标project/deployment/DB、5表scope、只读行数、backupId及此次operationId。父已给出的每次升级后清运行数据授权支持执行可恢复reset；当前没有执行指令清线上。
2. 进入持久维护状态并推进运维代次。所有会写DB的批次都检查维护状态/代次，包括create/join、超时poll/system和迟到重试；排空旧在途请求后再备份。仅页面显示维护提示不提供原子停写保证。
3. 在停写条件下制作同一时点5表一致备份，保存schema/engine/pool/源码与部署receipt、各表计数、文件digest、原始opaque文本；不要经JS解析重编码u64 state。先校验可读、hash、关联及在隔离本地库实际恢复演练。备份包含他人手牌/seed/seat凭证，位置只能是已授权的私有平台数据库备份或受控运维存储，不放GitHub、网站静态包或玩家下载。Library401不是DB备份成功，也不能据此绕过权限。
4. 在一事务中按entry_receipts→commands→journal→seats→rooms清运行行，CAS绑定此次维护代次/operationId并保存已完成回执。不能先部分清后宣称完成；失败保持停写与可恢复状态。当前规模能否一批完成要先只读计数和实际恢复/清理演练，不声称生产容量。
5. 验证目标5表空、无孤儿、源码/原图/配置/迁移元数据未改变、备份回执存在，再开放当前核新建房。旧席位收到明确runtime_reset/room_deleted，客户端结束旧轮询与pending，只清失效session；重复reset返回同一操作结果，不再次清掉重开后的新桌。运维代次阻止reset前迟到create在reset后产生意外新桌。
6. 误清复原时再次维护停写、确认当前新桌范围冲突，按同一备份恢复完整5类行、关联、回执和opaque字节；不能把已有新桌无提示覆盖。恢复前核验备份engine/schema身份。当前核严格拒绝旧核state，故可继续游戏的复原需要该次备份绑定的配套Site版本/源一起回滚；这是一次事故回滚，不是长期在新站保留所有旧核兼容。只需查资料时可离线只读恢复备份，不让旧桌混入新核。

Native reset更简单采用停止接收新请求、排空在途/关闭服务后备份并清理、重新Store::open；其内存room map/watch不能在DB清空后继续当权威。若在线做单桌管理，沿用room锁→DB锁顺序，提交后同步移除/标记map和通知watch，避免取得全局DB锁后等待另一个room锁导致死锁。没有必要为了生命周期清理引入长期旧核调度器。

永久销毁/不可恢复清理的当次确认是父本任务给出的边界；回收站恢复和已保留备份reset无需把它误写成每步重复授权。备份保留期限与销毁策略需另有明确选择，不能默认自动过几天永久删。此报告未提出执行任何清线上动作。

### 实施时最少验证

需要补真实本地同库两桌并发测试：同commandId/expectedVersion各自成功且A/B state、journal、回执不交叉；A卡在选择/响应时B继续；foreign-room token对state/command/quote/delete/restore全部拒绝。再测单桌回收站/restore不改B桌、对停写/在途create/join/timeout/重复请求竞态、SQL失败完整回滚、重置ACK丢失只回同一receipt、重置后重复旧operationId不清新桌、备份恢复字节/外键/原回放一致、跨engine恢复明确拒绝或配套回滚。全库行数/响应字节/请求延迟的容量测试另外开展，不能拿这些一致性测试冒充。

## 当前交付与剩余事项

本报告是固定源码和已存日志的独立只读审计，审查者没有修改clone任何跟踪文件，也没有stage、stash、reset或清理其他任务WIP。共享工作区WIP状态在审计期间由其他任务变化，最后HEAD仍fff81ec并可见原core WIP；本报告始终只读固定commit，不归因或干预WIP变化。没有线上表数/数据库大小或多桌负载实测，未核验平台私有备份与owner运维鉴权工具可用性。可据本报告选择实现最小回收站与备份reset，再提交固定候选测试/独立审查；在那之前不宣称删除入口已存在、空间已回收或线上已重置。
