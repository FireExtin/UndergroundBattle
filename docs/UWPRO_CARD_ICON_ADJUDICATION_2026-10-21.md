# 三张卡的印刷图标独立裁定

范围：JC029 新生血族、JZ49 蹒跚行尸（UWPro 编号 JZ049）、JC044 灾难使者；只裁定本次固定原图的黑底一般图标／白底临时图标，不扩大卡牌范围，不修改产品、冻结历史内核、旧回放或旧证据。

授权依据：父任务本次转述用户已直接确认代码作者及插画作者许可其复用，并授权最小导入预览；这不是公开 LICENSE 已变更的证明，也不推导第三方普遍授权。本次仅只读审查和写本笔记；未重试 Library 401 下载，未推送、部署、触发 CI 或运行仓库脚本。

## 裁定结果

下面三元组顺序为 **调查、战斗、势力**。白底“临时”指先手条件，不是“获得直到回合结束”的持续时长。

| 卡牌 | 实际像素观察 | 印刷永久／一般图标 | 印刷临时／先手图标 | 裁定 |
|---|---|---|---|---|
| JC029 新生血族，029/135 | 文本框左侧两枚：黑色双剑配白底、黑色势力星配白底；两枚均有白底方框 | `(0,0,0)` | `(0,1,1)` | UWPro 字段与原图相符；我方固定基线将势力1记为永久，属于印刷字段转写错误 |
| JZ49 蹒跚行尸，49/76 | 文本框左侧上下两枚双剑，均黑色双剑配白底；没有一枚黑底双剑 | `(0,0,0)` | `(0,2,0)` | UWPro 字段与原图相符；我方固定基线记为永久战斗1＋临时战斗1，属于印刷字段转写错误 |
| JC044 灾难使者，044/135 | 文本框左侧眼、双剑、势力星三枚，均白色符号配黑底；底部势力星的中心也是白色，和临时星的黑中心不同 | `(1,1,1)` | `(0,0,0)` | UWPro 字段与原图相符；我方固定基线将势力1记为临时，属于印刷字段转写错误 |

三张原图本次均足够清晰，不需要因图义模糊而隔离；但“裁定清楚”不等于已经修改运行注册表或完成机制回归。本任务未改任何产品文件。

## 实际查看与来源固定

我实际调用 `view_image(detail=original)` 查看了 `web/public/cards/JC029.jpg`、`JZ49.jpg`、`JC044.jpg` 的完整原图；随后逐字节比较 `resource/ymsj-fun.github.io/cards/` 同名原资料副本。三对文件分别完全相同，也与 `git show 100161b628e56b9153041d952ac0f5d83428ca88:<path>` 返回内容完全相同。不是依赖旧文档声称“已看图”，也没有把卡片文字 OCR 当图标判据。

我方审查基线是 **FireExtin/UndergroundBattle@100161b628e56b9153041d952ac0f5d83428ca88**；本地 HEAD 仍是该 SHA。父任务另外读取更新后的 main，本裁定不假定新 main 未发生相关变化，合入前应比较新 HEAD。UWPro 基线是 **RETORETO/UnderWorldPro@0d60c78b68697867697f20f0c171e41445735cc4**。

| 文件 | SHA-256 | Git blob SHA |
|---|---|---|
| JC029.jpg（338285字节） | `b3da8f1546e2f8bbbaae8d3c4d4c79dd712cfa7f8d133fe1723efaa1fa152785` | `ad4413df065848f59b666a172dfa5bc166f4414a` |
| JZ49.jpg（499091字节） | `208b2370433fb420ba91e1557ad6633589e43b1391d2008d63ebee2ed90fdf62` | `801ef5d4a758aa15cea0378355b7745ffbe27df2` |
| JC044.jpg（361527字节） | `e250b0edab5c9ba2dd3a3d3cd15d0708bb72420ca6a059bcb6b116321c411800` | `443891f38602794f8af25385c4347c09252b573f` |

固定提交原图链接：[JC029](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/web/public/cards/JC029.jpg)、[JZ49](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/web/public/cards/JZ49.jpg)、[JC044](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/web/public/cards/JC044.jpg)。

## 原规则的符号证据

1. **《隐秘世界规则手册》印刷第6页／PDF物理第8页（零基索引7）**：右栏“临时能力图标”明确一般图标为黑底、临时图标为白底，临时图标仅在操控者持有先手标志时生效；其下图例直接画出白底黑眼、白底黑双剑、白底黑中心势力星。本次用 PyMuPDF 在内存中渲染原 PDF 整页并实际查看，未保存或修改图片。它与三张卡片的底色可直接对应。正文同时规定：效果若计数牌上的能力图标，临时图标无论本回合是否生效都计入，不能据此将它转写成永久图标。
2. **《隐秘世界勘误及释疑》印刷第6页／PDF物理第6页（零基索引5）**：右栏“图标和符号”分别并列一般／临时的调查、战斗、势力图标。本次同样实际查看原 PDF 渲染页；一般势力是黑方框中的白中心星，临时势力是白方框中的黑中心星，排除了仅凭星形轮廓辨色的歧义。
3. 已搜索该6页FAQ中的卡名。新生血族见物理第2页，讲袭击触发；灾难使者见物理第3页，讲进场时间标志及支付范围；未检出蹒跚行尸。上述条目没有改写本次三张图的能力图标。FAQ旧编号与当前卡图编号不同，按同名能力核对，不以旧编号覆盖当前印刷版本。没有宣称已验证其他未知印次或外部未取得的勘误。

原规则手册 SHA-256 `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`，Git blob `04f5a7b711caf8e3ee3e7cb341099d855c644271`；FAQ SHA-256 `916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38`，Git blob `dc7d92b4e798abe711669484f378ef2abf488c9b`。两份文件均逐字节匹配我方固定 SHA。

固定来源：[原规则手册 PDF，第8物理页](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/resource/ymsj-fun.github.io/public/docs/%E9%9A%90%E7%A7%98%E4%B8%96%E7%95%8C%E8%A7%84%E5%88%99%E6%89%8B%E5%86%8C.pdf#page=8)、[原勘误及释疑 PDF，第6物理页](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/resource/ymsj-fun.github.io/public/docs/%E9%9A%90%E7%A7%98%E4%B8%96%E7%95%8C%E5%8B%98%E8%AF%AF%E5%8F%8A%E9%87%8A%E7%96%91.pdf#page=6)。GitHub预览可能不跳转PDF页，以上物理页码是本地原PDF核验结果。辅助文字转写在 [规则手册 Markdown L449–454](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/organized_content/rules/%E9%9A%90%E7%A7%98%E4%B8%96%E7%95%8C%E8%A7%84%E5%88%99%E6%89%8B%E5%86%8C.md#L449-L454)，但裁定以有图例的 PDF 为准。

## 双方字段和实际统计代码

### 我方固定基线

- `rust-game/data/cards.json:3473-3483`：JC029 永久势力1、临时战斗1。[JC029字段](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/data/cards.json#L3473-L3483)
- `rust-game/data/cards.json:3912-3922`：JZ49 永久战斗1、临时战斗1。[JZ49字段](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/data/cards.json#L3912-L3922)
- `rust-game/data/cards.json:5781-5791`：JC044 永久调查1/战斗1、临时势力1。[JC044字段](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/data/cards.json#L5781-L5791)
- `catalog():110-126` 直接从该JSON的 `icons.permanent/temporary` 填入 `CardDefinition.permanent_icons/temporary_icons`，不是名字不同但语义等价的别名。[目录装配](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/src/catalog.rs#L107-L126)
- `engine.rs:402-417` 将两类印刷字段分别作为角色图标基数；`current_icons:345-352` 根据 `team(controller)==first_team` 传入先手条件，`combine_character_icons:355-370` 只在先手时加 temporary（除明确的临时转永久效果外）。`icons:325-329` 另处理横置后不参与对抗。因此在正面、未横置、无额外授予或转换的对照状态下，这些静态分类差异会改变非先手对抗值，不能解释成UI标签差异。[统计入口](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/src/engine.rs#L325-L370)、[印刷基数](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/rust-game/src/engine.rs#L402-L417)

### UWPro固定基线

- JC029 `.tres:24-29` 为 Investigation/Strength/Power=`0/0/0`，FirstStrike=`0/1/1`。[字段](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Cards/Data/PlayerCards/JC/JC029_xinshengxuezu.tres#L24-L29)
- JZ049 `.tres:24-29` 为 `0/0/0`、FirstStrike=`0/2/0`。[字段](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Cards/Data/PlayerCards/JZ/JZ049_panshanxingshi.tres#L24-L29)
- JC044 `.tres:24-29` 为 `1/1/1`、FirstStrike=`0/0/0`。[字段](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Cards/Data/PlayerCards/JC/JC044_zainanshizhe.tres#L24-L29)
- `CardInstance` 从模板普通属性设置 CurrentStrength/CurrentPower，先手属性分开保存；`AreaStatsCalculator.ValueOf:25,45-50` 仅当 ControllerId 是先手玩家才加 FirstStrike。因此映射为我方 `permanent/temporary` 有直接代码依据。UWPro名为 FirstStrike 并不表示与原规则“临时图标”不同的机制。[模板到实例](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Script/Game/Entities/CardInstance.cs#L973-L1001)、[对抗统计](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Script/Manager/AreaStatsCalculator.cs#L21-L50)

这只证明UWPro三张印刷图标字段与本次原图相符，不证明该项目三张卡完整效果正确，尤其不以静态字段替代其未注册效果的验证。

## 旧文档冲突应如何记录

旧我方 `docs/factions/card-specifications.json` 的 JC029（起始L27625）、JC044（L29432）、JZ49（L45975）和现有运行表有相同误分类。`primaryImageReviewed`／`completeGameplayFieldsForPinnedImage` 状态是旧审查结论，本次可被相同哈希原图反证，不能压过像素证据。

其中 [JZ49_CARD_FACE_CANDIDATE L9](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/docs/JZ49_CARD_FACE_CANDIDATE_2026-10-07.md#L9) 明写永久战斗1＋临时战斗1；[JC044_SERIAL_WIP L5](https://github.com/FireExtin/UndergroundBattle/blob/100161b628e56b9153041d952ac0f5d83428ca88/docs/JC044_SERIAL_WIP_2026-10-18.md#L5) 明写临时势力1。两处是此次需登记的审查更正点，而非原图存在歧义；历史文档应保留其历史身份，通过新增更正说明串起新裁定，不把旧版本运行结果改称新图标结果。

## 最小后续验证建议（本子任务未执行）

仅对这三张卡增加或更新**新规则版本**的印刷字段与基础对抗断言，不改历史冻结规则。使用正面、未横置、无附属、无外部增益/临时转永久效果的状态，分别验证先手／非先手三元组：

| 卡牌 | 先手有效值 | 非先手有效值 | 旧基线非先手值 |
|---|---|---|---|
| JC029 | `(0,1,1)` | `(0,0,0)` | `(0,0,1)` |
| JZ49 | `(0,2,0)` | `(0,0,0)` | `(0,1,0)` |
| JC044 | `(1,1,1)` | `(1,1,1)` | `(1,1,0)` |

JZ49“迟缓”会横置进场，若只测刚进场状态，双方算法都显示0，会掩盖错分；必须补正常重置后的先手/非先手对照。三张卡的先手总数恰好不变，只测先手也无法发现错误。再检查图标数量类效果仍统计临时图标、Native/WASM投影一致、旧牌桌继续路由原冻结内核。这是验收设计，不是已通过的测试结果。

当前阻碍：无图义阻碍、无工具拒绝、无待审批动作；未运行行为测试，未审查更新后的main差异，未裁定其他印次。仅新增本文件。

## 主研究补充：新 main 与隔离策略

已核对 `d98117d188cfc65f4e29fb757dfcf9accdd545e5`：三张卡的完整注册数据及 web 原图与上述 `100161` 基线一致，因此裁定适用于本分支基线。导入映射暂时隔离三张；原因是目标内核注册表仍有冲突，而非原图不清晰。此次仅做导入预览，不修改 Rust 字段、ABI、历史内核或现有素材。需要主卡牌任务在新版本修正并通过先手/非先手回归，再明确解除隔离；不能单凭新版本号或目录字段变化自动解锁。

随后正常读取的 `d1c068ea8c4162212cd19fa023841513c4c67e33` 仅有20卡批次来源文档变更；运行卡表和三张图片没有变化。本节补充不把此前独立只读裁定改称全局内核复审，也不代表后续主任务已完成修正。
