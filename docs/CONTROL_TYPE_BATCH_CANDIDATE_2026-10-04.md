# JC129 / JC036 / JZ27 控制与类别候选批（2026-10-04）

本批仅实现父线程已批准的三张牌及共用有限生命周期。基线为 `8cb96a06a45bfe32d3c685f26b3e66db79b6f66c`，已审生产实现为 `c4d20b599be77a3e09a4bd8dc3e5e0bd272e0620`。候选分支 `codex/hegemony-control-type-batch-candidate-20261004`。没有 push、Sites save/deploy、预览访问或外部房间操作。

版本为 `rust-v0.2.15-control-type-batch-candidate` / `limited-v2.12-control-type-batch-candidate` / `hegemony-pdf-v1`。普通 registry 56 项（46 种玩家普通牌与原 10 地区）；秘社仍为 MSJC09 / MSJC01，扫描 58。此前 53 项 registry 原记录和 55 项扫描元数据逐项不变；原 14 核的二进制原样保留在 `legacy-v0.2.14`，不改 Sites 路由。

## 原图、原文和项目解释的边界

三张原 JPEG 已实际查看；新增扫描是逐字节复制，SHA-256 与 [原图研究记录](CONTROL_TRANSFER_NEXT_BATCH_SOURCE_2026-10-04.md) 一致。JC129 费用3、紫忠诚1、中立心灵事务；JC036 费用5、蓝忠诚2、状态/吸血鬼附属；JZ27 费用6、蓝忠诚3、吸血鬼/法师，防御3，普通调查1/战斗2，白底先手势力1，副标题“猩红凝视”。JZ27 为金名独有，不是印刷“唯一”；通常同名3张构筑。

FAQ 物理页3左上两条明确：孤立初拥消灭后，宿主归拥有者；宿主潜伏使初拥消灭，暗藏者归拥有者。霸权说明书物理页16（印刷15）明确 owner 不变、controller 可变，“你的/本方”按当前 controller，自称仅指该张牌。交付附这两个原 PDF 直接渲染的完整小图及源 PDF/输出 SHA，没有重绘原文。

**以下是父批准的有限项目解释，不冒充旧 FAQ 结论**：同一目标多个已结算控制来源按发生顺序排列，最后仍有效者取得控制；上层失效恢复仍有效下层；无下层时恢复原合法基准 controller（可能不同于 owner）。JZ27 历史效果的受益者固定为结算 actor，来源后续易主不送出已有效果；原来源 instance 离场或潜伏结束效果。目标潜伏/离场产生新 instance 时不继承旧控制或类别效果。孤立初拥潜伏路径回 owner 与 FAQ 相符。

## 具体实现

`control.rs` 只维护可持久化的逐目标基准与有限效果列表，期限仅为 TurnEnd / Attached / SourceLeaves，类别仅为无变更 / 人类转吸血鬼 / 额外奴仆；继续使用既有目标守卫、付款、响应帧、附属、离场、独有处理和重放。没有新增任意规则框架。

JC129 仅标准行动取得公开敌方角色控制，到本回合原子清理失效；事务正常入墓不结束已结算效果。JC036 结附及持续条件检查**印刷人类**，持续以附属当前 controller 控制宿主，将当前人类改吸血鬼并保留其他类别，不因自身转换而自毁。JZ27 仅付费现身触发，目标为来源地区的**当前人类**，非任意进场触发；控制与额外奴仆绑原来源 instance。

控制更换保持目标 instance、owner、伤害/横置状态与 JC008 实例修正。真实机动同样保持 instance；身份重置在 fresh 前恢复原基准并清理旧效果。控制变更排入既有独有冲突队列。附属 owner/controller 不随宿主更换。LC01 顶牌权随当前 controller，且只看其自己的牌库顶，队友不分享；每个默认 oracle 步骤对每个席位独立断言顶牌 instance 正确。

前端读取权威 `currentSubtypes`，显示当前类别并保留原图阅读；授权按 controller，未授权暗藏视图擦除类别及印刷数据。仅新增 JZ27 的普通角色副标题；旧 53 项普通卡对外字段保持兼容。

## 验证与覆盖边界

新增 16 项原生定向测试覆盖付款/敌方与公开目标、响应后守卫、JC008/LC01 四席权限、三层不同期限逐层恢复、被上层遮盖时下层先移除、附属 ownership 与当前 controller、基准不同于 owner、真实潜伏/现身新 instance、来源真实快速响应回手、当前人类过滤、仅 Reveal 触发、独有冲突、真实机动，以及伤害/属性/临时控制原子清理。布局/资金、合法基准及附属 controller 变更的少数原语检查明确标为 fixture，不声称自然出牌实现了未准入机制。

默认 WASM 新增六个场景，**不依赖额外 slice 参数**：五个边界场景只在初始布局/资金设定后走真实已准入牌和正常命令；另一个固定种子5的四席游戏从实际50张构筑、准备、开始、正常建资产、检索与付款出牌一路走到第13回合。自然场景在四席可派遣的中央地区完成 JC008 快速响应 JC129、LC01 被夺控、初拥、哈达莎秘密派遣/付费现身、两次律师进场潜伏。未注入状态或绕过付款。

每一步持久化后重新载入，原生重放每个真实 transition journal；真实 WASM 比对完整 opaque state、完整 transition 和每个席位投影。带 transition 的步骤使用 `step.transition.state/view`，保留每一预期字节与全部 views，减少重复外层副本。默认生成器输出 `external-cases-v1` 索引和同名 `.cases` 目录，每个 case 独立完整 JSON；比较器逐 case 读取，兼容旧单文件和定向生成器输出，`verify.sh` 同时清理临时索引和目录。完整默认原文件超过 Node 单字符串容量，按场景分文件是容量处理，不是抽样或删除步骤。

原生批末全量 204 项（172核心+32集成）、前端35文件319项、正式前端构建通过。首次并行检查因工作盘耗尽而链接中断、一个旧前端测试超时；只清理可再生成 incremental 缓存，降低并发后通过。随后限制旧普通牌副标题的微调另做2项聚焦 catalog 与21项扫描/投影复核。最终默认真实 WASM 117场景、5,964转换、23,351席位投影、70拒绝命令、4拒绝构筑和1次报价全部一致。二进制1,805,346字节，SHA-256 `a32fce15c7acac49793e6e3bab6a3c10caefbcdba00ffa5c51e7906a9d26d460`。完整默认结果已本地保留；交付摘要与各场景完整字节哈希在 `validation-summary.json` / `default-case-manifest.json`。

自动化原生/WASM 场景证明正常命令、持久化与投影；**不是自然浏览器 UI 验收**。未访问已被安全策略拒绝的预览地址，未尝试其他 host/工具绕过，未访问父或朋友房间。旧核兼容仅本地新 lobby 的只读恢复/版本拒绝及二进制哈希，不宣称旧公网房间已验。

## 本地复核

使用 Rust1.90、wasm-bindgen0.2.104、Node24。先按环境设置 CARGO_HOME / RUSTUP_HOME / CARGO_TARGET_DIR / WASM_BINDGEN，再运行：

```bash
cargo test --locked --offline -j 2 -p hegemony-server
bash rust-game-wasm/build.sh
cargo run --release --locked --offline -p hegemony-wasm --example native_fixtures -- /tmp/control-default-native-oracle.json
node rust-game-wasm/tests/compare.mjs /tmp/control-default-native-oracle.json
cd web
npm test -- --maxWorkers=2
npm run build
```

定向复核使用 `control_tests` 或生成器 `--slice-control-batch`，只是调试便利；本批新增场景已进入默认生成器。轻量交付只包含本批六个完整新增 oracle 场景、默认全量摘要/完整索引/每场景哈希、当前新 WASM、源码 bundle/补丁、聚焦原图/原文小图和完整测试日志。旧111场景完整默认输出本地保留，可按上述默认生成器重现，不在轻量包里重打包旧机制档案；包内定向6场景的通过不能替代默认117场景日志。
