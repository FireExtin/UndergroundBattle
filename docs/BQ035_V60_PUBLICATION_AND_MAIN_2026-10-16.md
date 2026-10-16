# BQ035 完整批次发布 v60 并正常合入 main（2026-10-16）

父会话完成串行独立复审，报告固定提交 `bd36b70b32890f539fde019613f5ae6d60c2903d` 无剩余 P0/P1/P2，并转交用户明确确认：将该完整游戏批次发布到现有网站，并正常合入 UndergroundBattle 的 main。本次只执行这项确认；原交接中“待复审／未发布／main 未推进”描述的是当时状态。

## 结果和身份

- 公开网站：[隐秘世界 · 霸权](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)，官方保存版本 **60**，部署于 `2026-10-16T16:13:55.867581+00:00` 成功。
- 官方 project：`appgprj_6abf7bf54a7481918a50e1ef1509ca68`。
- version：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_a728ca07f2c88191a553f801afe28fd2`。
- deployment：`appgdep_6ad24d37555c819187d15d9b5cb1c21a`，原生回包状态 `succeeded`，没有重复部署。
- Sites 源码由官方 helper 从原 `a2a002fb7cafb9030405853a1c98bd91800b627c` 正常提交、推送到 **`6d1a1fed34b5abad208488f09055554269e3df10`**。保存版本及发布后版本读取均返回这一 source SHA。
- 游戏远端 `main` 从 `3d642f0c1e6e5ac021cd40560086df35638bfe4c` 正常快进到已审 **`bd36b70b32890f539fde019613f5ae6d60c2903d`**；没有强推或制造额外合并提交。`origin/main` 已读取并核对一致。本说明及新增回执仅在现有 `codex/stage-pass-progress-20261016` 审查分支保存，未额外推进 main。

完整原生回执见 [official-receipts.json](evidence/bq035-v60-publication-2026-10-16/official-receipts.json)，main 操作见 [main-promotion.json](evidence/bq035-v60-publication-2026-10-16/main-promotion.json)。推送前后 Actions 总数均为 8，已审固定提交的 Actions 运行数为 0；没有触发、修改或手动运行 workflow。

## 复用已审源码和已测产物

运行源码闭合点为 `53b2114a220ca001ca8c40ff3b2a6f2b6276ef45`；它到已审 `bd36b70` 只有 docs 变化。本次复核有界源码副本的 1145 个映射文件和已测范围 852 个文件，随后同步到官方检出，不新建 worktree、不重写功能、不重建已经验证的生产产物。该映射和 [此前完整验证](BQ035_P2_WM023_TARGET_RESOLUTION_2026-10-16.md) 是输入来源。

官方检出保留既有夹具压缩存储及两个 package manifest 中的 prebuild/pretest 还原 hook；除这些已解释的 package 字节差异外，所有映射源码与已审清单一致。还原脚本指向的两份 JC046 夹具也与本批次字节相同。未映射的既有支持文件保持原样，官方源码提交共变更 138 个路径，未新增凭据、数据库或缓存。详见 [source-binding.json](evidence/bq035-v60-publication-2026-10-16/source-binding.json)。

生产运行文件 162 个及原 hosting 文件逐项与本地已测产物一致；包中另有三个原有 drizzle 元数据文件，共 166 个文件。自动核对结果中的 `runtimeFiles: 163` 包含 hosting。入口及内核身份如下，完整逐文件清单见 [archive-verification.json](evidence/bq035-v60-publication-2026-10-16/archive-verification.json)。

| 产物 | SHA256 |
| --- | --- |
| Worker | `8f1d60744c66cbbdcb09ad4cfaf0f68bd8f0c5010b4b32d6887240929d862e8f` |
| 前端 index | `46d4fc5a0be50884c32a3b066fa6c560daae9d94a709938c7bd87405f5452e00` |
| 当前 ABI78 WASM | `9511a72c960cf3e0fa249d62669724875d475ab245de4f9616615f38df43fd5f` |
| 保留原 ABI77 WASM | `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c` |

沿用既有 project、公开访问及 revision 2、逻辑 D1 绑定 `DB`，schema、迁移文件和 wrangler 配置与原 v59 相同；没有清库、重置、读写真实用户桌或重建旧房间。旧 ABI77 的精确内核和路由仍随包保留。线上旧房间恢复没有在本次浏览器中重新验证。

已审验证口径保留原样：Native 1130 lib + 67 integration、前端最终 1229、核心修复后完整 Worker 204；产物隔离后未重跑整个 Worker 组，所引用 Worker 和两个 WASM 字节未变，之后真实本地 HTTP 严格双内核链 20 命令通过。BQ035 13 场景／378 命令／1510 投影、56 负例／448 restore 调用，以及 current78 全量 433 cases／24403 transitions／96975 projections 均见原证据。本次没有把局部结果叠加成新全绿数，也没有将本地 HTTP 或组件检查算作线上试玩。

上传 gzip、解压后的本地 tar 与平台保存 tar 的封装哈希不同，分别如实保存在 [archive-identities.json](evidence/bq035-v60-publication-2026-10-16/archive-identities.json)。本地包逐运行文件核对通过；平台回执确认 source、166 文件及成功 deployment。未下载平台归档重新比对内部运行字节，不声称整份 tar 字节相同。

## 线上访问仍受证书阻断

发布成功后在 `2026-10-16T16:15:19.844388+00:00` 使用全新 Chromium context、普通 TLS、无忽略 HTTPS 错误、无 bypass 进行一次访问，得到 `net::ERR_CERT_AUTHORITY_INVALID`。未进入 UI、未取得线上资产身份、未访问旧邀请／真实用户桌；POST **0**，建桌／加入桌 **0**，线上完整对局 **0**。

因此发布与 main 提升已完成；真实线上试玩仍未完成。没有重复访问、绕过证书或变更站点安全设置。错误、脚本及截图见 [normal-browser.json](evidence/bq035-v60-publication-2026-10-16/normal-browser.json)。后续需在正常 TLS 可用且线上版本身份一致后再进行隔离线上试玩。

新增证据文件的完整校验清单见 [evidence-file-hashes.json](evidence/bq035-v60-publication-2026-10-16/evidence-file-hashes.json)。发布 tar、原始大夹具、数据库、凭据和构建缓存均不加入仓库。
