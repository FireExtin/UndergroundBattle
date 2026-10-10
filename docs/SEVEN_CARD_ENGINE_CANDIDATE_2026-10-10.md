# 七牌引擎候选：排除未裁定 XQ18

本次仅收口前批，不追加卡牌。直接父提交为 `0cd1a8083a34bc87508c3217c37a972d0f8f3147`；该提交的精确父提交及已审起点为 `37d05bc09aeaaea409dcd04b2d25f0c1f4e89a0e`。协作分支保持 `codex/jz48-combined-review-20261007`。未推送或发布，不改 UI、生产 WASM pkg、Worker、D1。

生产候选只含 JZ30、BQ028、BQ040、WM059、BQ078、JZ44、JZ45 七个明确完整定义（六个新牌名及鲜血的新版本）。卡池 127 条，红8／蓝13／黑21／灰9；原120条、默认牌组、世界牌与来源哈希保持原值。候选 engine 为 `rust-v0.2.59-seven-card-engine-candidate`，pool 为 `limited-v2.54-seven-card-engine-candidate`。旧58候选和旧57局都不静默迁移。

## XQ18 排除与保留

XQ18 的卡池记录、typed Definition、专用 TargetPredicate、两个 Op 和解释器处理全部从运行路径移除。现行目录不展示计时人，构筑及 `Game::new_with_deck` 拒绝该 ID，既有 `make_card` 准入检查在分配实例前拒绝，Game/Room 存档中的 XQ18 实体和旧操作／目标枚举均拒读。没有将未裁定效果换成空能力。

XQ18 原型源码、20项原测试和字段记录保存在父提交0cd1a80及其 Library 旧版审查包；新版包另含 `excluded-xq18-analysis` 原始材料。`RED_TIME_CANDIDATE_2026-10-10.md` 保留历史分析，不能作为当前准入清单。宿主自身0而附属1时能否经宿主移除，以及多个物理载体有标志时由谁选择，仍由父端裁定。

共享时间状态护栏、合法附属继承、JC045势力计数、JZ30离场前聚合时间冻结及死亡预测按已明确条文保留。JZ30测试的红色付款资产改用已准入JZ30，附属时间的转结／死亡测试使用明确的已有标志布局，不依赖被排除的计时人。测试没有忽略项；原八项XQ18原型执行回归保留于旧包，当前增加真实入口排除及JZ30声明／堆栈篡改回归。

## 精确规则依据

- `resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf`：术语表“标志X”“触发能力”“公开”“预测”；P11指示物标志及附属标志视作位于宿主、离场／翻面清理。JZ30采用可选、可响应的进场能力，死亡时冻结实际时间；无需XQ18移除规则。
- `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf`：当前团队、邻接移动及对抗规则，沿用已审机动程序和有限公开触发时点。
- `resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf`：对照现行印刷、附属与结算；未找到计时人实际标志载体选择裁定。
- 七张完整扫描：`resource/ymsj-fun.github.io/cards/{JZ30 末世论者,BQ028 水沟鼠群密探,BQ040 鲜血,WM059 雨夜屠夫,BQ078 专业清理员,JZ44 洛杉矶巡警,JZ45 警用直升机}.jpg`。鲜血另对照已准入 `XQ14 鲜血.jpg`。
- 原资源 `resource/ymsj-fun.github.io/cards/cards.json` 与 `docs/factions/card-specifications.json` 是字段／条文台账，均未改变研究准入标记。蓝、黑、灰逐牌说明仍在本目录对应候选文档。

新版审查包包含以上实际文件、逐文件 SHA-256 和关联路径，审查者无需共享本地路径。新的专项、workspace与WASM原始记录在新版包单列；旧版两轮测试继续按两轮记录，绝不合称单次全绿。开发回归不替代独立审查或真实多人试玩。
