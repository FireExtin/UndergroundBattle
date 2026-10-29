# JZ53 尸舞舒拉密兹封闭机制组

基线 `367dd32725aecb80a0833ca76bc400028395e4ec`。原exec断连已恢复，开始本组前工作区干净、无活动测试。新增一张正式黑卡JZ53，复用TK001，不重复计生成物。准入仍关闭，目录308条/公开236条，中立与双色机制不扩展。

## 来源与完整机制

实际阅读 `resource/ymsj-fun.github.io/cards/JZ53 尸舞舒拉密兹.jpg`，SHA256 `a84d896c1b552dabda05a1fc12c55224275a31b63a8408446508775235cc4623`。独有黑色角色、死灵术士、人类/法师、费用4、黑忠诚2、死亡领域、防御1；永久影响1、临时调查1/影响1。复用早已跟踪的公用原图和既有索引/客户端映射，字节不变；新增牌面组件核验，不重复增加索引或移出旧241条/239URL摘要基线。

原文：“进场触发 将你墓地中的目标死亡领域角色牌置于你的手中。”“触发 每当一张角色牌离开你的墓地时，在目标地区生成一个行尸角色指示物。”第一条限本人墓地死亡领域角色，第二条不要求离开的角色具有死亡领域；不能把法术或队友墓地算入。两条沿既有可选触发和目标声明/响应规则。

以真实墓地移除作为观察边界，覆盖回手、回场、置底、封印、墓地打出和多牌费用，不用指令开始/结束时的数量差推测事件。来源在牌实际离开时记录当前控制者、实体及原地区；已经移走来源的观察仍可结算，后来换控制权不转让原触发。墓地中的JZ53自身回场前未在场，不能凭自身离开墓地追溯生成。翻暗或外国控制来源不观察本人墓地。初始化/重开与已出局清空不生成游戏触发。

墓地牌每次离开保留独立事件身份；同一来源/同一事件只有一个声明或响应载体。恢复校验来源、离开牌、操控者、事件、完整程序、目标、待选和响应外壳。目标地区保存实际实体身份，响应换区后不在新地区生成。原尸体已离开墓地后，第一条不能借同名替身继续回收。

行尸依已核《霸权说明书》印刷p5/PDF6为印刷费用数值0、无忠诚、非资产，实际生成后迟缓横置。生命周期沿既有项目裁定：离场消失、翻暗保留、封印保留；不是新官方规则。TK010专用禁止普通现身例外保持不变。

## 验证记录

最终结果在 `/workspace/jz53-validation/delivery/results.json`；原始逐轮日志在 `/workspace/bq084-validation/logs/jz53-*.log`。所有命令通过同一资源包装器记录工具链、退出码、耗时、RSS和cgroup范围，Cargo单作业、测试单线程。本组10/10、相关共享墓地回归317/317，两组327次执行，其中CQ27过滤也匹配本组的jz53_actual_cq27_grave_return测试1项；去重326项，失败0、ignored0。按测试名过滤native/witness导出；受影响的MSJC16自然七封链实际运行并通过，不误报为未运行。未运行全库或8个集成目标。新自然50牌构筑622条Room命令，另有真实BQ088封印11条，共633条，完整重复提交结果由Native实际导出。同源默认WASM633命令/2532视图/633实际重复结果全部一致。UI相关整套8文件153/153；Worker两场景26命令/104视图/26重复提交和冷重开全部通过，失败0/跳过0。自然准备607命令只在Native/WASM执行，Worker执行其真实支付前检查点后的15命令及封印11命令；不声称Worker执行完整自然开局。默认WASM、Web构建和本地Worker构建均通过。

失败保留：01两项被地区实体身份登记遗漏挡住，补精确目标绑定名单而非放宽校验。02目录编辑脚本误定位末尾数组，0过9失败；修正精确cards数组追加并逐项核对所有旧数据未变。03五过五失败，四项夹具误把场上回手助手用于墓地，一项自然见证未闭合；修正实际墓地取牌夹具、自然资产忠诚准备与合法无名尸体输入，未放宽生产规则。

未重试被拒的独立审查、Library401或推送，不合main、不部署。作者自测不等于独审、公开准入、完整20卡批次或真实浏览器试玩。


UI04为152过1失败：JZ53已在旧原图索引中，误将其排除旧241条基线并重复追加索引；移除本轮重复条目，保留旧241条/239URL双摘要与闭合候选负断言，UI05整套153/153。仅UI测试/索引去重修正，Rust与Native/WASM/Worker证据同源不变。

Native原始JSON SHA256 `8c61d258fe73ed70728526811be525968a8d60dc87678218b52430b8c8500f97`；WASM `0599e7f42dcb1f7238b1123f9640a2bbb170af1e8fc61a661bfa9e33a1b0d829`；Rust来源清单 `b649026045378e4adb7bb7f22e010e9f6818989d1769be4ed25738359b017d64`。Rust/Cargo1.90.0、Node24.19.0、wasm-bindgen0.2.104；Cargo.lock SHA256 `1745cc1af931f500e763222c39100da3ac0521f4a1f96fc54e82c8589e2edb54`未变。

## 复现命令

使用新证据目录，保留旧失败与原始输入。设置 `UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/undergroundbattle-build`、`CARGO_BUILD_JOBS=1`、`RUST_TEST_THREADS=1`。

```sh
JZ53_NATIVE_OUT=/workspace/jz53-validation/evidence-04/native.json node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib jz53_ -- --nocapture --test-threads=1
node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- batch3_blood batch3_ghoul batch3_grave batch3_red batch4_wendigo black_expansion black_next20 bq061 bq086 bq088 bq095 cq27 import20_broker import20_gate import20_swords msbq03 msjc16 next20_black_wraiths next20_red pack_a_contest play_sources world:: xq35 next20_death_core --skip native --skip witness --nocapture --test-threads=1
bash rust-game-wasm/build.sh
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/jz53-validation/fixtures-04 node /workspace/jz53-validation/verify-04.mjs
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/jz53-validation/fixtures-04 node tools/build-storage.mjs -- npm --prefix web test -- JZ53Integration Import20CardFaces ChoicePanel.test ChoiceRecovery deckLibrary.test Next20Presentation --maxWorkers=1
```

Worker从sites目录运行 `CURRENT_CLOCK_WORKER=/workspace/UndergroundBattle/sites/dist/jz53-local-04/worker.js HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/jz53-validation/fixtures-04 node --test --test-concurrency=1 test/jz53-integration.test.mjs`。此本地时钟包装器禁止部署。
