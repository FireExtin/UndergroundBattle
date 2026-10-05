# 蓝色最低切片 Air 待审候选

本地候选在 macOS 26.3.1 arm64 的独立工作树完成；尚待父会话独立审查。输入是已核验交接包、已审 `713be787f47d4188640922a5d373662bd04ffe38`（tree `733361195cf87534ab0546b809512990255c6cdd`）和其上未审 `7c6b016a8b09f607ed21a4f2d5822cb77cc9e553`。原已审工作树保持干净，原7c6分支保留。输出为 source33/pool30、96普通卡+8会社、104原图扫描；没有 push、部署、改 main 或重启业务服务。

## 输入真相源与项目规则

已直接查看三张整卡原图，源文件都在 `resource/ymsj-fun.github.io/cards/`，公开原图副本在 `web/public/cards/`；manifest 生成器新增 MSJC03，旧101条原图记录及实际字节逐条保持一致。

| 卡牌 | SHA256 | 核对范围 |
| --- | --- | --- |
| JC032 血族长老 | `ebeb0a13c62cdf10569f4aa9f300998aff53a65788d756f055e9696f35489b4b` | 费用5、蓝2、死亡1、吸血鬼；动作2、每回合一次、无横置费用；顶6印刷吸血鬼角色展示后隐匿，余牌洗回 |
| JZ24 卡迪纳追迹人 | `e01995d70e06a6b179dcbbf81c3df73ab548269fc0efe9fafc33e2e2f3c8722d` | 费用4、蓝1、死亡1、吸血鬼/罪犯；可选 Reveal，手牌≤3敌席各牺牲本地区角色 |
| MSJC03 法师结社 | `07463a18a5cc6726f098bedc10304ef6c4032929cff72535d5f9227338e45f29` | 蓝牌最低25、起手6；动作3横置且有先手时抽牌；动作4横置找蓝色独有卡，每局一次 |

`霸权说明书.pdf` 物理第11页是印刷P10，已渲染并实际阅读：行动能力发动后与来源分离；触发能力可被响应、无强制字样时可选，结算流程与行动能力相同。证据包保留该原始页渲染。扫描PDF没有可抽取正文，空的 pdftotext 输出不作为依据。

原地区实例失效策略、JC032非空无匹配时私有零项确认、JZ24首次执行冻结敌席是已认可的项目规则，不是官方牌专属 FAQ。规则不按同名同色识别实例，费用、actor、owner/controller及声明时原地区实例各自保留。历史测试设计误把 JC125 人类列为吸血鬼，执行前换为实际印刷吸血鬼 JZ24，没有改变旧 JC125 定义。

## 实现和审查重点

`blue_minimum.rs` 保留两个固定操作和最小可序列化续体。JC032首次执行固定顶≤6实例，有命中强制1项；非空无命中只能查看并确认0项，空库结束。选择先严格核对地区和前缀；展示后创建隐匿新实例，owner保留、controller为actor，不触发 Enter/Reveal/Raid，剩余整个牌库只洗一次。JZ24响应后首次执行冻结敌席；逐席以当前本地正面角色/controller生成选项，前席牺牲/BQ022返还后仍处理已冻结的下一席，续体先于后续Death声明。MSJC03搜索蓝色独有**卡**，不缩为角色；9个蓝牌名每名最多3张可构成27张蓝牌，独有不等于牌库只能1张。

父会话应独立检查以下真实差异：

- 完整三牌定义相等校验以及固定操作直接移植限制；原7c6的通用 per_turn_limit gate 会在注册JC032时失败，现只额外接纳精确JC032动作，保留旧host grant限制。
- `ChoicePanel` 只对 `jc032_top_six` 且 min=max=0、options为空开放确认，不改变其他必须选择的空选规则。
- 原7c6的固定操作移植校验没有遍历 mode；红测试重现后增加 mode操作拒绝，30个形状变种、4个直接移植和2个mode移植均拒绝。
- 实际 Store/SQLite 在重启后重复command_id返回原回执；RoomEnvelope本身不储存回执。借控和隐匿权限按实例、owner/controller，不按颜色。

Air现有Bash3.2在 nounset 下空数组展开导致构建脚本失败，已改为兼容展开；Mac不区分大小写时 `DeckLibrary.tsx` 与 `deckLibrary.ts` 冲突，组件文件改名 `DeckLibraryPanel.tsx`，出口和行为不变。两项原始失败日志均保留。

## 实际核验与证据边界

证据是显式离线布局走生产命令协议和实际WASM，不能称为自然公网对局。`native-associated-union.log`：15个blue模块测试+1个准入变种测试，16通过；增加mode变种后只重跑同一准入测试，1通过。原始失败、夹具误差和定向修复均保留，没有掩去失败后重跑整套。

前端关联11个文件的97项实际执行：首轮96通过、1项旧动态蓝牌数量期待失败；修正期待27蓝/11中立后，`red/蓝`过滤实际运行2项、3跳过，两项通过。不能将此表述为一次全绿97。BlueMinimumScan初次7/8通过，唯一失败为 absent pending实际是null、夹具断言期待undefined；只重跑失败标题后通过。该文件随后包含在97项关联执行中。ChoicePanel零项确认先红后绿，11项通过。类型检查及Vite构建通过。

最新模式防护后的默认WASM33已重新构建；`wasm33-final-parity.json`逐条核对完整正文和四视图：488条命令、983个恢复点、47次状态不变拒绝、6116个完整席位视图、11份前端整状态夹具，并检查两种真实回执/重启证人。旧94定义、7会社、5预组、101原图和134个冻结ABI文件字节一致。当前33拒绝旧状态身份，冻结32读取旧身份，未迁移旧房。

| 产物 | SHA256 |
| --- | --- |
| 最新33 `pkg/hegemony_wasm_bg.wasm`，2,165,572 bytes | `2bdf206c232747045c481c47d3493be18bec9fea263293866ca4d95bf824c2bb` |
| 冻结32 `legacy-v0.2.32/hegemony_wasm_bg.wasm` | `fd375f5e05247c4e78b5cbd65f3da2e4393a729d49b4b2203347dd6d0e62fc89` |

复现环境：固定 Rust/Cargo1.90.0、wasm32-unknown-unknown、wasm-bindgen-cli0.2.104、Node24.5.0/npm11.5.1；工具/缓存置于任务隔离目录。jobs=2、incremental=0、dev/test/release debug=0。没有工具链升级混入本逻辑提交；nightly比较独立。

```sh
GREEN_EVIDENCE_DIR=/absolute/evidence/native-bodies BLUE_FRONTEND_DIR=/absolute/evidence/frontend-rows cargo test --locked --offline --manifest-path rust-game/Cargo.toml --lib -- blue_minimum_tests:: rules::tests::blue_closed_definitions --test-threads=1
bash rust-game-wasm/build.sh
node tools/validation/verify-blue-minimum.mjs /absolute/checkout /absolute/evidence
cd web
npm run build
npm test -- --run src/game/BlueMinimumScan.test.jsx src/game/ChoicePanel.test.tsx src/game/ReadModal.test.tsx src/game/CardTile.test.tsx src/game/PrivateDeckTop.test.tsx src/game/deckLibrary.test.ts src/game/DeckLibrary.test.tsx src/game/SocietyDeck.test.tsx src/game/MixedDeckConstruction.test.jsx src/game/Lobby.test.tsx src/game/Table.test.tsx
```

审查包另含713→候选完整binary patch、7c6→候选增量patch、候选git bundle、全部变更源码/三张原图、最新33与冻结32、原始测试正文与失败/通过日志、冻结文件hash表及恢复说明。包外无 node_modules、构建缓存、认证数据。当前线上仍是交接记录的Site30/b897；本任务未探测或修改线上状态。
