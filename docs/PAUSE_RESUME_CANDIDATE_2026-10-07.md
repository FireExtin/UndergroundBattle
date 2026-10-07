# 保存一桌，以后继续：暂停/恢复候选 engine46

输入是用户2026-10-07明确更正的目标：通常一次打一桌，保存未完成对局，约好时间继续。父要求先独立完成已审 UI/S2 发布，再以有测试的小批实现暂停；服务器5秒决定时限和客户端持久 AutoPass 偏好均须停用。起点93aa78240b7499c94a5cdd2c11dca3ad415dda55；Site39保持已审 engine45/pool42在线，本候选尚未发布、未清理任何线上数据。

输出为有限 RoomEnvelope 会话操作 `pauseRoom` / `resumeRoom`，以及独立 `RoomManagement` 控件，复用原有座位凭证、多桌索引、commandId 收据、CAS 和 SessionEvents journal。所有已入席玩家（包括仍持有原席凭证的淘汰玩家）都可暂停/恢复进行中的对局；未认证观众和另一桌凭证不能操作。没有账号找回、全局调度、通用绑定或游戏队列；Lobby/finished不能暂停。

暂停是已确认命令边界：付款、堆栈、手牌实例、当前选择、已声明的 composing 意图保存。未提交的 React 编辑/选项选择不是存档。暂停标记含可信服务器时间和执行席位，读取/轮询不调用超时推进，返回冻结时间、空普通动作和不能开始响应的窗口。普通游戏/Begin/Pass/Cancel/Submit/quote拒绝。原命令收据仍先于时间计算，重试 Pause/Resume返回原响应；换 payload/版本复用同一 ID 拒绝。

恢复只把每个尚未决定成员的 `deadline - pausedAt` 加到恢复时刻，不补离开时长，不自动推进阶段，不重新付费。composing意图原样保留。复用原窗口序号生成新 windowId，防止暂停前晚到的窗口命令进入恢复后的窗口；普通付费/选择动作及暂停/恢复本身要求准确revision。暂停请求若到达时旧窗口已到期，先正常提交到期 Tick，再拒绝旧revision的暂停；用户同步后重新点暂停，不把已经到期的决定倒转。溢出/无效暂停记录原子拒绝。

客户端暂停时取消 AutoPass计时器、冻结倒计时、停止当前桌轮询及 visibility自动唤醒。保存列表里未进入的桌本来就不轮询。伙伴恢复后可点“查看最新状态”或刷新；恢复时重启当前桌的正常轮询。使用原浏览器的“我的牌桌”或同一邀请码恢复原席；邀请链接仅含 room定位码。普通模式凭证留在localStorage，独立模式sessionStorage关标签可能丢失，无跨设备账号找回。普通返回大厅/关页面不会自动暂停；UI明确提示先暂停再离开。

选择弹窗内也提供暂停按钮；暂停后收起该弹窗，避免挡住恢复入口，恢复时回到同一服务端选择。新增样式限定room-management类，修正复用紧凑席位横幅导致的说明截断，不改牌库网格/原图/牌组合法性。

实际验证：539 Native规则测试 +40集成=579通过；Web620/70文件、类型检查通过；46项暂停相关专项及34项弹窗相关专项通过（与全量重叠，不能相加）。真实 WASM9完整转换和36个席位视图与 Native全部一致，内层opaque state逐字节一致。实际本地workerd/WASM/D1 SQLite27项通过，包括暂停后D1 reopen、原收据、拒绝另一桌/观众、相同commandId另一桌独立接受及另一桌原始5表不变。Native实际Store重启、replay、失败事务无部分收据亦通过。测试使用明示本地布局/房间，不是线上D1操作或容量测量。

双独立浏览器进程，真实本地React UI与Native服务通过3组自然检查：两人从UI创建/加入/准备/开始；私有选择者可暂停，暂停时没有自动state读；返回大厅/刷新/我的牌桌恢复原席；伙伴恢复、手动同步恢复同一个选择和手牌实例。3张截图已实际查看桌面暂停与手机暂停两张；第三张恢复截图作为行为证据保存。公网自然暂停未测试，因为候选未发布。

实际 release WASM2,209,157字节，SHA2569e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9，身份 rust-v0.2.46-pause-resume-candidate / limited-v2.42-jc089-poison-blood-candidate。规则版和103普通卡、8会社不变。13个原历史fixture文件字节未改；旧engine45 WASMafc1b425…883f7仍在已发布Site39归档及独立保留副本，没有重新生成旧程序。源码只改变room会话层/版本和所需Host/UI接线，封印/搜索14文件WIP另存不入此候选。

失败轮未计入通过：Native新增测试的部分move编译错误、TS测试变量的过窄类型已修正；首次bindgen输出目录错误导致读到旧45，改用绝对pkg路径后重建/重比；Chromium单进程不能建两个context，改为两个正常独立进程；早期自然脚本在同伴更新前点Ready，被正确版本拒绝，最后脚本等待实时准备数量。实际成功证据绑定在[review packet](evidence/pause-resume-2026-10-07/review-packet.json)。Native/WASMfixture位于 `sites/test/fixtures/pause-resume-native-v046.json`；数据库与真实本地席位凭证仅留在 `/tmp/pause-resume-evidence/natural-browser.sqlite3`，不进Git/静态包。

复现：按本环境offline Rust1.90设置执行 `cargo test --offline --locked -p hegemony-server --jobs 1`，`PAUSE_PARITY_DIR`可输出Native trace；release wasm32构建+wasm-bindgen web目标，再运行证据Node比对脚本；`HEGEMONY_BACKEND_TEST_ROOT`限定本地backend构建后 `node --test --test-concurrency=1 test/*.test.mjs`；Web `npm run typecheck` / `npm test` / `npm run build`。自然脚本用本地Native8090+Vite5191，不使用线上URL或bypass凭证。

范围边界：不添加不活动TTL/自动永久删除；删除入口、业主升级备份重置和可信同版本fast204仍属后续独立批。当前实验站点继续按固定tuple拒绝旧核，保存的数据不等于跨升级自动兼容；父此前授权升级可重置，此次未执行。版本协调改为暂停engine46/pool42、封印engine47/pool43、搜索engine48/pool44；BQ104/XQ48待明确规则裁定，未入池。交父独立审查，不能把尚未审/发布的候选称为线上功能。
