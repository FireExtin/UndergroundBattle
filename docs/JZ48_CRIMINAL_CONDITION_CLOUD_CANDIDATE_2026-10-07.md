# JZ48 街头劫匪有限机制候选

本批从已发布 Site35 的已审源码 `2305aa17c53e73dd3683ae9fe1e654531777f16e` 建立隔离分支，保留 JZ49，不从旧 main 开发。候选身份为 `rust-v0.2.44-jz48-criminal-condition-candidate` / `limited-v2.41-jz48-criminal-condition-candidate`。普通卡102（其中10地区），秘社8，既有预组5；只新增 JZ48，不包含待审37–42其他批次。卡面另由独立批次处理，本批不改原图、映射或阅读交互。

## 原图与规则

实际查看云端完整原图 `resource/ymsj-fun.github.io/cards/JZ48 街头劫匪.jpg`，SHA256 `a9a5da75938c22872cca524776356b10f2a0250370d3ba3ed3b31deb332e5b4f`，与 `docs/factions/card-specifications.json.cards.JZ48` 完整规格核对：费用1，黑忠诚1，无魔法领域，角色·人类/罪犯，非独有，永久战斗1、无临时图标，防御1。持续能力只在本地区存在其他本方罪犯角色时，让这一实例获得永久势力1和防御+1。

实际查看原始《霸权说明书》印刷P16、P21（PDF物理17、22），再次确认“本方”是操控者本人，友方才包含队友。同队罪犯不能单独满足本条件；正面角色的类别按已有 `current_subtypes` 查询，横置不消除类别。

## 最小实现

现有 `ConditionalIcons` 仅有资产领域/地区势力条件，且只改图标；`OtherFriendlyCharactersDefense` 对其他友方角色给防御，并包含队友，不能直接复用本牌。新增唯一无参数 `JZ48OtherControlledCriminalInfluenceAndDefense`，完整 `Definition` 白名单拒绝移植、删改、叠加及附加能力或关键词。

同一个实时查询用于永久势力与防御：来源必须是当前地区内、正面的 JZ48；另一实例必须不同ID、正面角色、当前controller与来源严格相等，且具有罪犯类别。资产、隐藏牌、其他地区、队友与敌方均不满足。多个罪犯只加1；两个本人JZ48可以互相满足。owner不参与条件，墓地仍归owner。未新增通用绑定、持续效果、身份或队列机制。

永久势力与先手无关；来源横置时仍有有效属性和防御，但沿用现有 `icons` 对抗参与规则，不贡献图标。移动、翻暗、离场及控制变化立即改变属性。沿用既有 `settle_deaths`：先在当前场面冻结整个同时致死集合的来源快照，再逐实例离场；防御条件失效后的新致死集合在下一次循环处理。没有给JZ48额外死亡触发或另设清理顺序。

## 验证

最终native工作区545项通过（513核心、32集成），0失败、0忽略。19项聚焦覆盖四席控制者/地区矩阵、自己排除、横置、隐藏/资产/非罪犯、双方同名独立实例、跨owner、源/支撑移动及控制变化、离场/到期级联、同时死亡原owner归属、真实付费派遣与忠诚/费用原子拒绝、XQ16可选进场翻暗声明与堆栈、妖火付费响应链、精确地区阈值及队伍10分。正常翻暗换实例并清伤，隐藏本身不按正面致死；cleanup先清伤再到期的原子步骤保留，创伤导致的防御归零另行核对。SQLite每步关闭重开后回执相同，重复命令不再次付款/致死，命令ID冲突拒绝，最终状态和回执数量核对。

实际WASM对照59转换（8拒绝）、282保存检查点、13步连续付费响应链、1416四席投影均与native一致。连续链逐步核对事件重放与保存恢复；上层妖火导致两张互相支撑的劫匪级联死亡，原下层目标消失后不会再次造成伤害。冻结engine43的原JZ49对照另有290转换、601检查点、13步链、3616投影通过；当前44的native全套也执行原JZ49测试。

Web65文件537项通过，0失败/待定；新增8项核查102目录和保留101旧定义、实际构筑、投影恢复、当前/印刷图标与防御区分、横置、隐藏隐私及跨owner实例。TypeScript/Vite构建通过。Worker/D1全套25项通过，0跳过，current-only路由拒绝39历史tuple与改变的身份，构建仅一个WASM模块。显式opt-in的 `society-foundation.fixture.mjs` 只同步未来44/41 fixture身份，本轮未运行该非生产路径。

旧199个历史文件逐个与基线git blob字节相同，新增冻结43五文件逐字节等于已发布ABI。冻结43 WASM 2,186,939B，SHA256 `3b0bee7b1c6788bc0a299338322f1c07d6108161a75e4999ac066971659520bb`。44 WASM 2,191,175B，SHA256 `6cf6dca6a06844c84e38fbed2a084145bc0e63ae3897de18b864814c314d11f9`；Worker JS 47,273B，总2,238,448B。既有JZ49界面夹具改绑冻结43，不跨版本解释旧状态。

## 失败记录、复现与边界

初轮新测试字段/辅助方法名写错，改为现有 `threshold` 与印刷计分求和；夹具误给妖火1资产（实际费用2）、误把派遣后的实例当作原手牌ID、以及把无能力JC084当作新增能力变异，均在测试中纠正。全套首次可运行结果为511通过、2旧计数断言失败：非地区91→92、黑色非地区14→15；未放宽规则断言。Worker初轮current版本预期仍43/40，已同步44/41。

本地编译曾因 `/tmp` ENOSPC 无法链接与写夹具，保留失败日志。只删除本轮不完整、可重生的夹具及失败对象；根清理了已完成旧编译缓存，源码、ABI、审查材料与数据库保留。最终native使用单job、`CARGO_INCREMENTAL=0`、`CARGO_PROFILE_DEV_DEBUG=0`、`CARGO_PROFILE_TEST_DEBUG=0`、对应 `STRIP=debuginfo`，规则源未因空间改变。默认并发Web首次exit1且没有完整报告，原因未确立；单worker完整重跑537通过。

复现：已有Rust1.90/wasm-bindgen0.2.104与npm依赖。运行上述环境配置的 `cargo test --workspace --locked --offline --jobs 1`；设置 `GREEN_EVIDENCE_DIR`、`JZ48_FRONTEND_DIR`、`JZ48_CHAIN_DIR` 后运行聚焦 `cargo test -p hegemony-server --lib jz48 --locked --offline -- --test-threads=1`；`bash rust-game-wasm/build.sh`，再运行 `node rust-game-wasm/tests/jz48_compare.mjs <证据目录>`；Web `npm test -- --maxWorkers=1 --no-file-parallelism` / `npm run build`；Sites先 `npm run build`（dry-run）再 `node --test --test-concurrency=1 test/*.test.mjs`。

脱敏机器记录见 [test-summary.json](evidence/jz48-local-2026-10-07/test-summary.json)、[WASM对照](evidence/jz48-local-2026-10-07/wasm-compare.json)、[历史/Worker核验](evidence/jz48-local-2026-10-07/history-and-worker-proof.json)。本批初态和界面投影是明确的离线规则夹具，不能替代engine44自然组牌与完整策略终局试玩；没有访问生产房间或修改运行数据库，没有推送、发布、合并main或Library上传。研究目录的 `notAccepted` 状态保持，等待新的独立审查。
