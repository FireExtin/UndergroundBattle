# Sites v57：邀请、牌组与手机卡图正式发布

父会话独审通过后，沿既有官方 Sites 流程发布，未合并 GitHub main。发布前核对官方仍为成功的 v56，没有本批重复保存版本或部署。

- 网站：https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site
- 已审源码：`79d09ff879425d043344b53613efa19bf7d0a7f6`；证据：`3e9bef02785afa4955d470f7cde6e04eb13150e1`。
- 官方 provider 源：`27f0956c075491d0d69cecf25d8e993593949672`。
- 保存版本：v57，`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_f3a7ab0b16908191b5198aa84455690f`。
- 部署：`appgdep_6ad1d374ecd481919deffc8ea851c0b7`，官方返回 `succeeded`，时间 `2026-10-16T07:34:26.724768+00:00`。

同步前验证原有 827 个源文件；同步 14 个 Web 文件后，828 个绑定文件与已审源完全匹配。官方构建的 client、Worker、WASM 与已验产物一致；归档中 162 个文件逐字节匹配，另一个构建 README 仅生成时间戳不同。官方附带的三个迁移文件与 v56 逐字节一致，未执行清库、重建或用户桌操作。原项目 ID、公开访问模式及权限 revision 2 保持不变。

本地上传包和 native 保存包的 tar 封装哈希不同，分别如实记录在 [归档身份](evidence/sites-v57-2026-10-16/archive-identity.json)；不声称两份 tar 整体字节相同。源码绑定、已验运行文件、保存版本的 source SHA 和成功 deployment ID 均已核对。

普通浏览器访问仍被 `ERR_CERT_AUTHORITY_INVALID` 阻断，没有绕过证书或权限。此回执确认官方部署成功，不表示线上试玩已通过：UI 未进入、POST 0、真实牌桌操作 0、线上完整对局 0。详见 [普通浏览器结果](evidence/sites-v57-2026-10-16/normal-live-browser.json)。

完整官方回执和发布后身份见 [publication-result.json](evidence/sites-v57-2026-10-16/publication-result.json)、[postpublication-identity.json](evidence/sites-v57-2026-10-16/postpublication-identity.json)。压缩后的源码及产物绑定也保留在同一证据目录，无凭据、数据库或构建产物。

随后新增的“让过到指定阶段”属于独立的开发测试批次，不包含在 v57，也没有获得公开发布授权。
