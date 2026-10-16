# 收藏组牌独立页面：Sites v55 发布回执（2026-10-16）

## 结果与输入

官方 Sites 已将本批次发布为 **v55**，部署状态为 `succeeded`，完成时间为 `2026-10-16T02:44:42.519051+00:00`。

- 公开地址：<https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site>
- 已独立复审的运行源码：`45bc54994569bb22dbd8a61a114b5878253cb88b`。
- 打包时原仓库提交：`1900cdd7e07fe2fc64200000a11312a4df07d16d`；相对上述提交仅补充证据，`web` tree 同为 `23652f1225ad35c1a8102a82c59cf835cd304ef8`。
- 审查分支：`codex/deck-collection-page-review-20261016`。
- Site：`appgprj_6abf7bf54a7481918a50e1ef1509ca68`。
- 官方源仓库新提交：`70fa29f69c1f204f810d7274f44499e4f522ddce`。
- 保存版本：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_29996bca7b048191b3cc0562288c45c9`，版本号 `55`。
- 成功部署：`appgdep_6ad18f8d0e4c8191a85b5f0590db0c07`，其 `version_id` 与上述保存版本一致。

本批次将首页、入席、卡牌收藏与组牌分开；收藏页包含真实字段筛选、逐卡组牌准入、草稿与已选牌组快照恢复，以及进入可见范围后才创建的卡面图片。机制、Worker、WASM 与既有存储绑定保持同一制品。实现及完整验收见 [源码审查材料](DECK_COLLECTION_PAGE_REVIEW_2026-10-16.md)。

## 源码与制品绑定

先通过官方 `site-workflow.mjs` 打开现有源仓库，确认远端为 v54 对应的 `cec2bd64b847d47ac971a459dc8ef2d61aec9e89`，本地工作区干净。复核上次 820 项输入的字节后，仅同步相对已发布来源变化的 22 个 `web/src/game/` 文件，另更新来源记录；保留已有的无损测试 fixture 压缩与恢复配置。

同步后 **827 项输入逐字节匹配**原仓库，涵盖前端源码与资源、Rust 源码、卡定义、Worker 源码与数据库迁移等既定范围。两个 package manifest 仍仅有此前已披露的 fixture 恢复 hooks 差异。官方 helper 完成构建、普通推送、打包，未覆盖远端历史。

发布源 checkout 重新生产构建成功，**158 个前端制品文件全部与已验收构建逐字节一致**；运行制品如下：

| 制品 | 字节数 | SHA-256 |
| --- | ---: | --- |
| Worker `dist/server/index.js` | 47,324 | `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6` |
| WASM（生成目录及 Worker 模块） | 2,909,006 | `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c` |

发布 archive 为 166 个文件，原始 gzip 包为 68,897,849 字节，原始包 SHA-256 为 `9ca91442b1d616b18dfe36365f92f606a9b80aaf5026f69043e90bd108c02948`。平台保存后的 `archive_format=tar`，记录 96,389,120 字节、内容哈希 `sha256:522dd9d78be2349386f0a79d7d07edc43d0ea6bba360ea5b006f7d9b51b310f7`；这与原始 gzip 包哈希是不同口径。

沿用公开受众，通过 `save_site_version` 后 `deploy_site_version` 发布。发布后原生 Site 查询确认最新版本为 55、`access_mode=public`、访问策略 revision 仍为 2、URL 不变。凭据仅经隐藏 stdin 传入官方 helper，未落盘或写入 Git；没有开启 push 自动私有发布。原仓库 main 未在本次发布中更新。

## 验证与尚缺的线上检查

针对固定源码 `45bc549` 已完成的发布门槛：生产构建成功；新启动的本地生产预览根页和 catalog 均为 200，返回入口资源与构建字节一致；真实 Chromium 本地生产浏览器 **37 项通过**；完整 Web **124 文件、1,158 测试通过、0 失败**。红、黑、蓝、灰可组牌 ID 集合分别为 13、25、16、13，与 catalog 的真实逐卡准入匹配。本轮不把未重新运行的 Native、Worker 测试计入新增通过数。

**线上真实浏览器冒烟未完成。** 在成功部署身份核验后，分别使用 Chromium 默认连接和已配置的正常 HTTPS 代理，均在根页导航处得到 `net::ERR_CERT_AUTHORITY_INVALID`，执行的 UI 检查数为 0。使用系统 CA、保持证书校验的 HTTPS 请求返回 `403 Forbidden`。这些结果只能说明本执行环境当前无法完成公开站访问，尚未证明具体的证书或 403 内部原因，也不能证明线上应用交互正常。

未忽略证书错误，未添加 owner bypass/授权头，未修改公开访问策略，未提交任何游戏 POST。故发布已成功，线上试玩验收仍需在能正常访问该公开站的浏览器中补齐。本地浏览器结果不能替代线上试玩。

## 证据落点

- [发布回执](evidence/deck-collection-page-2026-10-16/publication-site55/publication-result.json)
- [发布后 Site 元数据](evidence/deck-collection-page-2026-10-16/publication-site55/postpublication-site.json)
- [源码同步范围](evidence/deck-collection-page-2026-10-16/publication-site55/source-sync-plan.json)
- [打包制品绑定](evidence/deck-collection-page-2026-10-16/publication-site55/prepublication-artifacts.json.gz)
- [827 项输入绑定](evidence/deck-collection-page-2026-10-16/publication-site55/source-provider-binding.json.gz)
- [线上访问限制汇总](evidence/deck-collection-page-2026-10-16/publication-site55/live-smoke.json)
- [全部证据哈希](evidence/deck-collection-page-2026-10-16/publication-site55/manifest.json)

原始构建、浏览器、全量 Web 日志与本地截图索引继续保存在此前 `final-review/` 证据目录。发布包、构建缓存、数据库及浏览器 profile 未加入 Git。
