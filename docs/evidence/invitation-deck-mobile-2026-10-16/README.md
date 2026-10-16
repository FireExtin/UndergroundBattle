# 邀请、牌组、昵称与手机卡图复审材料

独审追加修复和 36 项逐卡映射验收见 [补充材料](../invitation-deck-mobile-followup-2026-10-16/README.md)。本目录保留首批固定结果，不与追加测试累加。

本批未发布。请父会话按下列精确范围安排串行独立复审，通过后再沿现有授权走官方 Sites 发布流程。

| 项目 | 固定身份 |
| --- | --- |
| 待审代码 | `3305bd928b6642c39c58c1a0ad92394bfcf234c3` |
| 比较基线 | `902f0615445047b621e128628f26a868bc7b73da` |
| Web tree | `c194becc7fa31f73f3d09826cb28dcaf1365d0b0` |
| 审查分支 | `codex/deck-collection-page-review-20261016`，无 PR |
| 本地浏览器 | Chromium，PC 1366×900、手机 390×844，真实编译 Worker + 独立持久化 D1 |

代码范围为 `Lobby`、`GameApp`、`api.readActiveSession`、`useGame` 的恢复入口投影、`Table.RoomLobby`、牌组编辑器的可见命名/列表、昵称存储及响应式样式。Rust、WASM 和 Worker 源码不变。新增发布输入为 `web/src/game/playerNameStorage.ts`；其余 Web 输入变化列于 [source-binding.json](source-binding.json)。

验证记录在 [validation.json](validation.json)，各轮不可相加。最终完整前端报告为 [frontend-vitest.json](frontend-vitest.json)；生产构建为本地 `wrangler deploy --dry-run`，未连接线上数据库，未发布。

[baseline-reproduction.json](baseline-reproduction.json) 使用构建前核对过身份的 v56 旧编译前端：新邀请 B 被浏览器旧桌 A 自动恢复遮蔽，没有发出 join，两边都显示 1/2 和取消准备。此为隔离复现，不代表已证明用户真实房间的根因。

[isolated-browser.json](isolated-browser.json) 是待审代码的 24 项验收，所有创建和命令均限 localhost：

1. 朋友已有桌 A，访问含目标邀请及 `#home` 的链接，只能加入目标桌，目标码只读，原席记录保留。
2. 空白昵称生成可读名字并记住；朋友加入目标桌后房主能看到同一名字，双方准备同步到 2/2。
3. 组牌主流程可以直接看到名称和保存列表。选择 B 的 51 张快照后再保存为 52 张，已选快照仍为 51 张。
4. 再次打开同桌邀请必须显式加入，并恢复原席、称呼、准备和实际牌组 A 的 50 张，不重复 join。
5. 显式应用 B 后服务器实际为 51 张，并取消准备；后来在桌内明确选择保存的 52 张时，旧的 51 张提示消失，所选内容也更新为 52 张。
6. 重新准备后双方均 2/2，刷新保留席位；房主开始、双方完成初始再调度，双方投影完成同步；游戏中保持最终应用的牌组，不能再换牌。
7. 手机手牌图实际加载，图框为 48px；暗置卡、资产及对方隐藏手牌不泄漏印刷图，页面错误为零。

截图均来自上述隔离桌。图中的邀请码和随机称呼为本地 QA 数据，未使用用户真实桌或私人聊天截图：

- [恢复后、显式应用前的手机画面](phone-restored-before-apply.png)：实际 A 50 张、所选 B 51 张，原准备状态保留。
- [手机开局卡图](phone-playing-artwork.png)：最终实际 B 原始 52 张，手牌印刷图可见。
- [PC 开局画面](desktop-playing.png)：桌面牌桌与真实卡图。

[artifact-manifest.json](artifact-manifest.json) 记录待审源码的构建输出哈希；`web/dist`、`sites/public` 与最终 client 逐文件一致。Worker SHA-256 为 `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`；三份 WASM 均为 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`。这说明本批沿用已发布引擎身份，不等于重新运行了全套 Native 或其它历史机制批次的测试。

真实用户牌桌未读写。本批没有线上试玩；此前普通浏览器访问公开站仍有证书阻断，不能用本地浏览器结果替代线上验收。未上传用户聊天截图、真实邀请码、席位令牌、SQLite、缓存或构建产物。GitHub 操作限只读核对和带 `[skip ci]` 的原审查分支推送，没有 PR、评审、dispatch 或 main 推送。
