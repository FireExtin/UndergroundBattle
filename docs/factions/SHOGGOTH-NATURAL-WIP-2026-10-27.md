# 修格斯自然入口 WIP 检查点

基于 `1df2185207352d41dec6057b56f261115b6ce453`。本检查点未完成、未独审，所有新增候选关闭准入；不推送、不发布。

新增 BQ050/BQ051 原图定义、命令程序、MSBQ02 明确能力、独立核心构筑字段与 50+1 开局初始化，以及对应原图、公用索引和 UI 构筑支持。普通牌组仍至少 50 张、至少 12 张真实命令；不据此裁定 49+1 非法。核心休整名誉支付与普通费用分开。

封闭测试注册表仅开放本候选进行严格构筑验证，再调用生产共用初始化。种子 84、50 张普通牌加独立 BQ046、487 条真实 Room 命令，在第 4 回合完成部署、死亡返回核心区、命令发动预测。不是公开大厅准入成功，也不是浏览器试玩。

最近实测：`shoggoth-native-04`，12 项发现，11 通过、1 失败、0 ignored，2684 filtered。失败为 `shoggoth_bq050_both_targets_real_program_and_response_cancel`，实际错误“破坏欲目标不合法”，尚未修复。完整命令为 `node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib shoggoth_ -- --nocapture --test-threads=1`。日志及资源回执位于 `/workspace/bq084-validation/logs/shoggoth-native-04.log` 和相邻 result.json；总耗时 91.07 秒、子进程最大 RSS 2653196 KiB。原失败日志保留。

`shoggoth-typecheck-01`：`npm run typecheck` 通过。此前 `shoggoth-native-03` 7/7 通过，属于较早源码子集，不能替代本检查点结果或与之累加。Rust 1.90.0、Node 24.19.0，锁定离线依赖、单构建作业及单测试线程。Cargo.lock 未修改。本增量尚未执行完整库、同源 WASM、UI 和 Worker 集成验证。

未完成边界：BQ048 重置模式对核心区对象及离场后标志的作用域尚待原文裁定；三失控核心派遣在响应中降失控后的选区语义未知，当前明确拒绝该候选入口，不声称规则禁止；MSBQ02 三任务胜利没有真实已登记任务牌正例，未执行正向验证。未审候选不计入正式完成批次。不得重试此前已拒绝的独立复审。
