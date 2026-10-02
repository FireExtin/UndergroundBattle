# 全部可读来源核验交付

2026-10-03。独立分支 `codex/faction-coverage-audit`；只供主实施/父线程集成，无发布。源码支持结论固定于 `286ad83ef5a54bc0caed26199ca4cfea269314bd`，该提交是本分支祖先。不得把本审计状态当作之后主实施分支的实时支持状态。

## 完成范围

| 685条定位记录的最终分区 | 数量 | 含义 |
| --- | ---: | --- |
| 完整牌面规格 | 678 | 同ID整图及必要放大区直接人工核验，原路径/SHA、名称、印刷代码、颜色、类型、费用、忠诚、领域、永久/临时图标、防御、关键词及正文保存 |
| 完整非卡牌参考 | 1 | TK007原图是先手钥匙图，不虚构印刷卡名/费用/编号；功能由原规则手册P9/PDF11核准 |
| 按用户要求暂置的原字段疑点 | 5 | 原图已看，只有下表指定字段阻塞，不能标整卡完整 |
| 同ID原图缺失 | 1 | TK011继续隔离，不用别的触手版或同名图替代 |
| 可读、有图但待核 | 0 | 原图最低核验阶段已完成 |

完整来源合计679，不是685张可玩牌。12张原缺图及1份截断图恢复自公开原资料仓库固定提交 `5d72e3207860b7aaa1eec50b81bb4f9b8a697cb7`，独立副本在 recovered-originals；原resource镜像未修改。详见 [source-recovery.json](source-recovery.json) 与 [B16R-source-recovery.md](B16R-source-recovery.md)。

| 暂置记录 | 唯一阻塞字段 |
| --- | --- |
| MSJC08 | 副名称字形 |
| MSJC09 | 副名称字形 |
| JC108 | 副名称字形 |
| DQWM001 | 副名称中间字形 |
| MSWM04 梦魇主母 | 额外印刷符号中的数字8及其图形含义；是“一个带8的额外符号”，不是“八个符号”。起始手牌2另已核准 |
| TK011 | 同ID原图缺失；本次固定原资料树搜索仍未找到 |

`card-specifications.json.openQuestions`登记56项字段、版次、模式及规则设计疑点，不等于56张牌面仍未查看。可读原文和设计裁定分开：例如WM081名称“井中厉鬼”与正文“井中尸鬼”、XG32正文“梦魔主母”与MSWM04名称“梦魇主母”均按原印刷保存，不能自动统一成同名；BQ061正文未明确墓地取回数量，BQ065的X未明确关联护盾数量，均不猜测机制。

## 主线程入口

- [README.md](README.md)：核准八种基本颜色及褐色中立、五种魔法领域、38条秘社记录、模式及原始页码/卡图证据。扩展秘社和徽记变化不增加基本颜色数量；绿色王廷叶片、红色破茧者三角见 [emblem-variants.json](emblem-variants.json)。
- [faction-coverage.json](../../rust-game/data/faction-coverage.json)：适合只读UI导航的颜色/秘社/预组概念与53项共享机制状态；原图证据和基线源码锚点随每项关联。
- [card-specifications.json](card-specifications.json)：678张完整规格、5条blocked规格、1条非卡牌参考及待执行验收场景。各批原页/原图核验说明见B01–B27文档。
- [card-research-index.json](../../rust-game/data/card-research-index.json)：全部685条，locator字段只用于定位；核准字段和原图版本独立，含同名/形态/测试观察版边界。
- [implementation-plan.md](implementation-plan.md)、[acceptance-groups.json](acceptance-groups.json)：按共享机制小步补全及每组真实UI验收门槛。
- [urgent-playtest-rulings.md](urgent-playtest-rulings.md)、[deck-proposals.json](deck-proposals.json)：此前交付的重复切尔诺贝利、JC125零费用合法窗口及四份无中立50张混搭设计；只是有源证的设计输入，不是已开放牌组。

基本快速两色搭配是8色任意两色各25张，组合数28。一般自组是至少50张，并受秘社实际限制和同名张数限制；不是只能两色。当前五份自定义50张预组不是官方五派系。导航须把颜色、官方秘社和运行时可选预组分开；秘社颜色不能自动替代其构筑文本，绿色王廷尤其不能统称猎魔人。

基线生产池只有25个玩家定义和4个世界定义。53项机制审计为10项有限行为implemented、8项partial、33项unimplemented、2项uncertain；每项的承诺范围以数据为准。现有29个卡级定义均标partial，登记存在不表示全原作规则等价。白/紫玩家牌数为0；所有新增官方导航、秘社、任意配对、研究牌及设计预组在本数据中保持禁用。生产可选牌组继续取 `/api/catalog.decks`。

建议次序：事件/费用/选择与触发 → 临时图标/数值/关键字 → 秘社及构筑 → 附属/封印/拥有者与操控者 → 多区域能力/牌库 → 时间/生成/形态 → 隐秘对抗/计划 → 模块化产品与模式。每次只实现一组共享能力与代表牌，做引擎、持久化、合法动作、私密投影和实际UI试玩，再由主实施负责人开放。

## 验证与交付边界

最终执行并通过：

```sh
python3 tools/factions/build_research_index.py --check
python3 -m unittest discover -s tools/factions -p 'test_*.py'
python3 tools/factions/validate_coverage.py
git diff --check
```

78项独立Python回归检查通过，685记录、原图哈希、证据引用、生成可复现、八色/中立、38秘社、当前有限池及禁用边界均通过验证。人工核验的文字正确性仍不能由哈希测试证明；B23的图标类别及交付前名称误读已在放大区复查中更正并记录。测试不表示Rust游戏规则验收通过。

没有运行Rust或实际UI/策略试玩，没有创建或操作线上房间，没有改共享engine/rules/resolution/catalog、React、现有cards.json，也没有发布、合并或写主实施分支。已修改路径仅在 `docs/factions/`、`tools/factions/` 及两份独立research/coverage数据内。

主线程可从私有远端读取并cherry-pick本分支自上述基线后的独立提交；如已接收前几批，仅取缺少的提交。最终远端HEAD由本线程最终回复给出，不把未核实SHA写入交付文档。若主线程源码已变化，严格验证会报告快照漂移；须重审受影响支持标签。`--allow-code-drift`仅供读警告，不是重新核验或开放许可。
