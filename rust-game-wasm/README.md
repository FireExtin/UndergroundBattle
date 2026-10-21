# 同一Rust规则核的WASM边界

本crate以`hegemony-server`的纯规则部分构建WASM，关闭Native HTTP/SQLite特性。Sites Worker在服务端调用它；浏览器展示投影，不运行另一套权威规则。当前版本及已验证阶段见[根README](../README.md)和[整改记录](../docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md)。

## 构建与验证

从仓库根目录执行。需要Rust1.90.0、Node、`wasm32-unknown-unknown`及`wasm-bindgen-cli`精确版本0.2.104；缺失时按根README安装。先设置独立普通磁盘`UNDERGROUNDBATTLE_BUILD_ROOT`。

```sh
bash rust-game-wasm/build.sh
bash rust-game-wasm/verify.sh
```

`build.sh`输出被Git忽略的`pkg/hegemony_wasm.js`、`hegemony_wasm_bg.wasm`及类型声明；`WASM_BINDGEN`可指定CLI路径。脚本通过现有[资源预检](../tools/build-storage.mjs)运行，不绕过磁盘/内存门禁。

`verify.sh`依次构建当前核、由Native生成真实结果、在Node中实例化WASM并比较完整状态/各席位视图，再调用现有Rust测试导出当前XQ24输入检查恢复入口。它只清理自己创建的临时夹具目录。当前ABI93验证状态见[夹具迁移记录](../docs/CURRENT_FIXTURES_ABI93_2026-10-21.md)；历史433/440场景结果属于先前版本，不能充当当前版本通过证据。回放不是浏览器试玩。

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

## 最新版本单核约定

隐秘世界以后也不提供旧牌桌兼容。每次交付只打包、装载该版本的最新内核，优先降低包体、构建空间与运行开销。当前为ABI93；Sites与Native均明确拒绝旧规则/卡池/引擎身份，保留旧数据并要求新开，不迁移旧opaque状态、不引入旧核路由或运行保留包。

当前比较入口是[tests/current-kernel.mjs](tests/current-kernel.mjs)。有效机制测试使用当前Rust真实导出的结果，并绑定当前源码清单与WASM哈希；旧原始状态只保留为拒绝边界和历史审查证据，不能改版本字符串后冒称新Native结果。旧内核loader和打包工具已移除，历史身份清单仍留作追溯。

当前版本的完整身份、schema/状态完整性、u64字符串精度、鉴权、费用和幂等保护不因清理而放松。性能结论必须注明实际测量边界；这一约定不适用于其他项目。
