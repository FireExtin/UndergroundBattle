# 同一Rust规则核的WASM边界

本crate以`hegemony-server`的纯规则部分构建WASM，关闭Native HTTP/SQLite特性。Sites Worker在服务端调用它；浏览器展示投影，不运行另一套权威规则。当前版本及已验证阶段见[根README](../README.md)和[整改记录](../docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md)。

## 构建与验证

从仓库根目录执行。需要Rust1.90.0、Node、`wasm32-unknown-unknown`及`wasm-bindgen-cli`精确版本0.2.104；缺失时按根README安装。先设置独立普通磁盘`UNDERGROUNDBATTLE_BUILD_ROOT`。

```sh
bash rust-game-wasm/build.sh
bash rust-game-wasm/verify.sh
```

`build.sh`输出被Git忽略的`pkg/hegemony_wasm.js`、`hegemony_wasm_bg.wasm`及类型声明；`WASM_BINDGEN`可指定CLI路径。脚本通过现有[资源预检](../tools/build-storage.mjs)运行，不绕过磁盘/内存门禁。

`verify.sh`依次构建当前核、由Native生成真实结果、在Node中实例化WASM并比较完整状态/各席位视图，再调用现有Rust测试导出当前XQ24输入检查恢复入口。它只清理自己创建的临时夹具目录。当前已记录433场景对照及XQ24检查，详细阶段差异保留在整改记录中；并非单次最终端到端命令全套结果，更不是浏览器试玩。

`--society-fixtures`只生成隔离的`pkg-society-fixtures`测试内核；不是公开准入牌或发布产物。

## 导出接口

[src/lib.rs](src/lib.rs)定义以下JSON字符串边界：

| 导出 | 用途 |
| --- | --- |
| `catalog` / `stateIdentity` | 当前公开目录 / 严格读取存档身份 |
| `newGame` / `joinGame` | 预组入桌 |
| `newGameWithDeck` / `joinGameWithDeck` | 自组牌组入桌 |
| `roomCatalog` / `view` | 经版本及席位验证的目录与席位视图 |
| `applyRoom` | 完整RoomCommand及可信服务端毫秒时间驱动的状态迁移 |
| `pollRoom` | 可信时间驱动的到期处理与状态读取 |
| `quoteRoom` | 响应操作报价 |

旧`apply`导出保留但直接拒绝：schema3必须使用`applyRoom`。客户端时钟不能替代可信服务端时间；decimal seed/time和完整opaque状态应保持原字符串，不经JavaScript Number重写u64。

## 精确历史内核

当前候选ABI90与原始ABI77–89分别装载。历史JS/WASM运行字节不在Git，身份与来源记录在[frozen](frozen/)，由[统一loader](../tools/frozen-kernels.mjs)校验JS、gzip、解压长度和WASM哈希。输入准备及v71原件来源见根README；不使用新源码重建结果替代旧字节。

`tests/current89-kernel.mjs`这个历史命名现在明确加载冻结v71 ABI89；当前新编译核的比较入口是`tests/current-kernel.mjs`。旧fixture保留原身份，不能改版本字符串后冒称新Native结果。缺失冻结输入会使相应测试初始化失败，应先补齐输入而不是放松哈希。

Sites负责旧桌的内核选择；单个WASM模块和Native服务都不提供任意历史状态迁移。
