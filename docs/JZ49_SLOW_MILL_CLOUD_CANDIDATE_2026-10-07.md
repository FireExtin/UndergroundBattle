# JZ49 迟缓与进场置墓云端待审候选

本批从 main `b3791b2f7f7ac09224db60a6d87ab5458b4c7273` 独立建立，不包含待审 engine37–42 的 JC094/XQ27/XQ42/BQ040/WM059/蓝色印刷元数据批次。父线程为本批保留 `rust-v0.2.43-jz49-slow-mill-candidate` / `limited-v2.40-jz49-slow-mill-candidate`；编号跳跃只避免候选身份冲突，不表示其他批已合并。当前普通卡101（其中10地区）、秘社8、既有预组5。

## 原图与规则

实际查看云端已有 `resource/ymsj-fun.github.io/cards/JZ49 蹒跚行尸.jpg`，SHA256 `208b2370433fb420ba91e1557ad6633589e43b1391d2008d63ebee2ed90fdf62`，编号49/76。与人工完整规格 `docs/factions/card-specifications.json.cards.JZ49` 一致：费用1、黑忠诚2、死亡领域1、角色·不死生物/行尸、防御1、永久战斗1及临时战斗1，非独有。

实际查看《间奏说明书》原图，SHA256 `efbe7e1e3711ca4b184b36f7801391afdec3b195d393cf17d2458215279f9b80`：迟缓的角色或附属进场后必须横置。实际查看《隐秘世界规则手册》印刷P2/PDF4（文件SHA256 `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`）：进场触发包含现身，普通触发可选择、入堆叠且可响应，重置步骤重置场上牌。JZ49原图提醒文字为“该角色横置进场”；能力为进场触发将目标玩家牌库顶的两张牌置于其墓地，没有敌方或本地区目标限定。

## 有限实现

在既有 `enter_triggers` 入口应用 `Traits.slow`，在冻结入场声明前横置该正面实例。没有增加效果、队列、身份或绑定框架。完整定义校验目前只接受JZ49使用迟缓；规则页提及附属不表示本批开放其他迟缓牌。

置墓复用XQ38已有 `MoveDeckTopToGraveyard(PlayerRef::Target(0), count)`，JZ49固定为2。完整JZ49定义及能力均限定原图程序，拒绝改事件、数量、目标、费用、次数或移植该定义。沿用既有可选声明、玩家目标、付款、堆叠响应、拥有者墓地归属和短牌库处理。

正面派遣、付费/免费现身、既有地区效果的正面墓地进场均迟缓；秘密派遣、墓地暗藏者入场及建立资产不执行正面角色关键词。地区移动只发EnterRegion，保持已有横置状态；离场后重新进场再次迟缓；下一回合正常重置。

## 验证与保存

最终单次干净native工作区全套526项通过（494核心、32集成），0失败、0忽略。

17项聚焦native覆盖四席所有自身/队友/敌方目标组合、可选放弃、入场前横置、临时图标与对抗参与、所有现有进场路径、隐藏隐私、移动/重新进场/重置、忠诚与费用原子拒绝、短/空牌库、控制者/拥有者、来源死亡后原效果、目标玩家淘汰、完整定义拒绝和SQLite关闭重开后原回执/冲突/不重复效果。原生初始布局均明确为离线夹具，后续使用正式命令。

实际WASM对照290转换（其中3拒绝）、601保存检查点、13步连续合法付费响应链，共3616四席投影，与native逐项完全一致。响应链包含普通选择→双方让过→BeginResponse→SubmitResponse→JC102消灭来源→原JZ49置墓继续结算；每步事件重放和持久态恢复一致。

相关UI聚焦4文件36项通过；Web全套63文件519项通过；TypeScript/Vite构建通过。JZ49新增UI测试核查原生投影恢复、四玩家目标选项、跳过选择、横置可见性、隐藏身份和50张构筑/同名上限。旧JZ55 UI规则夹具绑定字节相同的冻结v36。

Worker/D1全套25项通过、0跳过。旧193个历史文件逐个git blob及SHA256保持一致，新增冻结v36五文件逐字节等于已审ABI，WASM SHA256 `0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036`。current-only Worker只同步现有固定tuple，不增加历史WASM导入；实际构建只有一个WASM模块。新WASM2,186,939字节，SHA256 `3b0bee7b1c6788bc0a299338322f1c07d6108161a75e4999ac066971659520bb`；WorkerJS47,255字节，共2,234,194字节。

复现工具为环境已有Rust1.90、wasm-bindgen0.2.104及本地npm依赖，无新包安装。运行 `cargo test --workspace --locked --offline`；设置 `GREEN_EVIDENCE_DIR` 与 `JZ49_CHAIN_DIR` 后运行 `cargo test -p hegemony-server --lib jz49 -- --test-threads=1` 导出离线夹具；构建 `bash rust-game-wasm/build.sh` 后执行 `node rust-game-wasm/tests/jz49_compare.mjs <证据根目录>`；Web运行 `npm test`/`npm run build`；Sites运行 `npm run build` 后 `node --test --test-concurrency=1 test/*.test.mjs`（仅dry-run）。

卡面另由独立候选 `ab25c7393155f9aeba931919de08f5056f33b842` 提供，尚未混入本分支。父集成两批后须核查扫描资源和实际浏览器操作；本批UI夹具/ABI测试不能替代自然组牌、对抗和完整终局策略试玩。研究目录原有`notAccepted`状态未升级。本批未推main、未发布；当前Site34保持原已发布版本。
