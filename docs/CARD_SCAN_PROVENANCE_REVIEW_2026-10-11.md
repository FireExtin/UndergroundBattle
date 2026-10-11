# 已入池卡图来源修复：精简审查补丁

意图：`user_requested`。固定基线为 `14b7b5e06a0b1a3649f2a3f947ad515236ebdfd9`；本补丁选择性提取已独审通过的 `327dc8db49e3ff650ac6236c90903b2ba4e9ec00`，只保留三份原字节文件及此说明。完整库存、原始日志、Native 输入和 WASM 证据保留在本地交付包，没有加入本精简分支。

## 改动及真相源

- `web/public/card-scans.json`：补 XQ44 随风入梦（固定B14）及 JZ02 西姆斯教授（固定B22）已入池卡的来源清单；修正 JZ22 暗夜游掠者的原图仓库路径。
- `tools/cards/validate-admitted-scans.mjs`：只读核验 `cards.json` 的128普通定义，检查固定规格、来源URL/路径/SHA、客户端映射，以及公开图和来源原图逐字节一致；秘社和历史art-only ID不在该工具范围。
- `tools/cards/admitted-scans.test.mjs`：来源闭环测试及生产来源路径错改的内存负例。

原图位于 `resource/ymsj-fun.github.io/cards/`；公开 JPEG 和 `cardScans.ts` 原已齐全，没有复制或改写图片。来源清单137→139项，仅上述三项变化；其余136项及运行卡池、规则、版本保持原值。没有新增卡牌，没有修改引擎或前端交互。

## 验证及边界

先实际复现来源清单缺项/路径错误，再修复；另以JC125来源路径内存变异复现漏检，补守卫后Node两项通过。可复跑：`node --test tools/cards/admitted-scans.test.mjs` 和 `node tools/cards/validate-admitted-scans.mjs`。独审确认本分支三份功能文件与327dc8d逐字节相同。

原始327dc8d范围的既有专项验证：Native 17通过、822过滤；React 9通过；生产WASM与当前Native共1,279命令、5,316视图及28非法状态的对比通过。没有声称全workspace回归或真实浏览器验收。完整证据包保留在 `/workspace/card-admission-preparation-327dc8d-20261011.tar.gz`。

BQ030裁定官的规则来源和下一批计划已准备，但结附观察事件和共享调用入口仍待父端协调，本分支不包含其实现。三项P1和真实浏览器验收仍由父端统筹。本分支只供审查；没有合入main或发布。
