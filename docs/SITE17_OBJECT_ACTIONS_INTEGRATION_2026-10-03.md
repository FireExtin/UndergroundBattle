# Site17对象动作交互切片集成

输入为父已复核并明确授权的远端 `FireExtin/UndergroundBattle` 分支 `codex/object-actions-frontend`，精确提交 `39d1ead5430581228ef54c0ef7337db4eabeb5ed`（包含 `d0fc8bd` 与显示分组/缺目标修补）。Library v1下载失败后已停止重试，本轮按父后续指定的Git远端取得真实对象。输出落在独立 `codex/hegemony-frontend-39d1ead-site17-20261003` 工作树，并用于现有Site发布；发布版本与部署结果以原生Sites回执为准。

以Site17实现基线 `8f700cdcb90cfb5847d75b33bd02a69fa65d8664` 无冲突快进至上述提交，只有审核过的8个web文件变化，无额外产品修改。对象动作先选择能力/模式/费用分组，再点选授权投影中高亮的目标/地区并确认原合法动作；保留显示label/description区别，缺失投影目标禁止提交。刷新或版本/选择更新令本地目标草稿失效。此次新增文档只记录交接，不改变行为。

实际验证：完整前端24文件253项通过，无跳过；其中原有 `V028DeckEntry.test.jsx` 两项使用真实编译v028/v027目录及WASM，覆盖自组牌组与四席正常入席/开局、旧版来源拒绝，不代表对象动作的公网独立UI验收。对象动作新增测试为合成授权View上的React交互测试。既有Worker22项通过，无跳过；TypeScript与Vite构建、Worker dry-run均通过。

当前WASM与Site17发布源码逐字节相同：1,619,119字节，SHA256 `6bbd2ce9ffdefdb3c24b7c0ca1786440ce087e4be34cff6af7f6c4af5e7bbc51`。冻结v021–v027及JS均逐字节相同；最终Worker八个WASM文件hash集合一致。真实目录完整比较一致，仍48卡、5预组、10世界，`hegemony-pdf-v1 / limited-v2.6 / rust-v0.2.8`。Rust、WASM与Worker源码树均与基线相同，D1/migrations和房间路由未修改。

秘社 `0726c0ac7437c7e293a363a20692d66a69b1c6f1` 始终位于另一私有分支，本集成不包含该提交、候选/fixture ABI或第九核。本轮没有访问父或朋友的真实房间，也未建隧道或新域名；portable环境无父可用本地预览转发入口。父将在既有Site成功发布后亲自进行独立UI验收，自动回归与发布回执不能替代该验收。
