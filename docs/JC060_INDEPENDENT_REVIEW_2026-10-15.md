# JC060 ABI75 独立源码复审

日期：2026-10-15。基线：`3d642f0c1e6e5ac021cd40560086df35638bfe4c`。分支：`codex/jc060-gray-occult-adviser-review-20261015`。审查对象是未提交工作区快照，以下 SHA-256 标识本次所审内容；最终提交 SHA 由父任务补充。

**结论：未发现本次变更新引入、可确认的阻断缺陷，P0/P1/P2 findings 均为空。** 本结论是独立源码复审意见，不代表 Native、WASM、Worker 或浏览器验收已经通过。审查者未参与实现，未运行 Cargo、未改实现、未执行 git 写入；本次只新增本报告。

## 已核查的行为与证据

| 核查项 | 源码定位（以本报告指纹为准） | 独立判断 |
| --- | --- | --- |
| 完整印刷定义与来源绑定 | `jc060.rs:10–27`；`rules.rs:1262,1874`；`catalog.rs:5–10`；`cards.json` 的 JC060 | 顾问仅有完整持续修正器；定义删改、重复或移植到别卡均被拒绝。目录新增 1 张，候选 engine/pool 身份一致。 |
| 印刷资格与运行领域分离 | `jc060.rs:36–80` | 资格只读印刷类别与 `MagicIcon::None`；当前授予不会反过来使目标失格，任意非空印刷领域（包括 `Other`）都排除。顾问自身印有心灵，不获得自己的授予。 |
| 当前操控关系、全场范围与叠加 | `jc060.rs:41–69`；`engine.rs:96–101` | 仅枚举各地区当前正面 JC060；按当前 `controller` 的队伍逐实例计数，不按拥有者或地区距离。受益者也须匹配当前在场实例与操控者；来源/受益者离场或翻暗后不保留授予。没有把动态授予写入持久状态。 |
| 永久战斗与横置 | `engine.rs:323–349,399–412,455–470` | 同一来源计数加入永久战斗部分，先后手变化不移除；`icons()` 仍只在对抗贡献层处理横置。没有把黑底图标误当成回合永久缓存。 |
| 灵体数量 | `attributes.rs:150–167`；`attachment.rs:208–220` | 同地区双方累加的是当前领域图标数量，而非有领域牌数。两份心灵贡献计 2；外区顾问自身领域不计入本区，授予本区角色的领域会计入。资产不在该在场枚举中。 |
| JC006 声明与结算重检 | `resolution.rs:642–643,754,757–764,799,1321–1339`；`rules.rs:2694–2726` | 合法动作、目标摘要和结算守卫都走同一动态领域判断。最后一个来源被响应移除后，未结算的目标会失格、效果取消且不退款；没有仅在声明时缓存合法性。 |
| 圣水筛选与已生效结果 | `resolution.rs:1626–1654`；`attributes.rs:170–177,205–208` | 结算时选择本地区当前有领域的角色；随后保存目标实例上的回合防御修正。顾问之后离场只撤销领域/战斗授予，不撤销已生成的圣水效果。 |
| 黑猫、费用、忠诚和搜索边界 | `resolution.rs:435–461,484–500`；`rules.rs:184–195,2508–2516`；`attributes.rs:210–221` | 待打出/待现身牌与资产仍按原来的印刷筛选，不预支进场后的顾问授予。黑猫消耗后 `uses` 从 1 变成 0，既有代码不要求立即删除 modifier；新测试当前已按这一路径断言。 |
| 隐藏、资产、封印与公开字段 | `engine.rs:2143–2147,2270`；`model.rs:79–81,140–145`；`sealing.rs:100–111` | `currentDomains` 仅为公开在场受益角色输出；自己和他人的暗藏者、资产、手牌及封印都没有该字段。`magic` 保持印刷语义；省略 `currentDomains` 不代表印刷领域为空。 |
| ABI75 与旧74 | `catalog.rs:5–10`；`model.rs:816–828`；`room.rs:205–223`；`sites/src/kernel.mjs:5` | 两级持久状态入口仍要求精确 engine/pool/rules 身份，站点路由同样只接受当前身份。新增 WASM 比较脚本保留旧74原始输入，并断言 view/applyRoom 拒绝；本审查没有执行该脚本。 |
| 当前输入生成与证据保留 | `regenerate_native72_inputs.py:38–91,125–175`；`jc060-current.py:8–49` | 先检查当前版本目标不存在，再进行构建/导出；绑定身份与源目录身份均受检查。JC060 汇总保留原始 Native state、连续前后状态及原文件哈希，输出使用新建或相同内容检查；没有旧状态改版本或重写 opaque state 的路径。 |

已通读 `jc060_tests.rs` 的 21 项测试及 11 条 Room 轨迹导出代码。测试设计覆盖四座位/五地区、双人模式、0–3 来源、印刷领域排除、横置、费用不足、来源和受益者转控、来源翻暗/回手、动态目标失格、圣水结果保留与灵体 2:2/3:2 阈值。逐步恢复、journal replay 和四/双视角对比写在测试中；**这里仅确认测试源码具备这些断言，不将其标为已执行通过。**

## 独立执行的只读核对

- 通过本地 `view_image` 实看 JC060 完整原图，确认费用3、灰忠诚1、心灵、永久调查1/势力1、防御1、人类/学者，以及“所有未印刷有魔法领域图标的友方角色获得心灵图标和永久战斗”的正文；与候选目录吻合。
- 实看本地原 PDF 已渲染页面：霸权说明书 PDF19/印刷P18（灵体按领域图标数量比较）、PDF22/印刷P21（2V2友方定义），规则手册 PDF3/印刷P1（暗藏者不是角色）、PDF4/印刷P2（当前操控者和持续能力）。页面与 `docs/JC060_SOURCE_REVIEW_2026-10-15.md` 的来源定位相符；没有用整理版 Markdown 代替图标核验。
- 用 Python 读取 `git show <基线>:rust-game/data/cards.json` 与当前 JSON 逐项比较：去掉新增 JC060 后，原有 140 张定义完全一致；除 engine/pool 身份和 cards 外的全部字段（包括预组牌组）完全一致。
- 原始 JC060 JPG 与 `web/public/cards/JC060.jpg` 逐字节一致，SHA-256 均为 `6a879cc4c380b2a79b9fc3f60acf57c408768a9f8fc4dfa07682c9db63b2a4ae`。

## 剩余验证限制

1. 本审查没有运行任何 Native/Cargo、WASM、Node 回归套件、Worker/D1 或浏览器测试。最终运行结果、当前75产物哈希和旧74拒绝的实测结果须由父任务补充；不引用父任务中途结果代替本次独立执行。
2. `currentDomains` 的界面类型与展示由另一协作任务集成，本引擎分支的 API 正确不等于界面已经显示获授领域。组卡界面、桌面保存/刷新/重复提交和自然浏览器试玩均不在本审查的完成声明中。
3. 新测试没有为每一种来源移除方式分别录制 JC060 专属真实动作轨迹（如来源死亡、封印、玩家退出），也没有逐一录制“来源转控/翻暗发生于 JC006 响应中”的轨迹。源码均通过实时在场/当前控制查询进入同一规则路径，未据此发现缺陷；这些是覆盖边界，不能冒称已逐项运行。
4. 当前准入目录仍采用单一印刷领域字段；本次支持多份获授心灵数量，未实现任意多印刷领域或角色/附属双重类别系统。此结论仅适用于当前141张准入目录。

## 被审文件指纹

以下指纹由本次报告写入时的文件字节计算。任何后续实现或测试变化应重新核对对应文件，而不能仅沿用行号或最终提交号。

```text
0090a80f02c19137634632e45adb03b8073270cd1a1a7e426cf4245f8c2dec41  rust-game/src/jc060.rs
51eef7e001ed59ef279b86495c40ec0fa892c85abba3be1edc15d7ad6f059121  rust-game/src/jc060_tests.rs
7db8e78980bbfc02b7d53511191a5d20cee3ccb747efa3f9b0672a5242200eba  rust-game/src/attributes.rs
89377abb3566e85ab2669fc22c5dede84c529377170015688a120119b5895832  rust-game/src/catalog.rs
dd0b36386557864df918310b579c25b47aea0552843c654ff1a15abb68c92acc  rust-game/src/engine.rs
83a3fff53121a8c7fe62942a33d6208aa2ac8b9aec442134e0438ab6ae0b0497  rust-game/src/lib.rs
690ed5bda5d18e30a7e552af28bf0efaa39710b021a6d28934675da9d8516eae  rust-game/src/model.rs
72052e99681b4255c60c90ce3a8a26e9258cca318f8e9d5be9273b7bb9a48adc  rust-game/src/resolution.rs
3a6b4e8c30192e902b3c8d7a39a5aa52180f572cf252dfde08e95e558cab7867  rust-game/src/rules.rs
b556b68fc0c2156ebd8bc47bb423671f45532572fe3ee15a256be4d15361f8dc  rust-game/src/attachment.rs
82f19b6d0524cd378cbbf486fe607d2afb470b2ff314b2cbfef89a44f2f37539  rust-game/src/sealing.rs
23845ac832fa11205908d8c5cb6ca92bb23ebf07f891366735991930e67a0e84  rust-game/src/room.rs
820bcfe9b04c1f315561b1455e55f609674f98f884d1b85f6f5646a541535140  rust-game/data/cards.json
81b53cdc20c818beb19d09fcf481a8f0bcb3bddf2312a6fc1bc1162defb42861  rust-game-wasm/tests/jc060_compare.mjs
62f49f52985aaf39e355f01ed2b677bef4f4f186b02719a1fe1c79235512bc71  rust-game-wasm/tests/jc037_compare.mjs
9e6cd9a5775f2e760b39ac43d8073d8ecd23e09038ec41564dd2563f80a35f82  rust-game-wasm/tests/jc052_compare.mjs
d4272f5b94a97774261aa51b41b5ee04cb610c268412d54bdefb3666b0ae0ecb  rust-game-wasm/tests/current-kernel.mjs
8030136f2de64fb06509919926745eb55236f1de86dcf74b3f3857c4dddaed82  sites/test/jc060-integration.test.mjs
d651bf6cbbe21d8f879606e7277398b9b1d33cb9ef5d0f66ed0183b55f7d59da  sites/src/kernel.mjs
c678d0728dae27449ba1a3d1d2b4beea7b3cca2a5e65c823cd915c712fcfaee7  sites/src/kernel-router.mjs
c3484b39f681e65ddeb0f3760f2ad13c13083f59f694420b140b43cab3609965  sites/src/lazy-kernel.mjs
5baf917c534bb72a1a47104d2e4655d71e5976de448010fe8a53b08c005c11a9  sites/scripts/qa-preview.mjs
e537ba9ff32dcdff198f0283cf48adc34e4ddf773c219d9a0dfd062b1fc524b9  tools/cards/jc060-source.test.mjs
188c2ad442832e7441157a7927942e95a6c9c372c9e71a1fb313c368fea7f598  tools/fixture-tools/jc060-current.py
615983140eb95b16591526787f2af7bc83ff56d5c8c6f4d2db82e2337c5f000c  tools/fixture-tools/jc037-current.py
30b64eb7e572ba942efc6779b463c7c9ed8eb46d925b25afc2957738c2890750  tools/fixture-tools/jc052-current.py
e241a2044902262eba7ae07bc155b762334bb606b3bde0e00b943afd7f3a1ca4  tools/fixture-tools/regenerate_native72_inputs.py
b294c1b85844e36972097625cb6605244f2ac1a3f3b391b1ff9d05af69c740c5  tools/fixture-tools/test_regenerate_native72_inputs.py
ca1344e0ccc2b01b4b4dd5a61d20d3fac9f824bfe308341ceabc5edafdb81263  web/public/card-scans.json
bf4541306ab1941cec0ffee7505f9157e400d8ee0807b33b4924c2efbf68b678  web/src/game/cardScans.ts
6a879cc4c380b2a79b9fc3f60acf57c408768a9f8fc4dfa07682c9db63b2a4ae  resource/ymsj-fun.github.io/cards/JC060 神秘学顾问.jpg
a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886  resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf
2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35  resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf
```
