# 0.2.4：俯视牌桌、自组牌库与有限规则修复

规则仍为《霸权》`hegemony-pdf-v1`，卡池仍为 `limited-v2.2`（26玩家卡＋4世界地区定义），引擎为 `rust-v0.2.4`。新增有限构筑、冻结牌组与公共视图字段使用schema2并固定到此版本；后续响应意图节奏将另用schema3 envelope，不能把旧房间静默升级。

## 本批结果

- 桌面为用户确认的俯视2D：本人队伍在下、对手在上、三/五地区居中；角色留在各自地区，资产/牌库/墓地/计分/秘社区按玩家独立排列。仅本人手牌可读；暗藏者公开图标按规则显示0/0/1，横置不参与，悬停与放大阅读保留身份过滤。
- 牌库编辑器只使用当前已实现目录，支持预组复制、派系/类型/名称检索、张数编辑、命名、本浏览器保存与进入/大厅明确选用。保存草稿不会自动改入席牌组；选用后服务端验证并冻结副本，取消准备。当前没有账号跨设备同步、秘社或完整685卡池。
- JC059防御修正包含同队友方角色；完整命令ID意图校验拒绝不同座位/版本/动作复用ID，同时保留原操作跨推进/重启回执恢复。地区当前对抗图标与停战比较标记直接来自权威公开投影。
- Worker保持2.1、2.2、2.3准确旧bundle，room catalog跟随房间版本；旧房不开放本批自组接口。

## 实际检查

原生57项不同测试通过（56项完整轮，加最终17项定向轮，其中1项新增）。JC059与回执冲突都保留先失败后修复日志，见 [核心验收](DECK_BUILD_AND_JC059_ACCEPTANCE_2026-10-02.md)。WASM为1111835字节，SHA256 `fcf7f714a9b09f566202c0a8b00571c0da55dfa255793f2cffd908003f6d660f`；7场景432状态转换/1259本人视图完全匹配native，最大u64 seed精确保持。

目标提示修复后，最终官方封包前端125项测试、TypeScript和Vite构建通过。密集合成牌桌8组合（1V1/2V2与1280×720、1366×768、1366×900、1600×1000）均无页面溢出或角色跨地区/资产/手牌边界。1280×720四人桌155张合成牌逐张悬停与选择均成功；这些证明布局与命中，不证明规则或自然对局。

实际workerd＋SQLite D1模拟的6项测试通过，覆盖四版本路由、并发创建/加入/命令、原始回执、原子回滚、冻结自组、非法构筑拒绝、隐私及Worker关闭重开。首次集成轮读取了旧dist/server编译包，health仍为2.3导致版本断言失败；重建本批包后完整轮通过，失败日志保留。模拟结果不冒充远程D1验收。

B06/B07研究原稿已经合入。准确研究分支`097d9e6948636ce4b599d7ad7a11080bb7a4b651`隔离快照的生成器检查、来源校验、62项数据测试通过。最新游戏主线对其旧29定义审计产生代码/目录差异及2项重建守卫失败，原样保留；没有重写历史研究快照或由此开放未实现卡。

真实0.2.4云端Chromium短流程通过：房间`1aeec9c7d331f3a74e31641c`最终v9，两桌面独立profile，以10次实际UI POST（2次创建/加入、8次游戏命令）完成含事务预组复制、命名/保存/刷新、不同自组入席、大厅保存不改冻结副本、明确选用取消ready、开局、再调度中原席位/choiceId/有序候选刷新恢复，并完成双方再调度。页面错误、console错误、requestfailed及HTTP>=400均为0。首次尝试在创建任何房间前因测试label定位超时而停止；失败记录保留，不计为产品通过。证据`/workspace/scratch/hegemony-custom-decks-v024-2026-10-02/summary.json`。

原随机native3局自然JC058现身进入目标选择，发现匿名同名候选缺少公开归属/地区提示。普通地区目标已补目标序号、拥有者及不同操控者、地区编号；不改变CardContent身份过滤、选项ID或付款行为。31项相关组件检查通过，实际多人候选交互由明确合成组件测试验证，不能冒充自然多人场面。

私有云端证据位于`/workspace/.private-validation/hegemony-v024-evidence`、`hegemony-v024-web-final-{tests,build}.log`、`hegemony-v024-sites-{build,final-tests}.log`与`/workspace/hegemony-table-ui-preview/evidence`。远程发布及旧生产房兼容正在执行；5秒响应意图尚未在本版本实现。

## 实际发布及远程验收

既有公开Site已部署Site version6（version5仅保存未部署），保存版本`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_1b8042f232788191b9609774fb73a71b`，deployment`appgdep_6abfe034b1cc819181aa530d0fda8ebb`于16:47:58.659831Z succeeded。精确已推source为`c0edce2da76ef4cb9341b53945f2836a8f108f0f`，对应主线`6deec5d94c63e575b993c30897221329803ac3ea`；WASM hash不变。部署包仅21项文件，不含原图/PDF、685研究资料、数据库或凭证；既有public audience revision2保持。

生产入口：https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site 。两新桌面profile的远程D1短UI矩阵通过，房间`4a7fe14547f50bf1fb995f2a`最终v9，10次实际UI POST均200，四类浏览器错误及QA转发错误均0，没有bypass、native席位复用、API准备或状态注入。首次浏览器启动因Unix socket路径过长在0页面/0POST时失败，改短workspace TMPDIR后完整轮通过；不是生产失败。摘要`/workspace/scratch/hegemony-custom-decks-sites-v6-2026-10-02/summary.json`。

原生产2.1四席房`8438277a645d91a656f57ce9`v3296、2.2两席房`00a8e0dc7fc2e387708472db`v662的6本人视图哈希、2原命令回执哈希/原版本、旧room catalog在发布前后完全一致；0新房、0新游戏命令。证据`/workspace/.private-validation/hegemony-v024-production-compat/{before,after}.json`。历史503和此前严格传输失败记录没有被本批兼容通过抹掉。

原随机native3双人局在新桌面UI下自然出现JC058并走真实付费现身、目标选择刷新恢复、对手付费现身反制及嵌套触发。v209原暗藏目标已离场，效果取消且费用不退，双方正面角色仍在场；没有协调对手。最终同房`1cf4c7361709ae5598ff2336`v239停在预算边界，p0共120（57手动/63产品辅助）、p1共118手动/0辅助，无游戏reject；p1先前1条ERR_INSUFFICIENT_RESOURCES保留、原因未定。成功消灭、多匿名候选自然场面与来源移除仍未覆盖；本批不声称独立策略完成整局。
