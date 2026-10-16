# 官方 Sites v59：让过上下文已发布

父会话对让过批次完成独立复审、确认无阻断问题后，明确授权沿现有官方 Sites 流程公开发布。本次没有新增 Site、改变受众或请求额外确认。正式来源与发布回执均由官方 source helper／Sites 原生接口返回。

## 正式身份

| 项目 | 结果 |
| --- | --- |
| 公开 URL | https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site |
| Site | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| 版本 | **v59** |
| 保存版本 ID | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_bf06ac4bc0cc8191be6dea5da96585d5` |
| Deployment ID | `appgdep_6ad2157c32a48191a5ba22a02b63dc28` |
| 官方结果 | **succeeded**，2026-10-16T12:16:09.509155+00:00 |
| 正式 provider 来源 | `a2a002fb7cafb9030405853a1c98bd91800b627c` |
| 受众 | 原 `public`／access policy revision 2 保持 |

保存结果、部署结果及部署后的 `get_site_version` 回读均绑定同一 v59 ID、来源 A2a002f 与 deployment ID；部署后 `get_site` 仍为原项目、原公开 URL、最新版本 59。成功部署不等于已完成线上试玩。

## 精确发布范围与来源

只物化已审 `7ce4cb4d3b1af7bcd5502dc4f96e522d70cabc08` 加 `d3406a5d719d2ce748b7935350ec56c0dc65467a → 8a27c1ca9cfc065cc03a543d73f14b140ac93515` 的补丁，覆盖树 `ab1ea260a47cb028085dcfede5be9019b6098d87`。没有发布开发分支全树，没有 BQ035／ABI78，继续使用 143 卡 ABI77。

正式 source helper 先打开原 provider `eec9ffb81b66b149bdcc4f045c1be834dae21748`，确认没有远端推进或本地未保存变化。29 个映射文件写入前均核对原内容等于已审基线；再核对全部 **1122 个有限源码文件**与覆盖树的普通 QA 副本逐字节一致。正式站已有的 `package.json`、`web/package.json` 仅多出原有测试 fixture 还原钩子，按已发布 eec9ffb 的原字节保留，没有覆盖这些既有支持文件。

`source-provenance.json` 固化基线、补丁起点、已审源码、覆盖树、复审证据 `0dc88c4ae48f67a3b14c6a0928d94c5a4e9fef38` 及产物清单 SHA。官方 helper 在全输入审计通过后正常 commit／push 成为 `a2a002f`，再打包。provider 差异恰为 29 个映射文件加来源说明，共 30 个；无数据库、迁移、D1 绑定、WASM、卡池或 `.github` 改动。

本次逐字节提升已审产物，没有声称重新编译：对 `0dc88c4` 中完整 `artifact-manifest.json` 的 **162 项**先核对 QA 输出，再核对正式目录／归档；其中 160 项是运行归档文件，另外 2 项是原始 JS／WASM 输入。官方部署归档共 164 文件，额外 4 项仅为原 hosting 与原 drizzle 元数据，逐项核对不变。旧 provider 的被忽略构建目录先移到独立磁盘保存，没有删除工作。打包在磁盘 TMPDIR 运行；首次对 `/tmp` 的预检识别 tmpfs 后没有启动构建，改用既有磁盘构建守卫后才执行 helper。

WASM SHA-256 保持 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`；Worker 为已审 `a94d911f5b2aaa5abbc7ee442d08fbb36a0e9258b4e21b2a8b4fd091af27cb92`。本地 gzip 包 SHA-256 `6462d1f0f58ef5f26c04f9f497075ef1da5cbaafdd580b494224d666f3367492`；官方保存后 tar 的原生 content hash 为 `sha256:4cc8ef4306461d28fa8200c40a98d480b8fd9f87f8e89dd146f9f856af6290b8`，二者容器格式不同，分别记录。

原牌桌继续使用同一 D1 `DB`、相同 schema／迁移、相同 ABI77 reducer 和原状态／回执；没有清库、重建数据库、运行数据回填或访问用户牌桌。没有新 worktree、强推、合并 GitHub main、触发 Actions 或部署交易所。

## 浏览器实测与限制

部署和身份回读成功后，仅用一个空白 Chromium context 对上述原公开 URL 做一次正常 TLS 导航，`ignore_https_errors=False`。**2026-10-16T12:17:27.854156+00:00** 实际得到 `net::ERR_CERT_AUTHORITY_INVALID`；立即停止，没有绕过，没有使用旧邀请码或座位凭据。

UI 未到达、资产下载身份未能在线核对，POST 0、建桌 0、入桌 0、完整线上对局 0。红黑蓝灰选牌、连点、双端刷新、连续让过与完整对局均未执行，不能把已过的本地 1211 前端／189 Worker／1 Native 定向、构建及双浏览器检查当线上试玩。这是本环境的实际证书信任阻断；官方原生部署结果仍是 v59 succeeded。

完整白名单回执、源码／归档审计、来源说明、正常浏览器 JSON／脚本／截图、逐文件哈希见 `docs/evidence/pass-history-publication-2026-10-16/`。没有保存来源凭据、数据库或编译产物。BQ035 WIP 保留在原独立进度线，仍不能由本次发布结果推断其验收或上线。
