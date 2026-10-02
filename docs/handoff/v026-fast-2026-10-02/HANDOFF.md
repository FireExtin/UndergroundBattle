# WIP：新 fast 线程安全移交

2026-10-02 UTC。接收线程 `01a0fe17-0f85-76eb-ab92-184bbb6890fd`。本分支未验收，禁止直接部署；所有派生 writer 已停止，root 在 checkpoint 推送完成后停止写入与发布。不清房间、不接管匿名席位、不停用户服务。

## 分支与线上真值

- 私有仓库 `https://github.com/FireExtin/UndergroundBattle.git`。
- 独立 WIP 分支 `codex/hegemony-v026-wip-fast-handoff-20261002`；基线 `f51c8723d0c44b207abb846da40b8593e57b7e2d`。精确 checkpoint SHA 由最终交接消息及此分支远端 HEAD 给出。
- 原主分支 `codex/hegemony-playable` 保持该基线。最后已部署实现为 `f8c6b44a1a56a1ec0f22e455cff6819a52e0238f`，不是本分支 WIP。
- Site projectId：`appgprj_6abf7bf54a7481918a50e1ef1509ca68`；当前 Site8，`rust-v0.2.5 / limited-v2.3 / hegemony-pdf-v1`。
- 线上精确 Site 源码：`2757673f19805e00b76cb2a01a891a9ebca76565`；savedVersion：`appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_309819e4023481919ccdbe0477566c10`。
- 部署 `appgdep_6abffe55cda08191ad2623ddbe1a7fc1`，`2026-10-02T18:56:41.439178Z` succeeded；[游戏入口](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)。公开 audience revision2 保持；不得旋转用户决定保留的 QA token。
- 已部署阅读切片：142前端测试/构建通过，公网两席同房v6状态哈希与实际阅读通过；过宽POST guard导致的首轮CF验证错误保留，收窄后的单次只读复看通过。详见 `docs/READABILITY_ACCEPTANCE_2026-10-02.md`。

## 停止状态与 fast 证据

| worker | 所有权 | 交接时状态 | 实际 service tier fast |
| --- | --- | --- | --- |
| `/root/rust_game` | 唯一Rust核心、cards、原生测试、oracle源文件 | stopped/completed，明确停止writer | 工具不暴露，未知 |
| `/root/react_game` | 7个附属UI文件和定向测试 | frozen/completed | 工具不暴露，未知 |
| `/root/cloud_validation` | 既有公开两席只读QA，下一四席计划 | completed；下一QA取消，未启动 | 工具不暴露，未知 |
| `/root/ui_hunters` | C26旧p2控制器 | completed，无writer | 未核实 |
| `/root/ui_reclaimers` | C26旧p3会话诊断 | completed，无writer | 未核实 |

主session fast由用户确认，但本工具没有读取/修改派生service tier的接口；model/reasoning配置与fast是不同维度。用户要求的 `gpt-6.1-sol / xhigh` 不能当作fast证据。新线程应使用父提供的显式fast配置，保持核心单一owner，不重复旧WIP。

过去约30分钟的可核验产物：冻结5内核提交 `6467dc3`、构建/隔离源保留5包提交 `f51c872`；下面的58项库测试、2项新增SQLite回归与65项UI定向检查；本分支完整代码。实际慢点是附属宿主/拥有者/控制者及关键词原PDF核准、原子状态/投影改动、首次夹具和测试helper修复；没有服务档位证据，不能把慢归因于fast未生效，也不能用降测试掩盖。

## WIP 内容及已知规则契约

Rust修改 `rust-game/src/{attachment.rs,catalog.rs,engine.rs,lib.rs,main.rs,model.rs,resolution.rs,rules.rs,world.rs}`、`rust-game/data/cards.json`、`rust-game/tests/attachment_service.rs`、`rust-game-wasm/examples/native_fixtures.rs`。WIP版本6/pool2.4，37定义（27玩家+10世界）。公路猎手以3BQ022替3JC125，仍50张、14中立，不是已完成的零中立牌组。两套50张主题还待其余机制，不提前开放研究候选。

BQ022原图：绿、1费、无忠诚，结附正面人类或吸血鬼，无本方/同地区限制，宿主永久战斗+1。Game.attachments为独立card/hostId记录，View.attachments实际JSON是扁平Card字段+hostId/region，不计入characters或独立地区图标。标准play复用支付/目标/帧。目标结算失效则本卡进入OWNER墓地且不退款。普通移动保留关系；翻暗视离场。

**回收已由《霸权说明书》PDF物理18/印刷P17核准为持续能力：宿主离场自动回附属OWNER手牌，替代入墓，不响应、不可拒绝。** 最初“你的手中→controller”推导已撤销；发出的裁定问题随后找到明确定义而作废，用户回复自动无响应与原文一致，不登记为新版本裁定。附件本身被消灭、宿主仅失去合法条件但未离场，不能伪触发回收。Room/Session源码未修改，五秒时钟合同保持。

UI7文件：`types.ts / CardTile.tsx / CardTile.test.tsx / Table.tsx / ReadModal.tsx / Attachments.test.tsx / attachments.css`。展示宿主附属数量与点阅、服务端有效/印刷值并列、最新投影解析与离场/换席/私密选择结束清理阅读，兼容旧view缺attachments。没有前端族别过滤、加1或BQ022牌号执行分支。精确测试说明在本目录 `attachment-ui-evidence.md`。

Sites4文件：`src/kernel.mjs`、`test/kernel-routing.test.mjs`、`test/integration.test.mjs`、`test/session-integration.test.mjs`。已准备六核路由、5的schema3/pacing保留、6注册37预期及双5/6时钟driver；仅JS语法/diff检查通过。**新6 WASM和 `sites/test/fixtures/prepared-response-v026.json` 尚未生成，当前无法称Sites6测试通过。**

## 测试和未完成

- 58库测试全绿：`native-suite-first.log`。此全套在新增HK服务helper错误处停止，既有集成测试尚未运行，不能称89项全绿。
- 首6附属定向4绿2夹具错误（JC016被误当屏障、JC063漏mode），之后已校正；`attachment-first.log`保留。
- 新SQL首次编译借用错误见`service-first.log`；随后1绿1helper错误见`service-second.log`。最终`attachment-service-green.log` 2/2绿：付费入栈/owner回收/原receipt/reopen/replay；实际DQJC116与BQ022四席私密承诺、逐步SQLite恢复/去重/同展。属于明确合成初态的本地原生规则检查，不是自然UI触发。
- UI六文件64 tests+typecheck绿，新增chooser-only私密阅读退场回归后Attachments8 tests绿，合计65个唯一针对性测试；未跑完整前端套件、未统一构建、未做6浏览器试玩。
- oracle源码新增 `--slice-v026` 四代表case和preparedResponse；最后原子修改保存，**未编译、导出或执行**。不启动旧巨型全案例导出来替代此有限片。
- 下一步顺序：审当前WIP及fmt → 尚未运行的既有native集成/pure/bin → 编译有限native oracle → 实际wasm32比较 → 从Rust导出的preparedResponse生成新6fixture（state保持opaque，严禁JS改privateState或替换版本串）→ 一次前端/Worker最终构建和必要15项Sites检查 → 新明确四席正常persistent QA（最低13、总预算40 UI POST）→反馈修复和验收后再发布。

## 工具链、官方bundle及本机保留路径

现成Rust工具链：`PATH=/workspace/.cloud-setup/cargo/bin:/workspace/.cloud-setup/wasm-bindgen/bin:$PATH`，`RUSTUP_HOME=/workspace/.cloud-setup/rustup`，`CARGO_HOME=/workspace/.cloud-setup/cargo`；wasm-bindgen0.2.104在后一bin目录。native target：`/workspace/.private-validation/hegemony-v026-target`，任务TMPDIR：`/workspace/.private-validation/hegemony-v026-temp`。无需重装；不要复用HOME变量或覆盖旧服务。

WIP开发bin（未final验收）：该target/debug的hegemony-server SHA `0493576fe1567d4b2cc61bb9f903c4419dfadf5cf32282828c81a8096736dab9`，hegemony-audit SHA `ebdb372df335f34b0956652b7bdd96fd975f770a314e2289d87f86f274da8809`。当前ignored `rust-game-wasm/pkg`仍为5/hash `22299f6db9923a93ea7da7bd742afbf0a0b1e4de4ff3c41bd99f16860df3a971`；同包已在tracked `legacy-v0.2.5`中冻结。

官方Sites bundle原始缓存 `/tmp/hegemony-sites-official-bundle/bundle.tar.gz`，解包 `/tmp/hegemony-sites-official-bundle/extracted`。为新机器读取，已将官方原包原样保存在本分支 `tools/handoff/sites-official-bundle-20261002.tar.gz`（348,088 bytes，SHA `7bfc90a2613fc215ee6b8f394030c8fafc256f132df24d8806f04dda4e24f72f`）；无需改脚本。解包后 `scripts/site-workflow.mjs` 接隐藏stdin JSON，`scripts/build-site.mjs`运行项目build。指导来源为已安装Sites hosting skill，canonical `skill://plugin_connector_1p_689987207de08191979cf68eca2941c6/sites-hosting/SKILL.md`。新发布需工具新取短期credential；不保存或输出token。

独立Site checkout `/workspace/hegemony-sites-deploy` 仍是已推送的Site8 source2757673，没有staging本WIP。唯一期末推送者由新线程负责：主源码先commit，再`node sites/scripts/stage-source.mjs <独立checkout>`，用官方流程检查/推精确Site HEAD/打包，然后save→deploy→terminal status。保持projectId/audience/原迁移/全部旧核。

已验证8103/8104/8105仍监听；未启动8106。运行DB、dev bins、完整source原图放大缓存、浏览器profiles/会话及CDP文件没有推Git（含敏感身份，不能为handoff上传）；本地原地保留。所有源码WIP、5条失败/成功测试日志和UI检查说明已提交本分支，没有不可推送的源码路径。若新机器不能共享这些本地会话，先报告边界，不能伪造旧身份。

原C26 `3355ede19422f27a2ce1e547` p1/p2 controller保留，p3匿名context已丢失，无正常恢复凭据；不清旧房。现公开QA房`4e832f0868c0699c4ff10726` v6两OWN profiles保留；native5四席QA`da034a524f221895f2379379` v12四persistent profiles保留。新6QA尚未创建，计划路径`/workspace/scratch/hg-v026-native-four-seat-20261002/profile-p0..p3`仅计划，不能当已保存会话。
