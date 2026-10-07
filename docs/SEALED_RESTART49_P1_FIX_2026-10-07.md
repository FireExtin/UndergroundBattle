# 封印终局重开 P1 修复：候选 engine49 / pool44

父审查指出封印47的 `Game::start` 重建棋盘时漏清 `sealed_cards`，该问题也由JZ50候选48继承。本次从 **9aa74dc5163128d424d94f0e423253f3171e686d** 修复，历史清理暂停，没有发布或修改线上数据。线上仍为Site40（已审engine46/pool42）。输出为一行生命周期修复、候选身份、回归测试、新生成的49 Native链及[审查证据](evidence/sealed-restart49-2026-10-07/review-packet.json)。完整diff、增量bundle和精确ABI另置于本云端交付归档。

旧核复现使用真实付费XQ40封印及普通让过命令，经准备阶段空库抓牌实际淘汰对方并调用 `finish`；没有手工赋值finished。该复现明确布置了棋盘和对方空库，日志在重开后报 `restart retained a payload bound to old host i243`。终局保留载体和封印是复盘行为；修复仅在 `start` 增加 `self.sealed_cards.clear()`，新局不再保存绑定旧载体的场外牌。没有调整离场返还、封印规则、状态校验或终局清理。

生命周期审计覆盖Game字段：棋盘/世界库/装备/区域替换、修正项/本回合能力使用、堆栈/待选/效果队列、winner/turn，以及各席位手牌/牌库/资产/墓地/得分/会社/淘汰/本回合标记已有重置；begin Prepare重置让过和优先权，privilege已有重置。旧控制效果及baseline由apply后的既有settle_deaths→settle_controls按失效实例清除，本回归在终局保留真实旧控制绑定后验证新局两者为空。active_team由行动窗口设置，Prepare不依赖上局值；随机状态、序列、版本、日志及座位/准备状态/固定牌组按原房间生命周期保留。未发现需扩大修改的字段。

三项新增Native测试分别验证：实际终局→重开→Game/Room存读→下一次调度选择；已认证临时Store/SQLite的重开、进程重读、下一命令和原重开回执；全新大厅合法自定义牌组的完整对局。Store测试明确注入前一终局布局到可丢弃测试DB，不冒充从HTTP建桌开始的整场比赛。

第三项使用生产Game工厂、四席join_with_deck、构筑校验和固定seed9，再以真实Room准备/开始/资产/部署/付费封印/选择/响应让过/普通让过命令推进，不修改手牌、牌库、棋盘或status。**6,311条命令**在第43回合实际抓空库淘汰对方两席，winner为0，终局仍有一张封印；真实restart后Room持久化读取及下一次mulligan选择通过。每步校验journal回放和存读；紧凑轨迹保存完整转换canonical JSON、原样opaque state和四席视图的SHA256，并保留终局/重开/下一选择的原始状态。

最终Native **586规则+40集成=626通过**。真实release WASM重放上述6,311命令，另重放JZ50暂停14步、普通暂停9步、封印暂停13步、显式终局Store布局2步、新房间终局重开2步；后两条新房间重开命令也包含在6,311内，故这40次是额外执行次数，不作为独立新命令重复计数。完整转换、opaque state字节及 **25,404** 次席位视图比对全部一致。候选ABI **2,248,074字节**，SHA256 **9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885**。

Worker实际本地workerd/WASM/D1 **33通过**。新增测试从上面新房间真正生成的终局检查点开始，实际执行restart、关闭/重开D1、读回及下一选择，重复旧restart返回原回执而不再次重开，冲突ID拒绝，第二桌五表保持不变。它不是全场6311命令的Worker建桌测试，也不是浏览器端到端对局。现有Web封印/检索/选择面板 **22项/3文件**及类型检查通过；本次没有UI产品修改或新增截图，不把此前只读Native投影浏览器截图称为机制端到端证据。

引擎升级为 **rust-v0.2.49-sealed-restart-candidate**，卡池仍为 **limited-v2.44-jz50-death-search-candidate**，107普通卡/8会社，全部既有卡记录保持一致。19个旧跟踪Native fixture保持原字节，4个新49 fixture来自当前Native实际输出，没有重标旧状态。45/46/47/48冻结WASM均核验原SHA；Worker沿既定current-only策略拒绝真实旧46/47/48，不迁移或删除旧局。society-fixtures身份接线更新，但未生成新的fixture特性ABI或声称执行该套件。BQ104/XQ48未准入，未接线的两份历史bounded_search草稿保持原SHA，仍待父已提请的用户裁定。

保留未通过轮：修复前旧核P1复现、首轮新房间测试在34回合命令上限5000耗尽（未计通过，现上限10000）、默认rustfmt尝试创建只读/home/agent/.rustup失败（随后使用本任务已有工具链路径）。最终源后全量Native、release/WASM比对和Worker均已重跑通过。

复现使用本环境offline Rust1.90：cargo test --offline --locked -p hegemony-server --jobs 1；设置PAUSE_PARITY_DIR、SEALING_ROOM_TRACE_DIR、JZ50_ROOM_TRACE_DIR、SEALING_RESTART_TRACE_DIR、SEALING_FRESH_RESTART_TRACE_DIR导出五条链及紧凑完整对局。release wasm32及wasm-bindgen后执行证据check-native-wasm49.mjs；HEGEMONY_BACKEND_TEST_ROOT隔离构建后node --test --test-concurrency=1 test/*.test.mjs。证据中的绝对路径是本云端位置，跨环境复现需按实际解包目录调整。

本轮还只读重查用户指定云端任务的Sites能力：get_site成功、owner、active、线上version40；已恢复的官方OpenAI Sites0.1.75全部167文件符合integrity manifest，site-workflow.mjs可读，17,583字节/SHA256 34c3408f93a12684c993adf0256d7fe138f2baa43ead46a63921c87e36935acd。没有改变Plugin Manager安装状态，原下载403未重试、不宣称该地址授权已修复。Library配额仍未核实。仅将本轮重复展开Native轨迹压缩归档并逐文件核验后删副本，未删除Library或用户数据。

候选仍交父原审查者独立复审，不推main、不发布、不重发Site40。历史Go调试器在web路由和/api/debugger代理仍有活动引用，暂停清理期间没有据旧README直接删除。
