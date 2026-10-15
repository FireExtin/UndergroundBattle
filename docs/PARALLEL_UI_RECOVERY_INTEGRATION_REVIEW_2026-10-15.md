# 并行 UI / 恢复整合只读评估

日期：2026-10-15。用户提前收尾，本报告保存已有源码判断后停止；没有整合、修改生产/测试、执行 Cargo 或启动验证门禁。共同基线为 `3d642f0c1e6e5ac021cd40560086df35638bfe4c`。本地已保存引擎 HEAD 为 `dbdee5d6014c260f9833c162bf85aff4b491b690`（ABI76），工作区为进行中的 ABI77，父任务将其作为 WIP 保存。

**最新候选在源码职责上兼容，已具备明确的最小整合路径；尚未完成合并后的验证，不能据此称为可发布版本。** 本次仅用 Git show/diff/log/merge-base/grep 等读取对象，以及只读文件/哈希比对。

| 候选 | 精确提交 | 相对共同基线 |
| --- | --- | --- |
| 恢复 | `799717557a3536ebae75c472e1e5c4a58ac12fc9` | 17 个文件，6 个提交 |
| UI | `46f4bf98324d90c8ae60e6b9e229f00c7f3c1f61` | 19 个文件，6 个提交 |

最初检查的恢复 `5dc5907287a604aa2bb595c13c6a8850e74e9e2a` / UI `049c0de59bf1a17bcc062a9777df02df9119b507` 只有 ChoicePanel 交集。收到最新提交后已经更新评估，以下以 799 / 46f 为准。

## 共同文件与已解决的兼容点

两候选共有 6 个改动文件；读取提交内实际文件字节比较如下：

| 文件（均在 `web/src/game/`） | 最新候选关系与处理 |
| --- | --- |
| `choiceDraftStorage.ts` | 字节相同，恢复实现保留一份。 |
| `choiceDraftStorage.test.ts` | 字节相同，恢复测试保留一份。 |
| `Table.tsx` | 字节相同，均传入 draftStore/recoveryNotices 并根据权威 pendingChoice 清理草稿。 |
| `CurrentLifecycleChoices.test.jsx` | 两候选字节相同；整合时必须保留父分支当前 ABI77 fixture 引用。 |
| `ChoicePanel.tsx` | UI 版已融合恢复基底及后续交互，可采用 UI 最终版；不能退回旧 049 的独立 React 草稿状态。 |
| `ChoiceRecovery.test.tsx` | UI 版保留恢复断言并适配新可访问名称，增加筛选/单选替换/刷新恢复测试，可采用 UI 最终版。 |

UI 中 `565d571d052f2ceca4d660acdd8473f862cdbb60` 明确记录 cherry-pick 自恢复 `f12f47d71aaae8634af79e0efd94d3b6465fd7c0`。当前 UI 的 ChoicePanel:41–54 仍以 `draftRef` 为即时草稿来源，编辑时同步更新 ref、保存、再更新 React state；:116–120 提交读取当前 ref 并检查 busy/完整性。伤害加减、顶底移动和回底排序都经同一更新入口。新焦点 hook 只处理焦点，不另建草稿或存储层，符合“恢复负责存储与同步 draftRef，UI 负责交互与视觉”的分工。

旧组合存在确定的 aria-name 测试适配点：恢复测试曾精确查找 `+角色乙`、`+新目标`、`1最新角色b`，UI 将选择状态设为 aria-hidden 后名称已变化。最新 UI 的 ChoiceRecovery 已分别改为公开名称并保留禁确认、清空旧选择、不自动提交等断言；不能在整合时用恢复旧测试整文件覆盖它。

## 最小整合顺序

留待父任务恢复工作后执行，本轮不操作：

1. 先固定 ABI77 的 WIP/验证基线，再引入恢复 799 的累计 17 文件变化。恢复提交顺序为 `60c4389 → 09d7f8d → f12f47d → 5dc5907 → 9d04776 → 7997175`；仅摘最后一个提交不能得到完整恢复功能。
2. 保留恢复侧 API、useGame、GameApp、同席提示及存储失败重试链。再采用 UI46f 的两个不同共同文件（ChoicePanel、ChoiceRecovery）及 UI 独有 13 文件；4 个字节相同的共同文件无须重复覆盖。不要将整个 `799..46f` 差异直接覆盖工作区，因为其中也包含 UI 分支没有携带的恢复侧变化的反向差异。
3. 唯一与本次读取的 ABI77 工作区相交的候选文件是 CurrentLifecycleChoices。保留当前 `currentLifecycle77.fixture.json` 引用及完整 WASM SHA 校验，同时纳入恢复的选择/牌序刷新断言；不把旧 opaque 状态改版本号。UI 的 PriorityFactionSelection 明确只读 Native74 投影，JZ50Search 也有历史投影测试，不执行这些旧状态的当前内核重放；这些历史输入可原样保留，不能把它们称为 ABI77 机制验证。
4. 后续组合验证至少覆盖恢复/ChoicePanel/排序/焦点/候选身份/组卡测试，以及真正绑定当前 ABI77 输入的生命周期测试与 TypeScript/构建。提交说明里的原分支测试数字仅是作者历史记录，本复审未复跑，也未验证融合后结果。

## currentDomains 仍缺展示

两条最新精确提交及本次读取的父工作区 UI 都没有 `currentDomains` 类型或渲染。`types.ts:34` 只有印刷 `magic`；`CardTile.tsx:96` 仅读取 `card.magic / definition.magic`，ReadModal 复用 CardContent，因此获得领域不会在当前状态阅读中显示。合入上述两候选不会自动补齐此功能，应另交 UI 任务。

当前服务端 `model.rs:81` 为可选 `MagicDomainCount[]`，`jc060.rs:83` 输出形如 `[{magic:"心灵",count:N}]`，N 为顾问来源数；这是额外获得的领域，不是所有印刷领域的替代清单。展示须保留印刷 magic，并清楚表达额外领域及数量，不能回填组卡/资产的印刷资格。`engine.rs:2152` 对暗藏（包括其操控者）、资产及场外对象不投影该加成；将来 UI 接线也须维持这些隐藏/空白边界。

没有发现需要先改引擎机制才能整合的源码兼容阻断。剩余工作是父任务择时执行有界整合、补 currentDomains 展示并验证实际组合；当前保持 WIP，不发布。
