# XQ23 / ABI88：v70官方发布证据

固定复审提交 `b7e4c78e5afaf2b73ab5a0fb770451e7b5fd78d4`，实际生产构建源 `ffd2fb8303ba1c173d7f6ce9b1952802c5cc8742`。本轮只按既有授权发布已审产物，不改机制或重新构建。

官方源码 `8061ec2cb987202e68b9a4dd15ff57730364ee03`，v70版本ID `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_330c69ba00808191966dd4b32f9cc469`，部署 `appgdep_6ad5c7e7341c8191ad2f64270f2ce26c`。原公开站于 `2026-10-19T07:34:26.074701+00:00` 返回succeeded，访问revision2、环境revision0不变。

- `official-publication-receipts.json`：脱敏原生保存、部署及回读结果；无写凭据、bearer或个人邮箱。
- `official-source-workflow.json`：原provider打开与helper正常推送打包的完整结果，不含凭据；commands为空，无重复构建/测试。
- `provider-input-audit.json`、`provider-copy-plan.json.gz`、`provider-source-files.json.gz`：1324项有限来源映射及哈希，139个实际Native输入匹配，原钩子/配置/迁移及647历史缓存未变。改变的编辑源为独立副本。
- `publication-archive-audit.json`：185项已审产物逐字节匹配，归档只增加三份原Drizzle文件，共188普通文件；上传gzip与规范化tar身份。规范化tar SHA256为 `1aedce62268cda1a9343fd0288d729ec67157de886f3fc54047db7256ca7ccca`，138,547,200字节，与服务端回传一致。
- `closeout-integrity.json`：发布后双份185项产物、原ABI77–87、新88、原gzip、保护锁、provider干净及游戏main未变的只读确认。
- `cloud-browser-unavailable.json`：云浏览器工具/技能缺失，本轮0导航、0游戏写入、0线上场景、0完整对局；没有用本地或组件结果替代。v70 TLS未检查。
- 两份Python脚本与`evidence-files.json`：本轮实际源码准备和归档审计逻辑，以及原始证据摘要。gzip映射解压后的字节与本地原件哈希相同。

大归档、旧v69生产目录及全部原件留在 `/workspace/xq23-publication-20261019/`，未放入游戏Git。本次没有源码阻断或发布权限拒绝；剩余线上验证需要可访问原公开站的授权云浏览器。当前959,651,840字节磁盘读数不代表一次实际守卫失败，既有1GiB门槛未降。

详细结论见[发布记录](../../XQ23_ABI88_PUBLIC_RELEASE_2026-10-19.md)。原P2相关116/24/15及修前完整1340/1406/632保持原轮次、原范围，均不称为本轮重新运行或线上试玩。
