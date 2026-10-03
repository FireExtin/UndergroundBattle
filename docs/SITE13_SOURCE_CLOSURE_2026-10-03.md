# Site13 四项报告源码核查及 QA 取消回归

输入：用户独立报告的 A/B/C/D、已审实现 `37a09a2cc0088845657eafd7088f54a2bbc19012`、候选记录 `c2704d3e2c660d87e41b8987871e6f07b78e7e24`，以及随后“公网玩法由父 dot 亲自操作；执行 agent 只做实现回归和环境准备”的明确任务调整。未派子代理。

Sites building/hosting 技能已读。当前仓库及祖先目录没有 AGENTS.md；遵循 `docs/TASK_DOCUMENTATION_POLICY.md`。没有使用 Library、索取旧环境凭据或访问朋友房间。

Sites 实际读取确认原 project `appgprj_6abf7bf54a7481918a50e1ef1509ca68` 的版本 **13**，保存版本 `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_7a8842419c948191b2360f2ce75b2186`，部署 `appgdep_6ac05ae6758c81919382cf3d1073f441` 为 succeeded，源码 SHA **197fba9ca5dec95c09a68ddb4066c9f246a4a548**。既有公网 URL 为 <https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site>。

通过新发普通源码凭据只读 clone 并 detach 到上述精确 Site SHA；凭据仅留会话内存/隐藏 stdin，没有写文件或推送。Site `source-provenance.json` 指向已审 `37a09a2`，117 个可对应源码文件全部逐字匹配当前实施树。`git merge-base --is-ancestor` 确认 `34affdd8667ee0e9bcdfb9bad4d458165a696d4a`、`e1e2dea581fbb97f1c2a64f7133179d50f1dd4d8` 都已进入 `37a09a2`。远端两个指定分支也分别实际返回 `37a09a2`、`c2704d3`。

| 项 | 当前结论与本轮实际证据 |
| --- | --- |
| A 真正 Win 回底顺序的弱验收 | 已闭环于发布所用实施源码。`attachment.rs` 的默认单元测试从真实 `Window::Win` 和四席 pass 进入四个 owner 的排序选择，逐张比较完整尾序、牌库前缀、新身份和重置状态；在所有选择保存前冻结宿主/附属/aura。本轮隔离副本只反转最终 `commit_region_return` 回底循环，测试红：实际 `[LC22,JC059,BQ022]` 与预期 `[BQ022,JC059,LC22]` 不等。原源码在独立正确 target 下 1/1 绿。测试本身随 crate src 存在于精简 Site 源码，生产 WASM 不运行 `#[test]`。 |
| B JC016 撤回 + BQ022 回收 | **未闭环，仍待裁定。** 实际查看 JC016 与 BQ022 原图；`rules.rs` 前者 retreat=true，后者宿主离场 OwnerHand。赢区批次却先给非暗藏撤回对象设 hand，其余对象按 owner 设 bottom，并先移除附属，所以该组合的现行行为是宿主回手、附属回拥有者牌库底。没有针对 JC016 的准入禁止；“隔离”只指不算入完整机制验收，不能宣称线上已禁用组合。本轮不替用户固定优先级，也不新增该组合的猜测断言。 |
| C 核按 controller、前端按 owner 遮盖 | 已修且实际进入 Site 源码。`engine.rs::project_card` 与 `CardTile.tsx::visibleCard` 同按当前 controller；卡片和原图阅读共用该契约。本轮核心潜伏回归 1/1 绿；恢复 owner 误判的隔离前端副本使卡片/原图 2 项红，正式这两文件 11/11 绿。另含选择与牌桌的聚焦前端 40/40 绿。控制权转移初态是明确合成，不能称为自然整局覆盖。 |
| D 赢区 fixture 只在 slice | 已修于发布所用原实施仓库的默认 `native_fixtures`/`verify.sh` 路径。本轮不传 slice 参数实际导出 21 case，含 `won-region-batch-cross-owner-equipment-and-aura` 的 9 步和 p0–p3 私密排序选择。直接加载 Site SHA 保存的原 WASM 比较：591 转移 / 1889 席位投影 / 1 报价 / 6 拒绝请求全相同。**Site staging 不复制 examples 与原生 integration tests，因此不能说 verifier 文件本身部署成网站端点。** |

当前保存的 WASM SHA-256 为 `21169e9d579886a55474b21aa506a4a9adca779428d7d11f2e966ae077656198`（1,593,545 字节），六旧核目录及路由都保留。原 public audience revision 2、DB 绑定不变。未构建或发布新 Site 版本，未开 v028、扩卡、修改规则或删除旧房。

新确认工具问题：`full_game_persistent.py` 取消在 `finally` 使用未初始化 `result`，同时最终页面不可读也会掩盖取消并跳过清理。本轮默认 unittest 用例在原 `c2704d3` 工具上分别因 UnboundLocalError / RuntimeError 红；修复初始化、显式 CancelledError 失败记录、容忍最终观察失败及完整 context 清理后 **2/2 绿**。自动入口见 `tools/cloud-playtest/README.md`，是该目录默认 unittest discovery；未声称接入不存在的全仓 CI。

任务调整到达前已在四个独立正常 UI profile 创建另一张专用 QA 桌 `d7ade3fc2732dc003eb5b04f`。任务调整后立即向自己唯一 runner 发 SIGINT；最终失败摘要 cancelled=true，**16 次 UI POST 全 200、revision 15、第一轮 start/prepare、没有墓地动作或恢复证据**。自有四浏览器均已关闭，profile 与本地摘要留在 `/workspace/.private-validation/site13-v027-public-four-grave-20261003/`；该段属于执行者自测并已中止，**不构成独立体验验收**。原旧执行器 `/workspace/.private-validation/hegemony-v026-fast-evidence/` 在本容器不存在，不能交接原中断桌。父 dot 另行报告的旧桌恢复不冒充本容器实测。

父操作支持边界：现有 Site13 即为可用环境，父可通过自己的独立浏览器会话按正常 UI 创建/加入 QA 桌。上述本地 profile 不能直接转交给父独立云浏览器；不复制身份或座位凭据。父双人桌 `D9A53962F8E8` 未入席、未准备、未开始；需要第二席时先由父明确指定可直接操控的浏览器会话，再做纯环境准备。当前公网四席墓地切片、刷新恢复及旧房只读兼容仍待父操作，不以原 56 次请求或本轮 16 次请求代替。

输出：本说明、`docs/evidence/site13-source-closure-2026-10-03/` 的源码哈希/默认覆盖/红绿日志/脱敏中止记录，以及 QA 工具修复与默认发现的取消测试。真实手牌、profile、数据库和原生完整 fixture opaque states 不进入证据目录。有效 native 红绿使用不同 target；一次共享 target 导致的陈旧 executable 结果已排除并以独立 target 重跑。

仓库元数据实际为 public，GitHub 不支持给公共仓库创建秘密分支；按“可提交私有分支”限制只提交本容器本地分支 `codex/site13-grave-qa-20261003`，不将本轮改动推送公共 origin。
