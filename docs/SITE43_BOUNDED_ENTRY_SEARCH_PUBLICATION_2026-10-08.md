# Site43：BQ104/XQ48 有限进场检索发布

用户对最后BQ原地区消失边界明确回复“同意”后，父端指示按既有Sites授权发布固定树。官方workflow在既有UndergroundBattle云端provider checkout构建并正常推送，原生Sites保存为**版本43**，部署及独立状态回读均为**succeeded**。保持原项目、public受众，不注册新Site，不切换Air，不改GitHub/main。

生产URL（来自原生成功部署结果）：https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site

## 精确身份

| 项目 | 值 |
|---|---|
| 项目ID | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| 版本ID | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_ff8a94b97fd8819184efef4c2db57151` |
| 部署ID | `appgdep_6ac6f4d0c71c819184be22fddb742d8d` |
| 原生部署成功时间 | `2026-10-08T01:42:08.153891+00:00` |
| provider源提交 | `5fb79b63e4a041df712dc3e2cc09205d3dd06ba8` |
| GitHub批准材料提交 | `abe83b05f6a4382d6804cc975cb574821756753c` |
| 固定产品 / QA | `d55a740b97fc75aab25eea3dacbf870385cd1615` / `28a72af4b77fceb27b8452af48d48828dd53ba09` |
| 引擎 / 卡池 | `rust-v0.2.50-bounded-entry-search-candidate` / `limited-v2.45-bounded-entry-search-candidate` |
| 规则 / 普通卡 / 会社 | `hegemony-pdf-v1` / 109（10地区） / 8 |

BQ具体裁定原文见 [裁定记录](rules/BQ104_ORIGINAL_REGION_RULING_2026-10-08.md)。没有改变已审产品文件；34个核心文件哈希保持。用户裁定不使未核实的通用部分结算条款变成已证实原句。

## 构建、源与ABI核对

从批准Git blob与本地文件生成452项有限Site源码投影，并逐项验证provider Git提交和磁盘内容相同。按已审Go退役内容移除27个旧源码文件；原Git历史、原件和数据库保留。D1逻辑绑定、schema及3个迁移文件与Site42完全相同。发布前另行冻结49原WASM，2,248,074字节，SHA256 `9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885`。

官方 `site-workflow.mjs` 在 `/tmp/lantern-site-fce0-official` 调用官方build helper，TypeScript两配置、Vite生产构建、Wrangler dry-run均成功。128个运行/配置构建文件与已审准备构建逐字节匹配；唯一构建文件差异是Wrangler `server/README.md` 的生成时间。当前JS实际实例化当前WASM并读取catalog，确认109普通卡/8会社/engine50/pool45；ABI导出名称与种类和49版相同，这不代表49存储状态兼容。

| 文件 | 字节 | SHA256 |
|---|---:|---|
| Worker | 47,314 | `005c4b99e2fa91b15bb77aa67f0fd6a9a568849f6371412f8602dd6a7d2982e4` |
| WASM50 | 2,277,324 | `e3d26f7041a114a96c9933374a33e63b0c8e45b9a4506b7937fb70cbf99a9d81` |
| 本地最终gzip归档 | 51,048,019 | `c55aa6779e02666de7ba7830568c9657d5a2400cfd2891cb3dba3d5054738944` |

provider历史曾跟踪两个本地node_modules链接。仅从Git索引移除最初被workflow的git add恢复；补充 `.gitignore` 明确忽略这两个链接后，官方workflow正常推送最终提交。依赖仍在本机，最终Git不再跟踪链接，没有删除其内容。此修正不改变运行源码或已有构建，后续workflow复用成功构建重新打包，没有重复同QA。

## 原生保存与部署回读

官方workflow返回精确provider提交和archive后，原生 `save_site_version` 返回43；使用其完整原样版本ID进行 `deploy_site_version`。随后分别回读get_site_version、get_deployment_status、get_site，确认版本43、source上述5fb79b6、部署ID一致、succeeded、owner/public/active及同一任务绑定。源、版本和部署ID均来自原生返回，未自行造号。

原生保存的归档元数据：format `tar`，132文件，69,857,280字节；存储 `content_hash` 为 `sha256:593f58affbeeb2ab2988d8f7f925f0519b152ad0e00dff72fbde03ce8f1b4243`，前后原生回读一致。本地官方归档的132项内容已逐个对照实际构建/迁移文件，全部匹配。

**归档哈希限制保留：** 本地gzip解压后的原始tar SHA256为 `d3be7347483056b25c22e1787c094a9b02842eeac2fa37032a0bdaf5f4fe7358`，与原生存储hash不同。首次错误假定两者必须相同的断言失败，不能改写为通过；常见header标准化检查未证明对应关系，也不能猜测标准化就是原因。尝试通过已知原生Sediment文件ID读取存储归档时，`library.prepare_materialize` 对 `file_000000005a4081f5a3a0586a08c648e6` 返回 `HTTP error prior to action invocation. HTTPException: 404: Library file ownership could not be verified.`，已停止该读取，不换工具、URL或权限重试。因此**未验证原生存储原始字节与本地tar完全相同**。原生Sites保存/版本回读成功，source与归档文件数/字节数一致，按明确授权继续正常部署；完整差异与失败保留供父端审查。

## 用户可见范围及未完成验收

Worker50继续current-only。49旧桌返回 `unsupported_room_version` / HTTP410，需要新建50桌；未进行永久数据删除，保留旧桌数据及冻结49并不代表可以无缝恢复旧桌。

此前公网403没有换请求或权限绕过，也没有重新公共HTTP抓取或公网浏览器验收；原生成功发布不等于公网功能验收已通过。BQ/XQ自然浏览器新机制完成数仍0。已有有限短smoke的1桌、8条真实UI/D1命令、v9暂停保存闭环仍是独立本地证据，未重复运行。手机仅验证390×844渲染。Native/WASM/Web/Worker已审验证结果见候选文档，没有为发布虚报新全量测试数。

完整机器收据、ABI、源投影、最终归档每项哈希、官方构建/流程日志及读取失败见 [publication-site43](evidence/bounded-entry-search-2026-10-07/publication-site43/site43-publication-receipt.json)。未把数据库、座位凭证、Git token、SIWC token或签名截图下载URL提交到仓库。没有新建自动任务或修改已有自动任务。
