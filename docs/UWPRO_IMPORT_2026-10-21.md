# UWPro 牌组导入预览：本地接入切片

## 范围和协作边界

基线：`FireExtin/UndergroundBattle@d98117d188cfc65f4e29fb757dfcf9accdd545e5`。本工作位于独立普通克隆、分支 `codex/uwpro-deck-import`；未建立 worktree，不修改原任务工作目录。

父任务明确另一任务正在进行20卡批次开发。此分支新增独立导入模块、界面、定向测试和文档，仅向 `DeckLibraryPanel.tsx` 添加组件导入与入口。未修改 Rust 内核、卡表、WASM ABI、服务端、冻结版本、发布脚本、README 或存量卡图。整合前由父任务核对主任务新 HEAD 与本分支的文件交集；本任务不推送、部署、触发 GitHub CI，也不联系作者。

期间正常读取到远端 [`d1c068ea8c4162212cd19fa023841513c4c67e33`](https://github.com/FireExtin/UndergroundBattle/commit/d1c068ea8c4162212cd19fa023841513c4c67e33)：8个变更文件均为20卡批次WIP及来源文档，与本切片文件无交集，运行卡表与接入接口的Git blob不变。其[批次记录](https://github.com/FireExtin/UndergroundBattle/blob/d1c068ea8c4162212cd19fa023841513c4c67e33/docs/factions/batch20-2026-10-21.md)仍明确20卡未实现、未入池、未发布。这只是当时读取到的状态，不代替整合时再次核对最新main；本地分支没有自行合并或重定基线。

## 许可与来源

许可依据是本次用户直接确认：**“代码作者和插画作者明确许可我复用了。”** 因而状态为“用户确认已获作者直接许可”，不再沿用此前“未获复用许可”的研究前提。没有取得或编造额外授权条款；也没有声称公开仓库 LICENSE 已更改、许可无限制，或第三方自动获得再授权。

UWPro 固定来源：[`RETORETO/UnderWorldPro@0d60c78b68697867697f20f0c171e41445735cc4`](https://github.com/RETORETO/UnderWorldPro/tree/0d60c78b68697867697f20f0c171e41445735cc4)。保留仓库作者标识 RETORETO、来源 SHA、逐项资源路径以及原素材署名。该仓库 README 当时仍是保留所有权利的公共声明；上述用户直接许可是本项目本次复用的依据，并未改写该声明。

实际复用的材料是已审阅模板路径、ID、牌名及基础字段对照，用于显式身份映射；未复制 C# 效果实现、整张667卡表、Godot资源加载代码或新的图片/字体。解析器和 React 界面独立实现。界面沿用我方现有牌组编辑器及卡图流程，原卡面内的插画署名保持不变。

- 交换格式：[Script/IO/DeckList.cs](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Script/IO/DeckList.cs)、[DeckListLoader.cs](https://github.com/RETORETO/UnderWorldPro/blob/0d60c78b68697867697f20f0c171e41445735cc4/Script/IO/DeckListLoader.cs)。只支持实际导出字段 `deckName/cards/councils` 和条目 `templatePath/copies`。
- 逐项来源和差异：[manifest-review.json](evidence/uwpro-import-2026-10-21/manifest-review.json)。77个玩家模板及5个既有秘社路径均存在于固定 UWPro 对象，牌名与目标目录一致，不在运行时按编号猜映射。
- 我方既有约束：[deckLibrary.ts](../web/src/game/deckLibrary.ts)、[rust-game/src/deck.rs](../rust-game/src/deck.rs)。前端继续使用 `deckCardAdmission/validateDeckDraft`，最终入席仍受原服务端权威校验。

## 已实现行为

在现有组卡页展开“导入 UWPro 牌组”，选文件或粘贴 JSON，点击“解析并预览”。逐项显示原路径、数量、目标卡和差异；原始输入保留。只把完整可转换的牌组载入一个新 `DeckDraft`，不自动保存、选择入席或覆盖旧草稿。当前草稿有未保存修改时，可预览，但禁止载入。

77张四色身份中74张可映射，3张被隔离。身份核验不是双方整张卡规则等价证明；所有游戏解释仍来自我方当前目录与 Rust 内核。五种秘社仅为当前已有的王座会、鸣钟教派、国家机构、方碑序列及通用“秘社”，不增加秘社机制；其他来源明确显示未核验。

任何未知路径、未开放卡、隔离卡、目录身份变化、非法数量或未识别字段都阻止整个转换，不导入已知子集。相同路径重复出现时显示所有原条目，合计数量后交给原同名副本校验。不同ID同名卡仍按原规则合计。不会自动补50张、修正数量、补默认秘社、丢弃缺卡或放宽构筑规则。

已完全映射但数量不足/超副本限制的牌组可作为未完成草稿继续编辑；“保存并选择此牌组”仍由原校验禁用。导入器不新增开局端点，输出只含公共牌组定义，不接收局内状态、随机种子、私有牌库或外部规则程序。

路径只作字符串白名单比较，不读任意文件、不请求URL、不执行资源。输入限制256 KiB、2048条目/张；这些是处理与服务容量，不是新设原作牌组规则。深层非标量字段只显示有界类型提示，避免错误呈现触发递归序列化异常。文件读取使用序号保护，较早的慢读取不能覆盖后来粘贴的文本。

## 三张图标裁定与隔离

完整证据见 [UWPRO_CARD_ICON_ADJUDICATION_2026-10-21.md](UWPRO_CARD_ICON_ADJUDICATION_2026-10-21.md)。实际查看原卡和原手册PDF图例，白底为临时/先手图标、黑底为一般/永久图标。按调查/战斗/势力顺序：

| 卡牌 | 正确永久 | 正确临时 | 状态 |
| --- | --- | --- | --- |
| JC029 新生血族 | 0/0/0 | 0/1/1 | UW字段符合原图；目标基线势力分类错误，隔离 |
| JZ49 蹒跚行尸 | 0/0/0 | 0/2/0 | UW字段符合原图；目标基线战斗分类错误，隔离 |
| JC044 灾难使者 | 1/1/1 | 0/0/0 | UW字段符合原图；目标基线势力分类错误，隔离 |

这些卡的先手总值恰好不变，因此只测先手会漏掉问题。JZ49横置进场也会掩盖问题，需要在正常重置后测非先手。三张卡原图和注册表从100161到本分支d98117d未变；裁定明确，隔离是等待目标规则修正与回归，而不是卡图模糊。本分支不会静默修改内核或自动解除隔离。

解除隔离应由主卡牌任务确认正确字段、新规则版本、Native/WASM同输入一致性和历史房间保持原内核后，在映射文档与清单中显式复审。20卡批次新增目录不会自动开放此前未核验的UW来源路径。

## 本地验证

结果记录于 [validation.json](evidence/uwpro-import-2026-10-21/validation.json)：类型检查、Web生产构建、5个测试文件共62例全部通过。覆盖解析/77身份矩阵、三卡隔离、非法和深层输入、当前目录准入、同名副本、秘社、旧草稿保留、慢文件读取和目录变更。

实际Chromium检查通过1366×768和1920×1080两种PC视口：正常收藏入口、缺卡整组阻断、HTML转义、50张合法预览、新草稿不自动保存、手动保存保留原稿、选择后返回入席且没有发出房间请求；无页面异常或水平溢出。这里使用由实际印刷注册表生成的本地`/api/catalog`夹具，不是真实服务端开局、联网对战或原生浏览器125%缩放验收。

Astra独立静态复核发现深层非法JSON字段在错误呈现时可能递归异常，已改为有界类型提示，加入10,000层嵌套回归，复核确认该路径关闭。首次测试中的夹具可选字段问题已修复；执行环境使旧UI测试超过默认5秒，最终以单worker及60秒命令行超时通过，未改断言或仓库配置。浏览器初次验证脚本误用旧存储格式，修正验证脚本后通过；不是修改产品存储格式来迎合检查。

本结果不是完整内核复审；未重跑Native/WASM全套，不重试此前被拒绝的Library原件下载。依赖及浏览器缓存位于独立构建目录，交付不包含缓存、数据库、凭据或原始素材全集。

可复现的定向命令（在依赖已安装的独立克隆中运行）：

```sh
UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/uwpro-import-build node tools/build-storage.mjs -- npm --prefix web run typecheck
UNDERGROUNDBATTLE_BUILD_ROOT=/workspace/uwpro-import-build node tools/build-storage.mjs -- npm --prefix web test -- --maxWorkers=1 --testTimeout=60000 src/game/uwDeckImport.test.ts src/game/UwDeckImportPanel.test.tsx src/game/DeckLibrary.test.tsx src/game/DeckAdmission.test.tsx src/game/deckLibrary.test.ts
```

较长的测试超时仅适配当前执行环境的运行速度，不修改仓库测试配置、断言或生产行为。合入前仍需父任务核对最新主分支及三卡后续处理；本地提交不等于已上线。
