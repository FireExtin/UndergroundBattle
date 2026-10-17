# MSJC04 / MSJC05 已审批次发布为 v65 / ABI83

官方Sites于 **2026-10-17 14:21:49 UTC** 返回 `succeeded`；回读确认公开站最新版本为 **v65**，来源、归档、部署ID及公开权限一致：[隐秘世界 · 霸权](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)。本批新增鸣钟教派与国家机构、入席初始秘社选择和显式载入入席快照，沿用既有本地命名/保存/改名；牌组库仍在本浏览器，不提供账号云同步。

父会话已完成整批串行独审并明确授权发布：代码 `9767e293c9a3e43ea25d6620afc97ab9f32bc171`，证据 `0f4d2d95019706aa15a2e7102503cba8993916e0`，未发现新增P0/P1/P2。独立检查覆盖203个Git文件、129个Native输入及21个闭包文件、31份gzip日志与原日志身份、完整Native1201/前端136文件1292/Worker295结果和5张代表截图。此前“父独审未完成、ABI83未发布”的状态由此更新；规则来源档案的历史疑问仍原样保留。

| 身份 | 精确值 |
| --- | --- |
| 项目 | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| 保存版本 | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_09c1b8bf209881919e038913e2c3ddff` |
| 部署 | `appgdep_6ad38470da2881919f6587ae007634b9` |
| 官方源码 | `6efc6273a4788e3ed4774d1a07b0921ff6d9d9a0` |
| 引擎 | `rust-v0.2.83-msjc04-msjc05-societies-candidate` |
| 卡池 | `limited-v2.75-msjc04-msjc05-societies-candidate` |
| ABI83 WASM | 3,174,213 bytes / `f66f6ab06d36ca86a03ff8370471332627fea3df217ea43b168c759575a2c89f` |
| 生产Worker | `4a23e098b1abf6561edd0b1115eea54d58db976b9926058479817b22e6a6f481` |
| 上传gzip | 76,288,844 bytes / `a71b42653c7fed7a8fbb04cd9913122bc0d9fb27d5f0056cdc1fa62b17241a08` |
| 官方规范化PAX归档 | 119,152,640 bytes / 178文件 / `6fa42e6ff139e78936129d556061f88309a090016e08af197524d22fecf46225` |

使用已读的官方Sites 0.1.75技能和原有 `site-workflow.mjs`：先原生回读v64、公有权限revision2，再取得普通源码写凭据，经隐藏stdin交给官方open；返回原工作副本与v64来源 `74b1e8f2e6a0c4feb403520554a9275f52be82fd`。核对干净且没有远端进展后，按有限映射写入已审代码。1230个来源文件中1228来自9767的精确Git内容（两个package保留原fixture恢复脚本），另外2个是独立核验的pkg JS/WASM。129个Native及21个闭包来源全部匹配。

发布前实际遍历生产目录175个文件：164客户端、生产入口 `server/index.js` 和7个WASM共172个运行文件，另3个仅构建元数据文件。逐文件size/SHA、缺失/额外/符号链接均已检查；七个内核实际初始化并读取目录身份，77–82与冻结发布原字节一致，83为上述f66。旧桌继续按原身份路由，不迁移；生产入口与固定时钟测试Worker不同，测试Worker未进入归档。见 [production-preflight.json](evidence/msjc0405-publication-2026-10-17/production-preflight.json)。

官方工作副本保留旧夹具恢复钩子和存储文件、hosting/Wrangler/Drizzle/D1配置及迁移；原v64生产目录整体移到 `/workspace/msjc0405-publication-20261017/prepublish-v64-dist` 保存。没有读取或改写线上D1数据。用 `commands:[]` 复用已独审通过且字节不变的测试与生产构建，由官方流程正常提交、推送并核对其连接器远端精确SHA，再由官方打包器打包。未手动推送provider、触发Actions、修改权限或构建配置。

归档再次核对172运行文件全部相符。178个归档文件其余6个严格限定为hosting、3个原迁移/元数据、source map及官方说明。上传gzip保持不变，规范化为按路径排序的178个普通文件PAX tar后，SHA/大小/文件数与 `save_site_version` 返回值完全一致；之后只部署该精确已保存版本。部署直接返回终态成功，未额外轮询；原生版本/项目回读再次核对来源、归档与部署关联。公开权限revision2及环境revision0保持不变，凭据仅在内存和隐藏stdin使用，已丢弃。

身份与全过程证据见 [official-receipts.json](evidence/msjc0405-publication-2026-10-17/official-receipts.json)、[provider-input-audit.json](evidence/msjc0405-publication-2026-10-17/provider-input-audit.json)、[publication-archive-audit.json](evidence/msjc0405-publication-2026-10-17/publication-archive-audit.json)。首次额外诊断尝试 `git ls-remote origin` 因官方工作副本不设置持久origin而退出128，在读取归档前即停止；没有权限拒绝或备用凭据操作。按官方脚本实际使用连接器URL正常push再ls-remote的成功回执核对后完成审计，此边界也保留在证据中。

本次没有导航或抓取部署URL，**没有v65线上试玩结论**。上一轮13:29 UTC的普通Chromium证书阻塞仅属于当时v64，不能推断新v65连接结果；本地ABI83组件/HTTP/D1/浏览器验证也不计作线上试玩。完整本地结果、失败修正和来源裁定边界仍见[原审查交接](MSJC04_MSJC05_REVIEW_HANDOFF_2026-10-17.md)。

游戏远端main此前已按单独授权正常快进固定ABI82代码 `398e220721f9e14c4f523530183c6288b7749096`；此次v65发布不把ABI83推入main。main晋升见[独立记录](REVIEWED_V64_MAIN_PROMOTION_2026-10-17.md)，ABI83未来main推进需独立处理。历史ABI74重建身份、JC016＋BQ022返回/优先权及全原作验收仍不在本批解决声明内。
