# Next20 固定源码验证与父端审查证据（2026-10-25）

四卡与 BQ009 均保持 `supported:false`。目录为 277 张，公开准入仍是 233 张，44 张禁用。本轮没有增加供给卡，没有修改生产规则、原桌、站点、部署或 32 个交付输入。

完整验证固定在 `1bd07762824e138c0560863130ec7eacaddbf9c8`；当时主线文档 HEAD 为 `98906bbd207dc37617de7c8bb0000c170cbfc0c5`。所有构建在 task-7 的独立 Git archive 中运行，编译、临时数据库与缓存也留在 task-7。此后只有测试改动：`ed6d72a0aa4bb8c28be8596e4869233b78a77880` 更新目录和准入断言，`927e18191c07d1575470a9995b7ec4a53933d112` 修正新增测试的借用编译错误，`e853d8df54740a4c6633faddf091c6761a391afb` 更新 Worker 目录断言。没有重写提交历史。

## 实际结果

| 固定源码上的命令/范围 | 结果 |
| --- | --- |
| `cargo test --offline --locked --workspace --no-fail-fast -- --test-threads=1` | 2629 通过，2 失败。库测试 2557/2，集成测试 72/0。 |
| `bash rust-game-wasm/build.sh` | 通过；WASM 8,651,037 字节，SHA256 `c97f177335f0afb94e154886006c91b2a35d2d2e5113e76a4630d0b0ab8e6a02`。 |
| `npm --prefix sites run build` | 通过；含两套 tsc、Vite 及 Wrangler dry-run。没有部署。 |
| `native_fixtures` 新生成数据，再用 `compare.mjs` 执行 WASM | 483 场景、25,135 状态转换、99,903 视图比较全部一致；362 个拒绝命令、42 个拒绝建组结果一致。 |
| `npm --prefix web test -- --maxWorkers=2` | 159 文件中 121 通过、38 失败；已收集测试 1156 通过、4 失败。35 文件因旧夹具绑定而未收集，另 3 文件有 4 个目录断言失败。 |
| 全部 `sites/test/*.test.mjs`，真实本地 workerd/WASM/D1 | Node 报告 91 项，40 通过、51 失败；失败为 39 个整包指纹绑定、3 个目录源码绑定、9 个目录数量断言。Node 数字包含导入失败文件，不代表 91 个完成的玩法场景。 |

Native 两个失败都是精确枚举陈旧：死亡能力目录新增 XG25，梦境目录新增 BQ033。它们在旧 `ff3b1c03f4cc4e17ae740a61b1dc34c8c86a0210` 上实际各跑 1 项并通过。修正保留完整精确枚举和原断言，还用实际 JZ27 付费现身命令验证 XQ12、XG25 都不能被视为人类。

只重验受影响部分：`927e1819` 的两项 Native 各 1/0；`ed6d72a0` 的三个 UI 文件 34/0；`e853d8df` 的八个 Worker 文件 9/0。后者包含真实建桌、准备、开局、D1 重开、并发/收据等原断言。受限执行时测试子进程提前退出的两份诊断也保留；获准的同一套本地测试实际完成后 9/0。**没有把修正前的完整运行改标为新 SHA 全量通过，也没有重复完整套件。**

旧夹具保持原样。其记录指纹为 `b527bf3706d1a5a2970bf5b6417bb0a04b4924bd590e58e2fabd6cc07cee638e`，固定源码本次构建的指纹为 `09ac47dd46dfd5582c1a99cd7d0f1d7561385c87f780c000014e79c949140e2c`。用旧 ff checkpoint 真实重建 WASM，再加载一个原整包夹具，也发生指纹不匹配（旧候选指纹 `bdb004899dd74da289cc278177b960fee97dec4657f93f120ad09a1c47fcd1f0`）。这证明存在既有绑定不兼容，不能推出旧完整 UI/Worker 全通过或全失败；三项目录源码绑定在新增定义后也需要重新生成对应的真实 Native 导出。未重签旧状态或替换原始输入。

## 两个审查问题与 BQ079 核查

付费栈卡会分配新实例，不能要求 `item.card.id == frame.source.card.id`。原公共校验没有替蓝卡这一入口保证实例对应，d0b3e325 已改为复用 `pack_a_transaction_matches`：完整重置后快照相等，并要求 `source < transaction`、`transaction + 1 == frame`。真实红测 `next20_blue_core_review_deploy_rejects_transaction_instance_swap`、`next20_blue_core_review_reveal_rejects_transaction_instance_swap` 用退役来源 ID 篡改栈卡，在修复前失败、修复后通过；完整 Native 本次也覆盖通过。

BQ034 空目标、无饮血选项只清除属于自身 frame 的 `post_frame`；共享 `mystic_after_next` 原先已排队的单个 `MysticContinue` 继续推进。`next20_blue_core_review_no_target_continues_later_shown_exactly_once` 与 `next20_blue_core_review_no_blood_continues_later_shown_exactly_once` 验证后续 WM061 只磨 1 张、历史唯一、无残留续体，并拒绝缺失/重复续体和重复选择。两项也在本次完整 Native 中通过。

`commit_owner_grave(c, None)` 的确意味着：当牌名是“无名尸体”时，重新捕获当前 BQ079；传 `Some(frozen)` 时使用之前的捕获。死亡批次原先在任一移除前冻结 BQ079。当前延迟入口和恢复校验仅准入不可变定义 XG25（伥鬼），它不会满足“无名尸体”判断；因此它的延迟回退在 Some/None 两种情况下都直接入墓、不分配 BQ079 观察事件，不会纳入后来观察者或丢失同批次观察者。本次未发现可达语义缺陷，未改生产代码。完整 Native 中 `batch3_bq079_simultaneous_sources_and_corpse_capture_before_either_removal_order` 和 `batch3_bq079_normal_noncorpse_arrival_does_not_allocate_observation_or_duplicate_identity` 等 12 项 BQ079 测试均通过。若将延迟机制扩至尸体定义，需要另核其捕获时点，不能把当前结论推广。

审查原码在 `evidence/next20-fixed-validation-20261025/production-methods/index.json`，29 个方法分为不超过 85 行源码的片段，覆盖 XG25 死亡响应、失败入墓、原调用方恢复、CQ25 检索、恢复校验、实例匹配和共享续体。来源均为固定 1bd Git 对象；`core-test-names.json` 给出 52 项定向用例名称及位置。这 52 项是 45 个预置布局命令流程、3 个目录/建组门禁、4 个辅助接口契约，不能替代整卡准入。

## 接续边界

两项用户裁定已落实，不再待答：封印 XG25 由拥有者发动、付费、记次数；CQ25 来源离场仍检索/洗牌，载体无效时找到牌进入拥有者墓地。XG25 隐秘条件在结算时判断仍标明为卡文推导。

BQ009 合法手牌/牌库到自身结社的供给入口尚无实现；BQ033 的实付资产、真实锚点和收据已有 Native 证据，但预置封印状态不是完整可达玩法。本轮不加供给卡。四卡完整 RoomEnvelope/WASM/Worker/UI 生命周期、真实双席浏览器试玩、所有多 XG25/淘汰终局/授予能力续体组合及父端独立 review 均未完成。JZ35/XG25 宿主场景在当前类型规则下不可达，没有伪造宿主类型。共享 483 场景回放通过不等于这四卡完整验收。

远端四条引用本轮只读核实均未变化，详见 evidence 中 remote JSON。连接器以 URL allowlist/INVALID_ARGUMENT 拒绝 `GET actions/workflows`，该读取已停止，未换渠道。08:19 的旧快照仍有 active 动态 Copilot workflow，但当前触发条件无法确认，因此不推送、不改 workflow/安全设置、不执行 GitHub CI、不合 main、不发布。

原始日志、32 输入、完整新回放 payload 与测试数据库留在 task-7 独立验证目录。仓库仅归档约 1.4 MB 文本/日志证据和少量压缩原始日志，不归档数据库、构建产物、缓存或凭据。可读日志副本去除行尾空白；`validation-summary.json` 同时记录原始与归档 SHA256，存在差异的原始日志另存 `.raw.gz`。所有失败、临时编译错误和启动诊断均保留。
