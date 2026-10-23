# WM044/MSJC14隔离Worker准备：WIP，监听权限阻断

固定已审runtime e8bff04ea1a1aa3be85b55e8e3dd13ffd5bf2b52不变；本轮测试源码e21cec8381c31f1f9477c246ce3a1ba2f2b2b766。没有新增卡、发布ABI、部署脚本、服务层改动。
先核依赖与资源：负载约2.10，可用内存约46GiB；已有只读Miniflare/esbuild/workerd可复用。
全部新编译/执行在CPU29、nice19，避开性能任务预留CPU30/31；复用上一轮固定WASM
0ac2657cb7ca930c910a909e728f62eb893d104f8f6dfe29856a7668beb4c2ce，不重建Rust运行内核。

新本地打包工具只将既有kernel/router/http/service/store与候选WASM打到任务隔离目录。
唯一测试包装为注入本地测试时钟；不修改生产入口，不使用会写回generated/dist的旧脚本。
没有部署或wrangler deploy/dry-run调用。两个原生见证测试实际通过，生成3个场景：
WM044进场选择、私密确认、哨兵继承锁定移动。原生回执为实际工作树输入，不冒称执行时HEAD已是最终提交。

准备4个真实workerd/D1用例：三个显式原生布局后的真实HTTP命令、状态和journal对照，以及
实际HTTP选哨兵构筑、create/join/ready/start；重复点击对应同一command receipt，正常关闭重开Miniflare
沿同一临时D1路径刷新四座位。WASM不是Worker替身，这些断言必须在真实Worker运行后才能声称通过。

实际首次执行：4项全部在Miniflare启动监听阶段报
`listen EPERM: operation not permitted 127.0.0.1`。未执行任何用例命令链，未验证D1读写/重载。
这是执行环境权限拒绝，不是产品失败、资源排队或自动审批拒绝；没有重试、改端口、改通道绕过。
继续完成独立可做的脚本语法检查、原生见证、源码/证据封存。测试仍WIP。

精确命令与依赖/输出位置见证据中的run-sentinel-worker.py和sentinel-worker-first.json。
后续需要获准本地监听的执行环境，再运行记录命令；不能用已有原生/WASM通过替代本次Worker结果。
[verification.json](../evidence/sentinel-worker-wip-20261023/verification.json)绑定原始EPERM日志、所有原生输入回执、
见证与打包输入/产物hash。制品、node_modules、D1与缓存不入Git。未推送、CI、main或发布。
