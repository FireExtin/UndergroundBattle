# JC095 唐·格鲁尼 / ABI76 独立源码复审

日期：2026-10-15。基线：`e53e8b017f0ec974c6cf3d8f446b6541771cf4a5`（父任务已保存并推送的 JC060 / ABI75）。分支：`codex/jc095-black-influence-conversion-review-20261015`。基线和分支身份由父任务交接；本复审未运行 Git、Cargo、构建或内核测试，未参与 JC095 实现，也未修改生产或测试文件。本轮仅新增本文。

**当前源码未发现未修复的阻断问题。独立预审发现的一处 P1 接线遗漏已补正；JC095 的 19 项测试在父任务整库运行中均通过。所审源码已经父任务确认冻结，指纹记录于文末；WASM、Worker、前端的后续实际结果由父任务补充，本文不提前标绿。** 本文不是浏览器自然整局、生产部署或发布验收。

审阅顺序为独立新模块及原始接线 patch，随后对照实际接线后的源码和测试修正；没有因设计文档已有结论而直接采纳实现。主要范围为 `jc095.rs`、`jc095_tests.rs`、`engine.rs`、`resolution.rs`、`rules.rs`、`model.rs`、`room.rs`、目录与版本接入，以及本卡来源、当前输入、WASM、Face、Worker 验证代码。共享的领袖、声望、赢区和支付流程按本卡调用路径复核。

## 发现与处理

### P1，已修复：原接线只校验 Game 恢复，遗漏 Room 恢复

位置：`rust-game/src/room.rs:225` 的独立反序列化入口及 `room.rs:254` 的新增守卫；对应 Game 入口为 `rust-game/src/model.rs:852`。

原始 `/workspace/jc095-evidence-20261015/integration.patch` 接入了 `Game::from_persisted` 的 `validate_jc095_state()`，但没有修改 `RoomEnvelope::from_persisted`。后者直接反序列化嵌入的 Game，再自行调用各状态守卫，不经过 Game 的恢复函数。因此仅有 Game 接线不能覆盖真实 Room/WASM/Worker 使用的恢复容器。

静态复现：由合法 JC095 进场接受待选态复制存档，仅清空 `pending.choice.title`。该输入违反本卡完整待选元数据契约；原 patch 下 Game 入口会拒绝，Room 入口没有调用本卡守卫，不能保持相同拒绝边界。现有 `jc095_pending_metadata_and_frozen_source_identity_are_strict` 的 variant 3（`jc095_tests.rs:419`）正覆盖该输入，`invalid` 辅助函数同时验证两个入口。

预审发现后立即通知父任务；父任务保留原 patch，并在 `room.rs:254` 增加 `room.game.validate_jc095_state()?;`。复审已确认实际接线。父任务后续运行中该测试通过，不能把修复后的通过情况写成原 patch 从未有缺口。未要求改变规则语义或放宽校验。

### 验证修正，不属于规则缺陷：计分后地区采用新实例 UID

初轮定向测试为 18/19；唯一失败项一直等待旧地区 UID 出现在计分区，直至 192 步上限。`engine.rs:1665` 的已有 `Effect::Score` 在转移区域时调用 `fresh`，因此旧 ID 等待条件不可成立。复审与实现者分别定位到相同原因。

实现者只修测试推进条件：按原地区定义等待，同时保留既定赢家、唯一计分、新旧 UID 不同、计分牌 owner/controller 均为原赢家、JC095 回拥有者牌库的断言（`jc095_tests.rs:1112` 起）。`win-fix.log` 的单项复测 1/1 通过，后续整库运行的 19 个 JC095 测试均通过。没有删除赢家断言或修改生产赢区逻辑来迎合测试。

整库首轮另外记录了 MSJC07 旧目录数量断言：实际黑色非地区牌为 25，旧预期为 24；错误位置是共用辅助函数 `msjc06_tests.rs:164`。父任务只将 `msjc07_tests.rs:14` 的 `color_count` 更新为 25；复审已读取当前文件及单项复测 1/1 通过日志。这不是本卡转换规则缺陷，但首轮整库失败必须保留，不能记成同一次整库全绿。

## 原图与规则核对

实际查看了 [JC095 完整原图](../resource/ymsj-fun.github.io/cards/JC095%20唐·格鲁尼.jpg)，并核对 [设计及原页定位](JC095_NEXT_BATCH_DESIGN_2026-10-15.md) 与规格。原图 SHA256 为 `32dbf8c8af063f8b680bbba87c82920ac5a7dd4a18bcdb33e0d861894e05c020`。

- 金名独有；唐·格鲁尼 / 波士顿之主；黑色角色，人类、罪犯；费用 7，黑忠诚 3，防御 2，无印刷魔法领域。
- 永久调查/战斗/势力为 0/2/1，临时为 3/1/0。关键词为领袖、声望；独有没有被改成构筑“唯一”。
- 正文为“【进场触发】将本地区上的所有敌方势力标志移除，然后放置等量的本方势力标志。”没有目标、强制、费用、数量上限或来源必须留场的附加条件。

本轮额外实际查看了霸权说明书 PDF10 / 印刷 P9（势力抵消、赢区及后续窗口）、PDF11 / P10（非强制触发、可响应、现身属于进场）。此前同一复审会话也已实际查看霸权 P18 的领袖/声望/目标防护、P21 团队规则及旧手册暗藏/持续能力原页。没有将设计作者所列全部原页冒称为本复审逐页重新阅读。

原地区锚点是父任务明确采用的 **JC095 单卡项目适用**：冻结成功进场时 actor、槽位及地区 UID，来源后来移动、潜伏、回手、死亡或转控不改变已产生的效果；原地区 UID 消失则不作用同槽新地区。未找到 JC095 专属官方 FAQ，本复审不把同型卡 FAQ 或其他卡裁定升级成所有“本地区”文字的官方通则。2v2 团队声望合计、优势队仅一次可选奖励和赢区优先于声望也沿用已有项目口径。

## 实现路径复核

| 边界 | 实际源码判断 |
| --- | --- |
| 完整定义 | `jc095.rs:33` 是单个可选、可响应的 `Enter` 能力，空费用、空目标、固定无参数操作，并启用声望。精确定义及能力守卫阻止删关键词、改事件、移植或嵌套转换程序。 |
| 原 actor / 原地区 | `resolution.rs:403` 的来源快照已纳入 JC095；`jc095.rs:79` 同时检查槽位、非空位和冻结地区 UID。转换取 `frame.actor` 所属队，不追踪来源当前 controller 或 owner。付费派遣/现身也在 `engine.rs:1061` 检查原地区 UID。 |
| 当前敌方数量 | `jc095.rs:87` 在响应结束后读取原地区敌队共享计数 N；N=0 直接结束，不新增 Award。N>0 先清零敌方池，再调用 `place_influence(actor, region, N)`，保留普通阈值流程，不会只抵消成零或按两个敌席重复转换。 |
| 原子性与非目标 | 移除和放置在同一个固定操作内完成，无中间选择、续体或持久化 N；目标列表为空，因此不消耗护盾，也不经过角色屏障的目标检查。 |
| 进场与来源离场 | 正面派遣和付费现身经通用成功进场事件生成声明。秘密派遣、建立资产、仅进入地区不触发本能力。实际回手/潜伏响应有 Room 连续命令覆盖，既有离场清理不被本卡绕过。 |
| 领袖 | 复用 `jc052.rs:75` 起的印刷领袖与当前 controller 席位检查，不按 owner 或整个团队合并。同席强制牺牲先于可选进场声明；牺牲新 JC095 后已观察到的进场声明继续。 |
| 声望与获授奖励 | 复用 `renown.rs:45` 的正面资格和 `:72` 的当前未横置团队计数。`jc095.rs:160` 只接受绑定冻结地区的精确声望及 JC089 威名奖励，拒绝伪改数量/地区或借 key 移植转换。 |
| 已赢窗口 | 转换只改变势力池；`engine.rs:1587` 的 Award 按地区 UID 与已有 WinContext 去重，不改旧赢家。真实现身/接受/响应/回底/计分链明确检查原赢家及全局唯一计分。 |
| 费用和动作 | 忠诚、资产/资源支付和减费仍由共享流程执行；本卡拒绝额外目标、模式、选择等 payload。支付记录允许至多 7 个唯一非空资产 UID，不错误要求必须 7 张资产而排除资源或减费。 |

## 恢复、目录和输入边界

`jc095.rs:109` 校验冻结定义、来源 UID、actor/controller、owner、原地区槽位/UID及禁止的附属/死亡观察字段。若相同来源 UID 仍存在于任何已覆盖的实体容器，其不可变 owner/definition 必须一致；不会因合法离场、转控或身份重置而要求当前来源仍在原处。

`jc095.rs:172` 起分别封闭付费空程序、固定触发与两种已有奖励。拒绝多余目标、错误 step context、支付/游标/guard、帧壳 controller/frame ID、真实 JC095 卡配其他帧、伪造进场卡及直接排入 Effect 的转换帧。声明只允许精确原能力；待选必须是 actor 的 Accept，min0/max1、可放弃、唯一 accept 选项及固定标题/说明，无预览或数量字段。Game、Room、选择和实际帧执行入口均已接线；领袖强制待选继续由共享 JC052 状态守卫检查。

当前目录为 142 张 / 132 张非地区牌，只有 JC095 为本批新增。版本为 `hegemony-pdf-v1` / `limited-v2.69-jc095-influence-conversion-candidate` / `rust-v0.2.76-jc095-influence-conversion-candidate`。来源测试锁定原 141 张定义和目录元数据，保留完整原图扫描；没有新增印刷领域或用顾问的当前领域回填印刷字段。

已读本卡 `jc095-current.py`、`jc095_compare.mjs`、`jc095-source.test.mjs`、`JC095Face.test.jsx` 和 `jc095-integration.test.mjs`。当前输入生成器校验精确 76/69 目录身份、11 个场景集合、输入连续性及 WASM 哈希，已有不同内容的证据拒绝覆盖。WASM 比较代码检查完整 Native 结果和全部席位视图，并明确拒绝未改写的 ABI75 存档；非法存档目录只有实际传参运行后才能计为覆盖。Worker 验证代码包含完整状态/日志、D1 重新打开及重复命令回执。阅读这些代码不等于已运行或通过其测试。

## 已核实的验证证据

以下运行由父任务/实现者执行，复审只读取日志或导出文件；没有重复运行 Cargo 或内核，也没有将代码阅读计作运行结果。

| 证据 | 实际结果与范围 |
| --- | --- |
| `native-first.log` | JC095 初轮 18/19；失败为已说明的旧计分 UID 测试条件。保留首次失败。 |
| `win-fix.log` | 修正推进条件后，已赢窗口单项 1/1 通过；另有 1072 filtered out，不能称整库结果。 |
| `native-regression.log` | JC095 19 项全部逐项 `ok`；整库本轮为 1071 passed / 1 failed / 1 filtered out，唯一失败是 MSJC07 旧黑牌数量 24，实际 25。 |
| `native-count-fix.log` | MSJC07 数量预期修正后单项 1/1 通过，其余 1072 filtered out。与首轮共同说明全部已执行单元项已通过；不是修正后再执行一次完整整库，也不覆盖首轮过滤项。 |
| `native-integration.log` | 7 个集成测试文件分别为 3、27、5、2、12、10、8 项通过，合计 67/67；均为 0 failed / 0 filtered out。 |
| `native-final/*.json` | 独立只读解析确认 11 个场景、146 条命令、614 个席位视图、3 个预期拒绝；每条初态等于首步输入，步骤间完整 opaque state 连续，目录为 ABI76。这是数据一致性检查，不是再次执行内核。 |
| `source-tests.log` | 来源/扫描组合测试 37/37。 |
| `generator-guards.log` | 生成器守卫测试 19 项通过。 |

11 个连续场景为 paid-deploy-accept、paid-deploy-decline、secret-reveal、response-increase、response-zero、source-return、source-hide、leader-sacrifice-new、renown-team、win-window-fixed-winner、duel-paid-deploy。它们从明确披露的离线布局开始；之后的关键行为走真实 Room 命令、恢复及日志重放。

## 验证限制

- 来源移动/转控、同名再入场及原地区换 UID 的部分边界采用直接状态或原语探针；不能称为完整自然命令链。实际来源回手、潜伏和牺牲有各自连续命令证据。
- 本卡测试分别覆盖转换、声望和既定赢家流程，没有将“转换敌方两枚 → 后续声望补足 → 完整赢区”作为一条新的连续自然局面导出；现有共享赢区回归和本卡专项不能冒称该整链已经试玩。
- Face 单元测试不等于四端浏览器验收；组卡、刷新/多端同步、客户端当前领域展示、自然资产与忠诚积累及完整整局属于另行交接范围。本复审未变更 UI 生产文件。
- 最终 WASM/Worker/旧版本拒绝的实际结果尚待父任务完成，不能提前标绿。未涉及部署、迁移旧局或扩大准入范围。

## 冻结指纹

父任务确认 JC095 机制、新测试、恢复守卫及 MSJC07 数量修正已冻结后，读取实际文件计算下列 SHA256；没有执行 Git。指纹绑定本次审阅的文件字节，不将后续生成输入或文档更新冒认为已复审。最终 Git 提交身份由父任务保存后补充。

冻结读取时间：2026-10-15 05:56:06 UTC。

| 文件 | SHA256 |
| --- | --- |
| `rust-game/src/jc095.rs` | `37af979f23dfb622c35f53de5c2a2e5f9c2c6b4402f536170aee47d0a7c7944b` |
| `rust-game/src/jc095_tests.rs` | `ffe1a467b14489ff86e5260df6ff995cc8a3f4bdbb63aeec9c25e26c1b06b61c` |
| `rust-game/src/engine.rs` | `568346d8a0a35ca81dec71cde44da572e0e1717874f00ed19c4a70cbc67dcd64` |
| `rust-game/src/resolution.rs` | `91f09483afbefa7500ec21409adfcaf88366b5a59a2e5b7301f8cc064bbfa968` |
| `rust-game/src/rules.rs` | `27dfaa5a8cd282b0dd8604d7dfb110834c470260f0b371d336c693183c25bae5` |
| `rust-game/src/model.rs` | `2d1ab546a831098d4001779a8c31f418dcd7d1a675a8b75787573244869c62f0` |
| `rust-game/src/room.rs` | `0a0755764ad861a918093bd266bd005d50020f70cf0bb7124a4fd075078550f8` |
| `rust-game/src/lib.rs` | `3f682b66ec6bbfc5691c352a4dfe2ef991e8253826b6b92b229709827d652fbc` |
| `rust-game/src/catalog.rs` | `b756938cbc890ee7fac0406d8b6820335822722fa2d0a1dd33b409697aa32fee` |
| `rust-game/src/jc052.rs` | `62d7b128601c827662fc9d27842e490724581c8fda6b46e67d56d8d2ca020c3e` |
| `rust-game/src/renown.rs` | `8aea1693418cb872161603aebe77f50c48947ba3d77364e7b27e1268ca9e61d5` |
| `rust-game/src/jz22.rs` | `8c62cb699bc39bbbd69dd35e359e9a44a3fd2200ff3ab4edfd01e9dac1e2308d` |
| `rust-game/src/msjc07_tests.rs` | `27c034e581a001b2ce0fad566de268d262b8c501ee0e8a1f9004f1cc284d480f` |
| `rust-game/src/msjc06_tests.rs` | `764613e131756e4a834fcd8613e196e3c13ccbcd4ab4790769b6a8d10dc21fb2` |
| `rust-game/data/cards.json` | `061364aa2bfdcecc851c0af232eed39f326eaf83b9d9196e14c6348ee18818e2` |
| `tools/cards/jc095-source.test.mjs` | `3f035d14e66f2b05d8f4bf389a33de22c4acc8323ca3b2581344a752dccd3430` |
| `tools/fixture-tools/jc095-current.py` | `81896427c5411bb3344d46ba0e5d703d77547edf38af1c02d6e4bd7dc4325ddf` |
| `tools/fixture-tools/regenerate_native72_inputs.py` | `7eb3d3c1fd29e92c7aef48c389f814c6d5b42d20a42894c25bfa876360605afb` |
| `tools/fixture-tools/test_regenerate_native72_inputs.py` | `9401657db6bbce539976ed33346ca9b9e5d8d1d0ad8c33aed36d3ff504df6251` |
| `rust-game-wasm/tests/jc095_compare.mjs` | `2abbde7060c88419681be4629eeadbef7c650619d90ba2f4dddbc641b57ed8b8` |
| `rust-game-wasm/tests/current-kernel.mjs` | `d4272f5b94a97774261aa51b41b5ee04cb610c268412d54bdefb3666b0ae0ecb` |
| `web/src/game/JC095Face.test.jsx` | `6f90045a5e1ddbfeaf4fd58f6bc9973b8eb00e38e2518b57808018e66c976d70` |
| `sites/test/jc095-integration.test.mjs` | `344e11e9454efca654ca1ee0f83c4101fde7c5116b60e9ff7655a46eaa0b32aa` |
| `web/src/game/cardScans.ts` | `6048b4a0017a80c65b6f2cdaea6b074a86f5512335c84ba8f99ec4b7c99a0a3f` |
| `web/public/card-scans.json` | `ffad620a46048b3a2b7c96464136775c4e13249a46774b608f2ced892413e602` |
| `web/public/cards/JC095.jpg` | `32dbf8c8af063f8b680bbba87c82920ac5a7dd4a18bcdb33e0d861894e05c020` |
| `resource/ymsj-fun.github.io/cards/JC095 唐·格鲁尼.jpg` | `32dbf8c8af063f8b680bbba87c82920ac5a7dd4a18bcdb33e0d861894e05c020` |
| `docs/JC095_NEXT_BATCH_DESIGN_2026-10-15.md` | `87ffe1c957767c0cd669ae7b19c580473eb804ed22acef5983a7f6b29e5de828` |
| `docs/factions/card-specifications.json` | `797b502b7d5261ce467f24308f15e7b78f25def6067701f9d4fadff9958a2d68` |
| `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` | `a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886` |
| `tools/cards/validate-admitted-scans.mjs` | `8295abc5bc713a2bcfc2500dc6fb5b33cdfcc5bd0699df5ce6e2eb7476deb4d1` |

已读取的本地证据位于 `/workspace/jc095-evidence-20261015/`，以下哈希供保留首轮失败与修正记录；未将仍在生成的 WASM/Worker 证据纳入。

| 证据文件 | SHA256 |
| --- | --- |
| `integration.patch` | `77e3415ea33ac0933e5fd245b53939f1bed8fee589fed18311f011d86aa38b28` |
| `native-first.log` | `1a07d417019f592046625161fc6996a5e4a56c6af6b9950f0fa00e48edbf7d31` |
| `win-fix.log` | `eb7c2d1d254cf2f485bc9ac7e0a341e1c1d7138c9e14da9a0a8279a308ccf662` |
| `native-regression.log` | `7a33cb70e5a928c00c804f798168926f7c21d01aadd0791714a482c1bd83fd0f` |
| `native-count-fix.log` | `b9e8cdba8ea274178a1e854753a9b098a752500ea9d3c1987827e69aeaecd330` |
| `native-integration.log` | `596c44d1216439ed30ec781347fe2badfdc841a0ae10b16bd2cc969c6ec50058` |
| `source-tests.log` | `97809acd2b78872ecce9d855f53474286abb575b82ca0b736eb5c386adfc3c45` |
| `generator-guards.log` | `6acedd46db69b600557ddc41d6099a7bb724698a615f2367ff5e6dab3dc53a54` |
