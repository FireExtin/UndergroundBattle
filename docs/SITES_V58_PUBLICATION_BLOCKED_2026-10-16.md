# Sites v58：版本已保存，生产部署被自动审批拒绝

此文记录本线首次尝试的历史阻断。父会话后来获用户单独批准，已于 `2026-10-16 08:53:55 UTC` 成功部署同一已保存 v58，deployment 为 `appgdep_6ad1e61a0de881918e24c273e78f3eee`。当前发布结果以 [v58 成功回执归档](SITES_V58_PUBLICATION_2026-10-16.md) 为准；以下原始拒绝事实及当时状态保留，不再表示当前未上线。

父会话确认独立复审通过后，明确授权将源码 `7ce4cb4d3b1af7bcd5502dc4f96e522d70cabc08`（证据 `9b876258012b0995357ab73aede394a2f7bfa3c1`）发布到现有隐秘世界官方 Sites，并要求权限拒绝即停止、不绕过。此记录覆盖这一次官方发布尝试。

发布前官方仍只有已成功部署的 v57，本批没有重复保存版本或部署。保留原项目 `appgprj_6abf7bf54a7481918a50e1ef1509ca68`、公开范围 `public` 及 access revision 2。

## 已完成

- 官方源仓库先打开、核对远端，原有 828 个源码绑定匹配；仅同步 8 个已审前端文件与来源元数据，最终 832 个源码绑定匹配。
- 官方 provider source：`eec9ffb81b66b149bdcc4f045c1be834dae21748`，已由官方 helper 推送并打包。
- 官方构建通过，158 个 client 文件与已验产物逐字节相同。Worker SHA-256 保持 `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`，WASM 保持 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`；归档中的 3 个迁移文件与 v57 完全相同。
- 官方成功保存 v58：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_de61c2cbd32c819187d3476b74d9e8f9`，source 精确对应上述 provider SHA。
- native 保存归档：166 个文件，96,409,600 字节，`sha256:da6748f109cde6e55627ec0b28fde4ee789da5882eaf5225fe823061f282be9b`。

上传 gzip SHA-256 为 `84c890beda4e595fc913e7d7624a2fe95b05eb38ed4c7162fdd3002f5666cdfc`，解压 tar SHA-256 为 `a22fbdd2bd520921174d0760a86310abf6e939e5838941ce27b470d678e76b90`。native 保存端的 tar hash 不同；如实保留两侧身份，不声明整份归档字节相同。源码绑定、上传前运行文件及保存版本来源分别核对。

## 当时的阻断与状态

只调用一次官方 `deploy_site_version`，自动审批返回：

> This action was rejected due to unacceptable risk.

理由是该操作会发布到公开生产站，而审批仍认为可见授权仅允许开发/审查、要求单独发布批准，没有本次精确部署授权。当前父会话明确授权没有被该审批检查采纳。按指令停止，没有重试、旁路、直接调用平台部署接口或更改权限。

拒绝后只读核对：v58 的 `deployment_id` 仍为 `null`；v57 保留原成功部署 `appgdep_6ad1d374ecd481919deffc8ea851c0b7`。没有新的生产 deployment ID 或成功回执，不能声称 v58 已上线。现有正式地址仍为 https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site 。

后续若审批允许，应续接已经保存且有归档的 v58，不必再保存一个重复版本。当前不继续执行该步骤。

未访问真实用户桌，未修改数据库内容、Worker/WASM/迁移或公开范围。未合并 GitHub main、创建 PR、触发 Actions 或启动浏览器试玩。源码凭据仅用于官方 helper 的隐藏 stdin，已经从会话内存清除，没有写入证据。

完整拒绝回执和拒绝后身份见 [publication-blocked.json](evidence/sites-v58-blocked-2026-10-16/publication-blocked.json)、[postattempt-site-identity.json](evidence/sites-v58-blocked-2026-10-16/postattempt-site-identity.json)。同目录保留保存版本、源码绑定、构建与制品检查、归档身份；没有归档本体、数据库、缓存或凭据。
