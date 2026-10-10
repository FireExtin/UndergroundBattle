# 封印三卡：固定待审候选 engine47 / pool43

本批以已发布 Site40 的文档提交22fe9cfc8588506113233b54dfd0cc7fe7ca246e为父提交，产品底座为已审945e8b2c8b96e2d75fcb709db72c7a1e9e4ad635。输入为实际查看的XQ40催眠术表演者、XQ41启迪之梦、XQ45破颅而出完整原图及霸权P15/P16、基础P4、FAQ相关完整页。XQ45没有领域要求；不能把旧资源的星辰字段当作规则。输出是本提交的34个产品/测试/原图文件和[evidence packet](evidence/sealing47-2026-10-07/review-packet.json)。本候选未获独立审查、未发布；线上仍是Site40 engine46/pool42。

实现新增有限公开封印区及真实载体ID关系，复用已有Card实例、区域重置、私有选择、事件声明和堆栈。XQ40付款2并横置，绑定一名正面角色，先抓1，再强制封印行动者手牌中的1张；付款后来源离场不取消，原目标离场或被替换取消整段抓牌且不退款。封印牌在场外公开且空白，没有能力、图标、防御或普通目标资格。载体离场或翻暗，各牌重置为新实例回各自owner手牌；场内移动和控制权变化保持真实实例关系。赢区直接移出和玩家淘汰路径亦接入。没有新增全局身份、队列或通用绑定框架。

XQ41在封印成功前冻结手牌持有者及来源，之后空白或载体离场不取消已产生的可选抓1触发；拒绝不抓。墓地及返回手牌按owner，效果行动者按冻结controller。XQ45可选合法但未封印角色，结算时仅按真实封印关系消灭该实例，不把同下标的新角色当作原目标。三个完整Definition/ability及有限Op均守卫；嵌套ForEach、FromActor、IfTargetExhausted两支、modes和EventSealed移植拒绝。

持久状态核验封印区的真实载体、有效拥有者、重置空白状态和跨区重复实例；私有HandSeal帧还核固定来源/key、冻结行动者、两步程序/游标、目标实例、实际手牌选项和恰好1的边界。来源不必仍在场。Game及Room读取复用同一有限校验。UI在已有HUD提供只读封印菜单，展示owner、当前载体controller、地区与实例；当前阅读显示空白，原图标签明确印刷能力不生效；载体离场后关闭旧ID阅读弹窗。原有牌库构筑、原图、暂停/恢复和选择弹窗入口保留。

最终源码及原始cards.json metadata均是rust-v0.2.47-sealed-cards-candidate / limited-v2.43-sealed-cards-candidate，106普通卡（含10地区）、8会社。精确最终release WASM为2,229,913字节，SHA256 **42ae5289b3998520bd8f9d1c35a32ffd2a1693960dc54579a9f47e184a01c4a9**。未用旧46名义重新生成历史程序；46冻结WASM仍是9e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9。14个历史fixture、既有115条原图登记和原图字节保留，新增3张原图逐字节等于输入。

最终验证：Native563规则+40集成=603通过；Web625/71文件及类型检查通过；真实本地workerd/WASM/D1 SQLite29通过。24个封印专项包括实际牺牲/摧毁/致死、救援回手、翻暗、赢区牌库底、拥有者与持有者不同、同名同色独立实例、控制/移动、同时致死、淘汰、失效目标、私有选择、FIFO触发、LIFO结算、定义移植、存读及原收据重试。专项与全量重叠，不相加。13步封印选择/触发/付费堆栈暂停恢复和9步普通暂停由实际Native输出；WASM与115条命令（5条拒绝）、242个检查点、22步链及1,516个席位视图全部一致，完整转换和opaque state逐字节一致。Worker检验四席权限、D1重开、原收据、另一桌5表字节不变，并将真实旧46状态按unsupported_room_version拒绝而不重标。

桌面浏览器4项通过：真实React读取实际Native47投影，公开空白阅读、原图明确失效、最新owner-return投影关闭旧菜单/阅读、刷新保留原席和最新视图；4张截图已实际查看。此轮GET被本地投影拦截，写请求全部禁止，不是假称从自然对局或本地Worker生成该状态，也没有线上写入。父要求桌面优先；没有声称最新手机刷新通过。Web生产TypeScript/Vite构建已通过，编译入口/JS/CSS和122文件摘要保留，字节核对后删除可重建的64,649,620字节dist副本。最终metadata纠正后已重建精确ABI并重跑上述Native/WASM/Worker/Web；较早364项隔离会社检查仅作早期记录，不冒充最终源码验证。

复现：本环境offline Rust1.90执行cargo test --offline --locked -p hegemony-server --jobs 1；release wasm32加wasm-bindgen绝对pkg路径；Native trace通过GREEN_EVIDENCE_DIR、SEALING_UI_FIXTURE_DIR、SEALING_ROOM_TRACE_DIR及PAUSE_PARITY_DIR生成（具体名称以测试源码为准），运行证据check-native-wasm47.mjs；HEGEMONY_BACKEND_TEST_ROOT指定临时backend后node --test --test-concurrency=1 test/*.test.mjs；Web npm run typecheck、npm test、npm run build。证据保留原始日志压缩包、Native输入、精确ABI、完整diff/增量bundle及浏览器脚本；不含真实席位凭证或数据库。

搜索族两个草稿保留但未接线/未编译/未执行，不在本提交。下一批只先接已定B的JZ50到engine48/pool44；BQ104接受后选择最小值、XQ48原地区替换后检索与尾洗边界仍等明确答复，不能擅自套JZ50裁定。此批不推main、不新增发布、不清线上D1，也不把本地缓存清理当作Library配额释放。
