# ABI82发布目录只读身份核验（未发布，待批准）

本次仅核验本地文件、已安装Wrangler源码及Git身份；没有重新构建、运行组件测试、申请凭据、改动Site源、保存生产版本、发布、修改权限或访问远端D1。父会话的发布权限校验拒绝仍有效，本批明确批准仍待回复。没有重试或绕过。

核验开始时游戏分支 `codex/stage-pass-progress-20261016` 本地／远端均为 `95d89d664f12fcd6e9958a475263ca00d9983bfc`，工作区干净，无本批构建／测试进程或待处理源码编辑；main仍为 `d1a170939ac680c74dc7b73e2c1da9647d0bf32e`。本次仅新增本核验说明及 [逐项机器记录](evidence/response-preview-2026-10-17/directory-identity-audit.json)，因此保存非空文档检查点，不改变已测源码 `398e220721f9e14c4f523530183c6288b7749096`。

## 169项与完整物理目录

`/workspace/UndergroundBattle/sites/dist` 的169项运行文件，逐一比对95d89d证据中的 `runtime-manifest.json`：**169/169字节数与SHA256匹配，缺失0、内容变化0、未知运行文件0**。其中客户端162项、服务端运行文件7项（index.js＋6份WASM）；`sites/public` 与 `web/dist` 也分别逐项匹配全部162个客户端文件。

完整dist有172个普通文件，另3项未列入运行清单，全部明确列出：

| 路径 | 字节 | 性质 |
| --- | ---: | --- |
| `.openai/hosting.json` | 91 | 部署元数据；project_id与DB绑定沿用原值 |
| `server/README.md` | 122 | Wrangler生成的构建说明与时间戳 |
| `server/index.js.map` | 248067 | 生成的source map；Worker末尾有调试引用，不是第7份WASM或新增业务运行模块 |

没有其他遗漏／额外文件或符号链接；3项的完整SHA在机器记录中。169是运行清单的计数，不能表述为“整个物理dist只有169个文件”。本次未删除、加入发布包或改变这些文件的处理方式。

## build日志164与客户端162为何不同

已安装Wrangler **4.146.0** 的 `wrangler-dist/cli.js`：`buildAssetManifest`（163075行）先调用 `fs.readdir(dir,{recursive:true})`，163077行直接记录该数组长度；随后163088行对每项stat并排除目录。日志函数162754行将这一未过滤计数称为files。

在相同目录用相同Node递归readdir只读复现：**164项 = 162个普通文件＋`assets/`、`cards/` 两个目录项**。普通文件为158 JPG、HTML／JSON／CSS／JS各1。并非有两份资产从169清单遗漏，也不是由服务端README或source map导致；后两者不在assets目录内。依赖文件SHA与实际行号同时记录，未运行Wrangler发布命令。

## 旧核与官方源隔离

候选server恰好6份WASM。77–81各自与冻结原gzip解压结果、原JS及 `sites/generated/abi77..81` 逐字节相同；新82为3,165,174字节，SHA256 `1479c3d2ac98fa0f3c0aefff4eba409f2f84a28f437833cbb0a654332b752d37`。旧81仍为 `669403d547a0ce451570e7953195315949a8ed8252ede34aa9269a607a4e0583`，没有第7份残留WASM。

官方源工作区 `/workspace/hidden-world-sites-abi77-20261015` 仍干净停在已发布v63的 `90bfb25faa4c142f13140e141b6c8dfe2ec9355a`，其dist逐一匹配原v63的168项运行清单。它尚未被82覆盖：若硬与82清单比较，165项相同、2项内容不同、2项候选文件不存在（新客户端JS和新82 WASM）；这是v63与候选82的身份差别，不能将官方旧目录当作已经准备好的82发布目录。

结论：本地候选169项身份闭合，164/162计数已由实际源码和目录项复现解释；官方仍v63。下一步等待本批明确批准，当前不保存生产版本或发布。

后续状态：用户明确确认本批后，2026-10-17 12:20正常权限校验完成v64发布；本页关于拒绝／等待的描述保留为核验当时状态。精确来源、归档及部署回执见 [v64发布记录](SITES_V64_RESPONSE_PREVIEW_PUBLICATION_2026-10-17.md)。
