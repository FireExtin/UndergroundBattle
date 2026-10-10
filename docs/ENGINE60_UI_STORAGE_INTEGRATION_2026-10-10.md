# engine60 客户端与 Worker 本地接续候选

从已交付界面候选 `fed806ba0ad29874f192965b93616c5433ce7c7e` 正常合并已审 Rust 提交 `16efe7b170ec1cd099ea31bf7ebc62ad023a240c`，合并提交为 `8e0f90ae2931e844f6d2d342702cd21953cdfdc3`，没有冲突。引擎 `rust-v0.2.60-xq18-carrier-candidate`、牌池 `limited-v2.55-xq18-carrier-candidate`，128 普通卡含 10 地区，另 8 秘社。Rust 生产源码和卡牌数据与已审 16efe 逐字节相同；既有配额恢复、座位保留和牌组存储实现没有修改。

本次生产接续仅更新 Worker 当前元组、登记 XQ18 原图，以及将 BQ028 预览标题改为“你检视的手牌”。计时人实际原图已查看，公开 JPEG 与原 Git 图片 SHA256 均为 `c33ebe42e01874807b60ff591a8e94c62e0ce8bfb1bdef7626bfad10432ed10e`。XQ18 继续使用已有文本选项面板：完整公开实例编号和拥有者区分同名载体，强制选择恰好一个，选项没有 CardView；没有新增专用 renderer、历史路由、迁移或身份机制。

22 个旧 engine57 准备布局由既有有限 Native 测试导出器重新验证当前构筑、执行实际 Room 命令/报价并导出 engine60 文件。比较完整行为时仅规范化版本头和 catalog；生成的 opaque state 完整来自 Native。旧 57/59 文件不变。BQ028 导出器增加检视队友手牌且不弃牌的真实 Native 命令。XQ18 UI 材料从完整 Native 测试的两次同名、同拥有者、同标志数量载体选择及三步暂停/恢复链逐对象提取；状态字符串未重写。生产持久化 loader 保持已审严格契约。

通过的实际验证：

- 完整 Native workspace：879 通过、0 失败/忽略/过滤，含 26 项 XQ18，GREEN Room 断言开启。
- 默认生产 WASM：433 场景、24,403 转换、96,975 私密视图、1 quote 与 Native 完整一致；319 非法命令和 42 非法构筑拒绝，最大 u64 seed 精确。
- XQ18 相关原始 Native 材料：2,177 记录，其中 1,420 快照、723 转换、8,572 完整四席视图；34 非法存档均在 view/applyRoom 拒绝，转换含 16 拒绝命令。范围是原始状态包含 XQ18 实例的记录及全部专用非法/Room 输出。
- React：82 文件、736 通过，包括检视队友的标题、两张同名载体分别选择、强制数量及其他三席私密投影。
- 实际本地 workerd/D1：76 通过、0 跳过，包含 XQ18 选择、暂停阻止选择、恢复、重开和原重复回执。Native 测试 ID 场景经 RoomService/D1；BQ028 及既有整套入口场景另实际经过编译 Worker HTTP。
- 双 TypeScript、Vite、Wrangler dry-run 通过；Worker source-map 全部 8 生产输入与当前源一致。pkg/generated/Worker 中唯一 WASM 均为 2,539,310 字节、SHA256 `ede0815193fd4fbff0e32194138ce4ba3e90b89268486a7c04e92aaba2b274f9`。

Native 完整运行和后续成功专项/UI/Worker/build 的 OOM/kill 增量均为零；默认 WASM 验证的 memory.max 事件增量为 18,175，OOM/kill 为零。XQ18 第一次启动被 1 GiB 余量检查阻止；首次实际专项因 runner 把仅含 state 的非法样本误识别为合法快照而中断，模块正确拒绝。修正测试 runner 分组后完成上述双入口验证；两次原失败记录保留，没有改非法输入。

证据位于 `/workspace/friend-playtest-20261010/engine60-integration`。完整 Native 原始输出逐成员 SHA/模式校验归档后释放重复展开副本；旧三个 ZIP、109 项历史/夹具保护清单及 engine59 原字节保持不变。默认 WASM 的约 2.49 GB 临时输入由 verify.sh 退出清理。已结束预览的依赖目录及 431 个已核对重复构建文件清理记录独立保留；唯一证据、源码和原包均保留。

这是开发者本地接续验证，最终候选仍需父审和实际浏览器验收。现有 fed806 GitHub 审查分支固定供原 QA 使用；本次没有推新候选、创建 PR、修改 main/已审协作分支或发布，也没有重试 Library 401 或被拒灰色复审。
