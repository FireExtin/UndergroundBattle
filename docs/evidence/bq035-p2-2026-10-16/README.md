# BQ035 P2 最终本地证据

汇总为 `verification.json`；机制说明见 `../../BQ035_P2_WM023_TARGET_RESOLUTION_2026-10-16.md`。Native1130＋67、新BQ035完整13/378/1510与56负例／448调用、来源44、工具10、前端130/1229、核心修复后Worker完整204，以及新生成旧机制433/24403/96975均通过。结果不跨轮相加。

`browser-final-report.json` 与 `browser-*.png` 是最终真实监听HTTP、GameApp、workerd和D1双浏览器结果，20命令、POST12／9、两次重开、原回执及实际移动。初始为显式合成布局，服务器时钟在本地测试入口固定0；不是公开线上试玩。最终截图已实际查看。

旧94无引用模块预检失败、实际确认出屏失败、几何前后值及原日志均保留。包装隔离后未重跑204：后台入口及被引用两个WASM字节未变，已按父会话避免重复测试要求保存精确边界，最终HTTP在严格两模块目录通过。

BQ035全量原始JSON在仓库golden gzip，121个编译输入绑定。旧机制原生索引保存在本目录同名json.gz；完整433 case原文位于 `/workspace/bq035-p2-evidence-20261016/current78-old-mechanisms-p2-release.json.cases/`，约2.49GB。原始索引在同一Git外目录；比较器必须读取每个case并核对字节与哈希，缺原文即失败，不能只用索引证明通过。151个源码绑定见 `old-mechanism-native-source-binding.json`。

压缩日志保留原始输出字节，最终文件摘要另见evidence-file-hashes.json。没有凭据、运行数据库、缓存或编译产物。main提升因共享UI已知新增失败及新修复未独审而跳过，单独材料在 `../reviewed-v59-main-2026-10-16/`。公开证书检查在另外两个目录。当前候选仍待父会话串行独立复审，未发布。

最终源码提交为 `53b2114a220ca001ca8c40ff3b2a6f2b6276ef45`；有界普通目录副本1145文件及121编译输入逐字节相同，完整BQ035重放与原77身份检查通过。闭合记录见 `source-closure.json`，后续提交仅补文档和证据。副本不是Git worktree，不含原始档案PDF、数据库或凭据，未在副本上声称跑完整档案／前端来源测试。
