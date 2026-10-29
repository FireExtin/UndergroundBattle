# MSWM01 曼纽家族封闭实现

基线 `6bc644d140405c6a48a216f7e8efb8382c6549e5`。蓝色正式秘社；不涉及暂停的中立/双色新卡开发。仍 `supported:false`，不是独审通过或发布批次。

原图：`resource/ymsj-fun.github.io/cards/MSWM01 曼纽家族.jpg`，SHA256 `137856482891485cc92ea659ff473da5ad5b7335dbe8b31cf662850d9e37346d`，Society 12/17。重新直接核图，与既有完整字段一致：

- 构筑限蓝、黄、中立，或其他派系吸血鬼/阴魂角色；按印刷类别验证。
- 你展示的阴魂牌在隐秘对抗获得 +1；保留展示时本方明置秘社及必要整局授予凭据，不随之后秘社变化或发动授予重算旧结果。
- 标准行动 4、横置秘社：本局所有本人拥有的吸血鬼和法师角色额外具有阴魂。使用拥有者而非控制者，跨区域/新实例，保留其他类别；可再次付费发动但不叠加类别。新对局清除。资产/封印区遵循既有空白规则，授予记录保留但不在这些区域添加角色类别；返回正常区域后重新按当前类别生效。
- 标准行动横置秘社、横置一个本方阴魂角色：结算时本方秘社没有暴露则抓一张；没有印刷货币费，不要求隐秘标志。响应期间原费用角色离场不退款、不撤销已支付费用。

旧规格的 conditionalActivation / ownedCardGameModifier 标签在本卡原文、完整字段及已用费用/效果语法下可确定；没有新创官方裁定，不推广到未印费用衍生物。原始卡面与两端索引均保留 SHA 校验。

自然链使用真实 50 张构筑（3 张 JC029、47 张 JC125），仅私有候选 registry 打开秘社以验证；以真实洗牌、手牌及指令建立两枚蓝色忠诚资产和四费支付，再部署吸血鬼、发动整局授予、下一可用行动横置阴魂抓牌。无布局/资产注入。另有明确标注的准备场景和对抗场景。

## 失败记录与范围

- native-01：模块引用错误，编译失败，0 用例执行；修正 `catalog::card` 引用为已导入的 `card`。
- native-02：6 项中 2 过、4 失败。测试误用人类/邪教徒 JC042，且自然链缺蓝色忠诚资产、对抗缺灰色忠诚资产。改为实际吸血鬼 JC029 与合法资产输入，未放宽生产约束。
- native-03：6/6；包含完整自然链。后续补类别转换、重复付费、新局清除、历史加值与空牌库边界，最终统计见结果附件。
- integrated-04：51/51；首次 WASM 构建通过。随后只读核对庄园共用空白规则时发现资产/封印区遗漏排除，补最小生产限制和返回区域边界测试，05 为最终重建轮。没有声称静态发现已跑过红测。
- 当前目录断言精确增加 MSWM01；society.rs 同时补已存在的 MSJZ04，与实际注册顺序一致。其余目录、准入、卡图历史摘要和业务断言保留。

本轮只做相关集成回归；没有重复全库或旧 Worker 对局。Native/WASM、UI 组件、HTTP Worker/D1 冷恢复均不是真实浏览器试玩。

独立审查原拒绝、Library HTTP401、推送与发布限制仍有效，未重试或换路。无 main 合并、CI、部署、清理删除。证据与恢复 bundle 仅保存本环境。

最终 Native integrated-05：实际发现 89 项，89 通过、0 失败、0 ignored；含 10 项 MSWM01，不与前轮 6/51 相加。显式过滤旧 MSJC12 自然导出，未重跑旧对局；过滤后其余 2771 不属于本轮执行。Native 单线程，144.12 秒（其中测试 63.31 秒），子进程峰值 RSS 2,808,640 KiB，cgroup 采样约 8.49–11.20 GB。

最终默认 WASM 构建通过（202.18 秒），未启 society-fixtures。原生导出 4 个场景共 947 条指令，WASM 逐条状态/结果一致，3788 份四席视图、947 次原请求重复均一致。自然场景 seed=591，共 767 指令；准备授予/抓牌 158，对抗未授予/已授予各 11。原始 Native SHA256 `11d8fda581fedcbc312944c0a3e9de8e6d339d378fe8a3136a7c8cd26d034069`；WASM SHA256 `f1843085049901de3b725601dd3e0a6f9d49f0c1df92cf673edfe748b71f19ed`；完整源码 manifest SHA256 `bd74d7eb0995b3ab7e44b6d3a48614cdbb023abd46cb9a84f041fbd21f7899d2`。

Web build 通过；相关 UI 8 文件 144/144，含实际秘社行动按钮、原付费载荷、刷新视图、构筑逐牌过滤、原图/索引/历史双摘要及准入负例。没有更新依赖或锁文件。

最终 Worker HTTP/service/store/D1：4/4，0 fail/skip/cancel，333.99 秒。341 指令、1364 四席视图、341 次重复请求及 341 次冷重开；每场结束后再取首个收据。自然链只从原生真实挣得的授予付款前检查点开始执行 161 条，前 606 条仅 Native/WASM；准备链 158，对抗各 11。准确区分自然准备与显式单元布局。

复现使用 Rust/Cargo 1.90.0、Node 24.19.0、wasm-bindgen 0.2.104，`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/undergroundbattle-build`。完整命令、每轮终态/资源与三个锁文件 hash 见本地 evidence 的结果 JSON 和日志。最终执行：

```sh
MSWM01_NATIVE_OUT=/workspace/mswm01-validation/native-05.json node tools/build-storage.mjs -- cargo test --offline --locked -p hegemony-server --lib -- mswm01_ msjc12_ jc008_ society::tests control_tests batch4_manor_tests mystic_lifecycle_tests --skip msjc12_natural_fifty_card_deck_to_three_humans_paid_draw_room_witness --nocapture --test-threads=1
bash rust-game-wasm/build.sh
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/mswm01-validation/fixtures-05 node /workspace/mswm01-validation/verify-05.mjs
node tools/build-storage.mjs -- npm --prefix web run build
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/mswm01-validation/fixtures-05 node tools/build-storage.mjs -- npm --prefix web test -- MSWM01Integration Import20CardFaces ChoicePanel.test ChoiceRecovery deckLibrary.test Next20Presentation --maxWorkers=1
node tools/build-current-local-worker.mjs /workspace/UndergroundBattle/sites/node_modules /workspace/UndergroundBattle/rust-game-wasm/pkg /workspace/UndergroundBattle/sites/dist/mswm01-local-05
# From sites/, using the fresh same-filesystem local Worker output:
HEGEMONY_CANDIDATE_NATIVE_FIXTURES=/workspace/mswm01-validation/fixtures-05 CURRENT_CLOCK_WORKER=/workspace/UndergroundBattle/sites/dist/mswm01-local-05/worker.js node --test --test-concurrency=1 test/mswm01-integration.test.mjs
```

新导出/Worker 目录须为空，不覆盖既有证据。完整新增为一张封闭正式蓝色秘社，独审通过新增 0；无新未完成基础层。此前未决卡、被拒审查与未上传证据边界不变。本轮不声明完整 20 张批次就绪。
