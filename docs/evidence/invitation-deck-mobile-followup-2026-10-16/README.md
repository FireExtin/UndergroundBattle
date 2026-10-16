# 独审 P2 补充与逐卡映射证据

本补充未发布，等待父会话串行复核关闭。首批 `3305bd9` / `9967fab` 的证据原样保留；本目录是追加范围。

两项 P2 的代码位置、已有修复身份和四项通过的关键回归见 [review-findings.json](review-findings.json)。最终完整前端为 **124 文件、1172/1172**，定向为 **80/80**，浏览器为 **36/36**，生产构建通过。

| 项目 | 身份 |
| --- | --- |
| 补充代码 | `79d09ff879425d043344b53613efa19bf7d0a7f6` |
| 补充比较基线 | `9967fabf9155486bef85190d20055fcb69be12d5` |
| Web tree | `1bd246d7f5602dd128015ecb2e26bc059c3cdb95` |
| 分支 | `codex/deck-collection-page-review-20261016`，无 PR |

第一项 P2 的原桌恢复逻辑已在首批最终代码中；本次进一步明确退出邀请文案及回归：

- `GameApp.tsx` 的“旧牌桌行动恢复”区域显式提供“退出邀请，恢复原桌待确认行动”。只显示与旧 pending 匹配的原席。
- `useGame.resume` 检查 pending 的房间和席位，恢复原凭据并清除 URL 的 invite 参数，不清除 pending；没有自动跳转。
- `PlayerSessions.test.tsx` 的 `keeps an old unconfirmed action...` 验证点击前旧桌不被轮询、不新增命令，加入 B 被阻止且原 session/pending 未变；点击退出后回到 A、invite 为空、原 session/pending 保留，且未发出新的入席请求。

第二项 P2 本次修复：`DeckLibrary` 接收完整快照；`sameDeckSnapshot` 比较全部公开内容，包括逐卡数量及版本，仅忽略保存时间和牌的排列顺序。保存列表不再以同 ID 代替已选择内容：

- 选 51 张后同 ID 保存为 52 张，标记为“所选为旧快照，需重新选择入席”。保存不改变所选内容。
- 明确重新选择 52 张后才显示“已选择入席”。桌内目录显示“实际上桌”或“与上桌快照不同”。
- 回归还覆盖同 ID、同总数、不同逐卡数量的修订，以及仅时间/顺序改变时不误报。

[isolated-browser.json](isolated-browser.json) 为编译后的真实 Worker + 持久化隔离 D1 双浏览器验收，36/36。新增的 `deckMappings` 为 11 个阶段完整的 `expectedCardCounts` / `actualCardCounts`（每阶段 12 种牌）和秘社选择：

| 阶段 | 全映射目标 |
| --- | --- |
| 邀请入席 | 与房主默认原牌组 A 完全相同 |
| 选择 B 快照 | A 的 JC125 增加 1，其余 11 种不变 |
| 保存 B 新版 | 再增加 1，但所选快照仍是前一版 |
| 恢复原席 | 服务器仍保留 A |
| 显式应用 B 快照 | 每种牌与 51 张快照完全一致 |
| 桌内后来显式选择 B 新版 | 每种牌与 52 张新版完全一致 |
| 准备、准备后刷新 | 都保持同一完整 B 新版映射 |
| 开局、双方完成再调度 | 都保持同一完整 B 新版映射 |

所有牌组映射来自 UI 入席/行动返回及轮询中的真实 `yourDeck.cards`，所选/保存版本另从该隔离浏览器的快照读取；期望 B 独立从原牌组按 UI 编辑的 JC125 数量推导。没有用名称或总张数代替映射。全部检查只访问 localhost，未访问真实用户桌。

最终测试计数和命令见 [validation.json](validation.json)、[frontend-vitest.json](frontend-vitest.json)；定向与完整回归不累加。[source-binding.json](source-binding.json) 固定本补充的 Web 输入；[artifact-manifest.json](artifact-manifest.json) 记录构建输出且三套 client 文件逐字节一致。Worker 和三份 WASM 沿用首批相同身份，没有 Rust/Worker 源码变化，也未重跑 Native 全套或新增线上试玩。

原审查分支使用 `[skip ci]` 正常推送；推前核对工作流及无开放 PR。不创建 PR、不请求 GitHub 评审、不 dispatch、不修改权限、不写 main。
