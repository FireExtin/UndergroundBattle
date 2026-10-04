# MSJC09 独有字段修正（2026-10-04）

## 依据与最小输出

父线程对私有候选 `3ac482808460fb192cd0986c114deae6e9a4e410` 源审发现目录错误。执行器重新实际目视仓库《霸权说明书》物理第5页／印刷P4 左下“秘社牌详解”第1项，原文规定“所有秘社均为独有卡牌”；MSJC09 原图的标题也是金色。此前以没有单独标记推断非独有的判断错误。

本次唯一运行时改动是 `rust-game/src/society.rs` 默认真实 MSJC09 定义增加 `unique:true`。原单元测试的否定断言改为肯定，正常 HTTP Worker 测试新增 catalog 字段断言；原候选说明同步更正，并新增本篇说明，共五文件。每玩家的秘社选择最多一张，本次没有证明自然对局发生独有冲突或增添相关机制，只纠正不能发布的错误元数据。

该私有分支仍为 `codex/hegemony-msjc09-society-20261003`。结算、费用、发动权利、四席秘社区结构、普通48卡、前端和八冻结内核没有改变。默认 registry 仍只含 MSJC09，fixture registry 仍只有三张人工秘社；`societySupported` 仍来自实际 registry。

## 重建与本次执行证据

- 默认 native 完整测试重新执行：106库＋32集成＝138通过，包含更正的独有断言与原 MSJC09 五项回归。
- 默认 WASM 重新构建；默认 native oracle 重新生成，全量 native/WASM 对照重新执行：27案例、1143状态转移、4091投影、1报价、7拒绝命令，全部一致。两轮完整 oracle 的原始字节比较证明，除 catalog 的该布尔字段外，27案例及其余字段全部相同；因此原状态轨迹可沿用，更正 catalog 即可。
- 默认后台 Worker 仅本地 dry-run 重建，九核。重新执行三个受影响入口：MSJC09 四席、完整版本路由、通用 Worker/D1 集成，3通过／0失败／0跳过。真实 MSJC09 场景461接受命令、2346 HTTP调用，两次付费帧的实际 workerd/D1 重开均在响应到期前，四席隐私、原收据、正常同 instance 下一回合恢复通过。
- 直接加载重建 ABI 的 catalog 与首轮 ABI 比较，唯一字段差异是 `societies[MSJC09].unique` 从 false 到 true；普通卡和预设与冻结 v0.2.8 相等。旧私有候选的带秘社初始 Room 在修正版仍能读取，初始 opaque 状态和创建席投影一致；本次四席投影另由上述 Worker 场景复核。
- 当前默认、fixture 与冻结 v0.2.8 交叉身份恢复仍拒绝；实际九核 Worker 路由通过。身份字符串保持未发布的 `rust-v0.2.8-msjc09-candidate`／`rust-v0.2.8-msjc09-fixture`，不新增正式版本，候选审核时用新提交和 WASM 哈希区分产物。

默认新 WASM：1,633,006 bytes，SHA-256 `3e258d6c28c4086c5623c00ee7aea9ef352bfd983db3f0d3697adcd7109797e0`。首轮默认 WASM `cb000d6b…` 因错误字段被本产物替代。

## 可沿用的证据及边界

`society-fixtures` 不编译改动所在的真实 registry 分支，原五项 MSJC09 单元测试也不编入 feature。本次重新运行 fixture registry 聚焦测试1项通过；实际加载现有 fixture ABI 得到的完整 catalog 与原证据相同，ABI 字节仍是1,635,383 bytes／SHA-256 `f2e71812608304eb0c8e833a6464f310f4934358bc72f33f0f6108d19616e55f`。八个冻结 ABI 全部逐字节哈希复核一致。

因此沿用首轮 feature Rust138、fixture WASM163转移／646投影、fixture Worker1通过的完整证据，不声称本次重新构建或完整重跑了它们。默认 Worker 首轮完整23项仍作为其余未变路径的历史证据；本次仅重跑上述3项，不能写为修正版完整23项再次通过。默认 Rust及默认 WASM完整结果则为本次重新执行。

证据是本地 native/WASM 与真实本地 workerd/D1 自动化，尚无自然浏览器UI或公网验证。本次不推送、不部署；Site18 源码 `0187d15884bccedba2ec4770cb9c2ec46a3a9d4b` 和前端主实施 `34db569c0ce0bd4326697405336505f0d7f75a4f` 的工作树均未改动。父线程下一步审核修正增量后决定整合。

## 增量交付

增量包以 `3ac4828` 为基线，提供五文件差异、五份提交源码、增量 Git bundle、新默认完整 WASM ABI、重建 Worker 入口、原文页图、MSJC09 原图、原始日志、实际 catalog、ABI／registry／Worker 文件哈希证明及 oracle 摘要。其余基础源码、八旧核和不变 fixture 产物沿用首轮复核包；不重复打包全部初始准备。

原始目录：`.private-validation/msjc09-unique-correction-20261004/`。《霸权说明书》PDF SHA-256 `a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886`；原文页图 SHA-256 `8ec69cf85783090dd299eadde3396938238d4973b9ae3c62cd7bf11bcfe12418`。完整默认 oracle 在本地保留并记录新哈希；增量包仅附 catalog 和全量案例计数，不重复装入约118 MB的完整状态轨迹，首轮 MSJC09 轨迹可以使用更正后的新 catalog 比对。
