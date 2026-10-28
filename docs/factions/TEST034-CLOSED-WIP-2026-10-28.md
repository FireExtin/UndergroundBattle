# TEST034 兽性直觉固定 TEST 图版封闭实现

基线 `615f5490a9857114ce3d4fc1e1e1275d2403f9ba`。仅单蓝固定图版，原图完整、机制无待裁定字段。原始计划 `card-specifications.json` 的 B14 条目记录印刷 14/50、TEST 水印，formalReleaseEligibility=unconfirmed。此实现不确认正式版身份，不计入正式新增20张；生产 supported=false，未独审。

原图 `resource/ymsj-fun.github.io/cards/TEST034 兽性直觉.jpg`，SHA256 `51b211a96598671a2fc92dd3192dac1fb146a1bdfdde2e219febe8e7754b0ee5`。白名非唯一，附属／状态／超能力，费用1、蓝色忠诚1、心灵1，非资产。结附目标吸血鬼角色，宿主获得永久调查1；神秘学博弈②为原手牌行动：支付2、弃自身并发起博弈，不套用打出费用或忠诚。复用既有结附、非资产、持续图标、手牌博弈及 Room 事务，不引入新博弈协议。

验收包括任意操控者的吸血鬼宿主、拒绝人类与资产放置、响应期间宿主翻暗失效但不退款、手牌行动支付2及不足拒绝、真实弃牌、结算完成、Game/Room 损坏帧拒绝。自然构筑 witness 仅在私有 registry 打开本卡；生产构筑仍拒绝。实际起手种子有限搜索，开局后不修改布局、资源、回合或选择，逐命令保存实际 Native 重复回执和四座位投影。两条自然链分别完成蓝色资产→吸血鬼→结附，以及中立资产→支付2弃自身→博弈结束。

失败历史保留：01 编译失败、零用例（Rust match guard 空格语法）；02 34过4失败，结附校验错误要求角色绑定携带地区实例；对照原绑定器修为角色目标 region_instance=None。03 37过1失败，witness 仅放一枚蓝色忠诚不能合法部署新生血族；修为真实起手两张巨型蝙蝠并逐张放置蓝色资产，未改生产费用或目标规则。02/03 未发现的 observers 过滤名称已更正为实际模块 import20_observers_tests，发现数按实际运行结果报告。

MSWM02、MSCQ02 与本项统一进行必要全库及同源跨端验证。自测不替代独审，组件和本地 Worker 测试不是真实浏览器试玩。不推送、不合main、不发布；原规则待决项和被拒审查保持原状态。结果在完成后追加。

04 定向实际55项全过、0失败/忽略、2675过滤；包含本卡6项、MSWM02、MSCQ02和原手牌博弈/观察者回归。两条TEST自然链为seed282结附319命令、seed4手牌博弈167命令。全组三卡新Native共5场景1070命令，WASM全部命令/4280视图/1070真实重复调用一致。相关UI 6文件63/63通过；默认WASM构建通过。Rust1.90.0、wasm-bindgen0.2.104、Node24.19.0、锁定离线依赖，单构建/测试线程。全库与Worker状态暂为运行中。

Worker04已完成5/5场景、0失败/跳过/取消，1070命令与1070次冷重开，完成后首条请求回执仍一致；1030.22秒、峰值RSS837728 KiB。前端类型检查与生产构建通过（11.52秒）。全库尚在运行，保存此WIP时不声称全库通过；Rust输入自04定向后保持冻结。

## 合并全库收束

被测源码对应本地WIP `d803695463de62749d24f28d874aea3f45884481`；测试期间378个Rust输入冻结，后续仅追加结果文档。`test034-group-full-04` 实际发现2730项：2729通过、0失败、1 ignored、0 filtered，1774.16秒测试时间（回执1774.77秒），子进程峰值RSS850868 KiB。定向55项与全库重叠，不相加。MSWM02、MSCQ02、TEST034与现有机制一起覆盖，未逐卡重复全库。

命令：`node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- --nocapture --test-threads=1`。设置 `SENTINEL_WORKER_WITNESS=/workspace/test034-validation/full-04/sentinel.json` 和 `SENTINEL_PLATFORM_WITNESS=/workspace/test034-validation/full-04/wm044.json`，两个主体实际执行，分别1场景5命令和2场景6命令。它们是准备布局witness，不称自然开局，未将其与前述1070条跨端命令混计。唯一ignored仍为 `bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment`，原理由 `Internal candidate catalog variant only; mainline admission remains closed`。

三份新Native夹具共用源码清单SHA256 `3dcc9dd2a1a9ca3f00ef4f4030936871d7cfd25fba908baaacc6efed7eb1fd39`，WASM SHA256 `29984b0c9a62e40a115b2ffa0c5042c99f2885b4ca5300fe44a29b75ce32f46c`。TEST034原始Native SHA256 `f10b871c4a2f604b72f683b96583339047e6978b7fcb58e6d5b99312302f7910`，原始字节另存gzip，非仅重签旧fixture。所有锁文件不变。

本轮没有重跑8个独立集成目标、完整web/sites其它套件或真实浏览器试玩；以上结果限于全native库与本组适用跨端验证。01编译失败仅有日志，02保留日志与修复前模块，03保存当轮改动文件快照；不声称01/02已捕获完整编译输入。04有冻结输入清单、实际编译绑定、原始Native及同源WASM结果。失败历史均保留，不以最终通过抹去。

准入、独审、规则待决和发布边界不变：本卡固定TEST图版正式资格未确认，不计正式新增20张；BQ095仍只是基础层，公开新增仍3/20。未尝试此前被拒独审，未推送、合main或发布。
