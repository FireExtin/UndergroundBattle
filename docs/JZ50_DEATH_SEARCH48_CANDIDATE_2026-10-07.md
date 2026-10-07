# JZ50 墓穴食尸鬼：有限私有检索候选 engine48 / pool44

父要求封印47固定并正常推送后，继续JZ50已裁定部分。本批父提交为未发布、待独审的封印47 **919e574a7eb0ca2531119486e54bfc4970abf9af**，没有把47写成已验收。线上仍是Site40的已审engine46/pool42；本候选没有发布或线上数据清理。输出23个产品/测试/Native fixture文件及[evidence packet](evidence/jz50-search48-2026-10-07/review-packet.json)。

输入为再次实际查看的完整JZ50原图，SHA256403e44bb5f7f0ff8715acb267a8369c3715b2dc752d7fe612de085da29b523e7，以及已实际查看的基础手册、霸权P16来源/控制者、P21队伍私有区域和FAQ相关完整页。原卡cost2、黑色忠诚1、角色/不死生物/食尸鬼、死亡领域1、永久战斗1、防御1，非独有、无关键词。原文的死亡领域符号转写为“死亡领域”。已登记JZ50原图保持原字节，未重复新增扫描。

项目[owner FAQ](rules/JZ50_SEARCH_OWNER_FAQ_2026-10-07.md)记录用户2026-10-07 12:23UTC明确选择B：接受现身触发后可选0或1，即使有候选也可0，0仍洗牌；拒绝触发不检索、不洗牌。这是JZ50项目补充裁定，不能称为印刷规则或官方FAQ原文，也不套给BQ104/XQ48。已接受搜索始终提供行动者私有0..1选择，空候选亦相同；其他三席（包括队友）只见同一种等待。allowDecline=false，选0是完成并洗牌，不是第二次拒绝触发。

实现只有无参数Op::JZ50SearchDeathToGraveyard和无参数FrameChoice::JZ50DeathSearch，独立jz50_search.rs只编译JZ50。触发在真正Reveal时冻结source.controller为actor；公开部署和盖伏均不触发，横置现身仍可产生触发。只从该actor真实牌库选死亡领域角色，选中后复用既有reset_zone_card，产生新ID，按被移动牌owner进入墓地，不产生Death。沿原有shuffle_player精确一次洗余库；0/1张牌库零交换，不声称RNG必变化。来源离场、换控制、原地区替换不取消这个无地区目的地程序。多来源先FIFO提供可选声明，接受后用已有LIFO堆栈结算，保持两位actor及同名同色实例独立。

准入守卫完整Definition/ability，递归拒绝modes、ForEach/FromActor、IfTargetExhausted两支及其他卡移植。有限运行时帧核固定来源/key/冻结行动者、Accepted guard、单步骤/游标/context、无目标及付款；源不必仍在场。Game和Room存读核私有选择席位、0..1、不允许第二次拒绝、实际当前牌库候选ID集合及重复/失效选项；未准入牌库定义返回错误而不panic。没有通用目的地绑定、身份注册或新队列。BQ104/XQ48原型仍在未接线的旧草稿里，未编译、不入池。

ChoicePanel只为有限jz50_death_search允许空确认，并显示“不取牌并洗牌”；选1仍确认原实例。原有Search、JC032零命中、暂停选择弹窗及读牌入口保留。实际Native投影进入Table测试：actor看见候选，其他三席无私有选择/候选控件，空候选可确认，真实完成投影关闭弹窗并显示owner墓地。没有借此修改牌组构筑或移动布局。

最终身份 **rust-v0.2.48-jz50-death-search-candidate / limited-v2.44-jz50-death-search-candidate**，107普通卡（含10地区）、8会社，仅新增JZ50。真实release WASM2,247,997字节，SHA256 **90ba2fdc49c8dc256b69a2b4cc2acdf3acfaf87060140ec2829701db10805114**。45/46/47冻结WASM、原16个已跟踪fixture和全部118张原图登记/字节保留；新增3个实际48 Native Room链fixture，没有重标旧46/47状态。当前Worker分别将真实旧46、47按unsupported_room_version拒绝，旧数据仍保留，不承诺跨升级恢复。

最终Native583规则+40集成=623通过；20个JZ50专项与全量重叠。覆盖付费部署/现身、横置现身、拒绝、空/无候选/0/1、精确确定洗牌、四席隐私/队友隔离、owner≠controller、真实付费回手响应、来源离场/换控制/地区替换、失效候选、同名同色实例、FIFO声明/LIFO效果、置墓无Death、定义/动态帧守卫、四席各21种非法存档、CAS及实际Native Store SQLite重开/原收据/重试不重复洗牌。临时布局/测试DB明示，不操作用户真实数据库。

精确最终Native trace重新导出并与WASM比对：214命令（6拒绝）、469检查点，JZ50暂停链14步+普通暂停9步+封印暂停13步=36步链，2,876个席位视图及完整转换/opaque state全部一致。Worker实际本地workerd/WASM/D1 SQLite31通过，包含三条链、四席权限、D1重开、原收据、重复ID跨桌独立、第二桌5表字节不变。隔离会社fixture脚本的版本/数量接线更新，但未生成新的society-fixtures WASM或声称额外执行其套件。

Web631/72文件及类型检查通过，TypeScript/Vite生产构建通过；122个dist文件核验，入口/JS/CSS及摘要保留，静态资产逐字节等于源后才删除64,649,731字节可重建dist。桌面实际React4项检查通过，3张截图已实际查看：有候选可0且不二次拒绝、刷新原席/同一私有选择、空命中同样确认洗牌、真实完成墓地投影关闭弹窗并刷新保持。浏览器只读GET由独立实际Native布局投影拦截，改变投影不是自然对局或实时Worker流程；没有提交写命令，不冒充选择→置墓的浏览器端到端验证，也不声称手机适配/刷新通过。首次浏览器因磁盘紧张退出未计pass；核验并清除两份未运行、可重建旧Native测试缓存后，最终只读轮通过。

未通过轮保留日志：草稿双来源错误假设按FIFO结算，已改为FIFO声明+LIFO；非法移植测试错误索引无能力的LC01，已改为添加非法ability；TS参数化数组推断、旧总卡/非地区计数预期已纠正。它们不计最终通过。最后源变化后重新跑全量Native、精确专项输出、release ABI/比对，相关Web/生产/Worker检查均完成。

复现本环境offline Rust1.90 cargo test --offline --locked -p hegemony-server --jobs 1；JZ50专项设置GREEN_EVIDENCE_DIR、SEARCH_FRONTEND_DIR、JZ50_ROOM_TRACE_DIR导出；全量设置PAUSE_PARITY_DIR、SEALING_ROOM_TRACE_DIR导出另两链。release wasm32及wasm-bindgen绝对pkg输出后运行证据check-native-wasm48.mjs；临时HEGEMONY_BACKEND_TEST_ROOT构建后node --test --test-concurrency=1 test/*.test.mjs；Web typecheck/test/build。完整diff、增量bundle、精确ABI、Native输入及生产编译入口置于云端交付归档，原始日志gz可恢复原字节，无真实席位token或数据库入Git。

仍待父裁定：BQ104接受后且有雇员时必须1还是允许0仍洗；XQ48原地区被替换后是否跳过检索/入场但洗，还是私检0..3留库再洗。父已向用户询问，本任务未自行选择。封印47与本48都交父独立审查，不推main、不发布、不重发Site40。
