# 修格斯自然入口 WIP 检查点

基于 `1df2185207352d41dec6057b56f261115b6ce453`。本检查点未完成、未独审，所有新增候选关闭准入；不推送、不发布。

新增 BQ050/BQ051 原图定义、命令程序、MSBQ02 明确能力、独立核心构筑字段与 50+1 开局初始化，以及对应原图、公用索引和 UI 构筑支持。普通牌组仍至少 50 张、至少 12 张真实命令；不据此裁定 49+1 非法。核心休整名誉支付与普通费用分开。

封闭测试注册表仅开放本候选进行严格构筑验证，再调用生产共用初始化。种子 84、50 张普通牌加独立 BQ046、487 条真实 Room 命令，在第 4 回合完成部署、死亡返回核心区、命令发动预测。不是公开大厅准入成功，也不是浏览器试玩。

最近实测：`shoggoth-native-04`，12 项发现，11 通过、1 失败、0 ignored，2684 filtered。失败为 `shoggoth_bq050_both_targets_real_program_and_response_cancel`，实际错误“破坏欲目标不合法”，尚未修复。完整命令为 `node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib shoggoth_ -- --nocapture --test-threads=1`。日志及资源回执位于 `/workspace/bq084-validation/logs/shoggoth-native-04.log` 和相邻 result.json；总耗时 91.07 秒、子进程最大 RSS 2653196 KiB。原失败日志保留。

`shoggoth-typecheck-01`：`npm run typecheck` 通过。此前 `shoggoth-native-03` 7/7 通过，属于较早源码子集，不能替代本检查点结果或与之累加。Rust 1.90.0、Node 24.19.0，锁定离线依赖、单构建作业及单测试线程。Cargo.lock 未修改。本增量尚未执行完整库、同源 WASM、UI 和 Worker 集成验证。

未完成边界：BQ048 重置模式对核心区对象及离场后标志的作用域尚待原文裁定；三失控核心派遣在响应中降失控后的选区语义未知，当前明确拒绝该候选入口，不声称规则禁止；MSBQ02 三任务胜利没有真实已登记任务牌正例，未执行正向验证。未审候选不计入正式完成批次。不得重试此前已拒绝的独立复审。

## 后续双目标修正

原图 BQ050 第二模式明确为“消灭结附于敌方秘社的目标附属”。原失败布局错误地把 1 号队友秘社当作敌方；四人队伍由 `seat / 2` 决定。第一次布局修订又把敌方控制的 JC033 放到队友秘社，违反该附属自身的宿主关系；`shoggoth-native-05` 保留这一失败。最终分别创建本方队友 1 号和敌方 2 号的合法 JC033 附属，验证队友目标原子拒绝、敌方成功以及响应移除后整帧取消且费用不退。生产目标合法性未放宽。

`shoggoth-both-06`：1 项通过、2695 filtered；源码基于上述检查点，修订见后续提交。随后冻结 Rust 输入执行完整库及默认 WASM，新 Native 轨迹加入编译时源码绑定与真实重复命令回执。最终统计以本节后的验证结果为准，早期红测不覆盖。

## 冻结源码整合验证（06）

完整 Native `--lib` 实际发现 2696 项：2692 passed、3 failed、1 ignored，1742.59 秒。修格斯 12 项全部通过，已经包含在总数中。3 个失败分别是 `deck_seal_search_draft::deck_seal_draft_dream_filter_includes_transaction_and_character`（期待漏掉基线已存在的关闭 BQ084）、`jc008_tests::jc008_printed_admission_and_finite_modifier_program` 及 `society::tests::every_seat_has_an_empty_non_region_zone_and_registry_matches_support_flag`（完整秘社名单遗漏本轮关闭 MSBQ02）。失败仍保留；证据目录提供三行未应用期待值补丁，不将失败改记通过。

两个通过项 `sentinel_local_worker_room_witness` / `wm044_local_platform_room_witness` 因未设置 `SENTINEL_WORKER_WITNESS` / `SENTINEL_PLATFORM_WITNESS` 而提前返回，主体 NOT_RUN。唯一 ignored 是 `bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment`，原因仍是候选注册表未公开准入。

8 个 Native 集成目标共 72/72 通过。默认 WASM 构建通过，WASM SHA256 `01c45b869cf5a51d9ac935ab50ce286228a70284fdbf377e0f88a053657f1060`。同源比较 11 个场景（10 预设、1 封闭注册表自然构筑）、570 命令、2280 视图、570 重复提交全部相同。原始 Native 输入 SHA256：核心 `14f5a2a79564a3451f906c97f07d2525f3ce9ccc5f1f1453bf585d4940555725`，自然 `dd449e126a553be353350ab43693764a7cd2077cd03543bd1ff4a437e16d41e7`。源码绑定通过后才生成新的候选夹具，没有重签旧数据。

UI 的构筑/牌面/牌组 58 项及核心操作 42 项全部通过，共 6 文件、100 项；本地客户端与 Worker dry-run 打包通过。构筑 UI 测试仅在测试本地注册表开放精确候选 ID，公开目录仍关闭。所有验证仅为本地组件/协议证据，不是浏览器试玩，也不构成独立审查或新 20 张批次完成。

Worker 最终 11/11 通过，0 failed、0 skipped，634.44 秒：570 条真实本地 HTTP 命令、570 次重复提交、570 次 D1 冷恢复；其中封闭注册表自然链 487 条/487 次冷恢复，最后还核验完成后首命令原回执不变。没有触发公开发布。最终源码仍保留上述三项失败；三行目录提案未应用。

主要命令、完整日志及资源回执见证据包 `results.json` 和 `logs/`。环境：Rust/Cargo 1.90.0、wasm-bindgen 0.2.104、Node 24.19.0；`UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/undergroundbattle-build`、`CARGO_BUILD_JOBS=1`、`RUST_TEST_THREADS=1`。所有 Native 构建使用 `--offline --locked`，WASM 使用 `CARGO_NET_OFFLINE=true`。子进程最大 RSS 与共享 cgroup 范围分开记录，cgroup 包含并行本地构建及文件缓存，不当作单进程用量。
