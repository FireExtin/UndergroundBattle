# 收藏区布局修复：Sites v56 发布回执

用户授权、父会话独审确认 `7146467ef1994debd9613e2f3a0f1d2a45b37632`（运行源码 `3492f7775cfd470f18164c9dfe847b3c2e3cb7ec`）无阻断后，已沿现有官方 Sites 流程发布为 **v56**。原生部署返回 `succeeded`，完成时间 `2026-10-16T04:15:34.383385+00:00`。

- 公开地址：<https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site>
- Site：`appgprj_6abf7bf54a7481918a50e1ef1509ca68`。
- 官方源提交：`37d5d090c54c05d78dbdad06eafad6a880a60096`。
- 保存版本：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_0542c88f409c8191acf756762cd66bd5`，版本号 56。
- 成功部署：`appgdep_6ad1a4d963788191a38b50e9b176f385`，其 `version_id` 与保存版本一致，保存版本的 source 与上述官方源提交一致。

## 源输入与制品

发布前原生版本列表仍以 v55 为最新，没有发现前两次 UNKNOWN 启动已创建新版本。通过官方 helper 打开现有干净源仓库，核对远端为 v55 的 `70fa29f69c1f204f810d7274f44499e4f522ddce`。仅同步已审 `DeckLibraryPanel.tsx` 和 `lobby-pages.css`，另更新来源记录；不覆盖其他远端变化。

**827 项源输入逐字节匹配**原仓库，两个 package manifest 仍仅保留既有无损 fixture 恢复 hooks 差异。官方 helper 重新生产构建、普通推送及打包成功；**158 个前端文件全部与固定3492f777的已验收制品匹配**，并核对了已提交的冻结制品清单，入口为 `index-CmsQW2b0.js` / `index-BWMrMCsD.css`。

Worker 仍为 47,324 字节，SHA256 `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`；WASM 为 2,909,006 字节，SHA256 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`。没有修改内核、卡池或存储配置。

发布包166文件，原始gzip为68,898,961字节、SHA256 `928e6701ba9c08eb591f8ad1d5b8f158d5845925d92690c0cd9c8b4f922f2d8a`；平台保存tar记录为96,389,120字节、内容哈希 `sha256:8326c57ab84dda9de5524e94910d3788286fb4edbc1d5fe001d72d252307d59f`，两者口径不同。包内未带源码树、数据库、凭据或缓存。

按公开受众使用 `save_site_version` 后 `deploy_site_version`；发布后原生查询确认最新版本56、`access_mode=public`、策略revision2和URL均保持。未启用 push 自动私有发布，未修改权限、环境变量或GitHub main。凭据只经隐藏stdin传入官方helper，没有落盘，使用结束后已清除会话变量。

## 验证范围及剩余限制

已审源码此前的生产构建、四个PC场景各33项、既有37项本地生产浏览器、124文件/1158项完整Web通过结果仍适用，见[布局独审材料](DECK_COLLECTION_GRID_HEIGHT_REVIEW_2026-10-16.md)。本轮在官方源checkout重新构建并比对全部制品，没有将旧测试计为新跑的Native或Worker全量结果。

在成功部署身份匹配后，以正常Chromium访问公开URL，仍得到 `net::ERR_CERT_AUTHORITY_INVALID`，**执行线上UI检查0项**；系统CA和正常代理请求在 `2026-10-16T04:16:21.866742+00:00` 返回403。具体内因尚未定位。没有忽略证书、添加owner bypass或授权头、绕过403，也没有游戏POST。因此v56官方部署已成功，线上页面交互和真实试玩仍未验收，不能由本地浏览器结果代替。

[发布回执](evidence/deck-collection-grid-height-2026-10-16/publication-site56/publication-result.json)、[线上访问限制](evidence/deck-collection-grid-height-2026-10-16/publication-site56/live-access-summary.json)、[全部证据哈希](evidence/deck-collection-grid-height-2026-10-16/publication-site56/manifest.json)已保存；包含827项源绑定、158项制品绑定、官方构建/推送日志及archive身份。发布包本体和构建缓存未加入Git。
