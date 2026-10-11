# 终局顶部摘要候选修复（2026-10-11）

源码基线：`764d49462a41ff00a7c29725006e125c1e76a985`。本候选仅修复父会话实际试玩后报告的终局顶部摘要问题，供独立审查与父端再次浏览器确认。

## 问题与修复

`Table.tsx` 的顶部摘要原先先检查选择及合法动作；终局房主仍有合法的“再开一局”动作，因此摘要误报“轮到你参与行动”。其他玩家无合法动作时，摘要按保留的优先权误报“等待 A 行动”。

在原摘要表达式前增加 `view.status === 'finished'` 判断，终局统一显示“对局已结束”。生产代码改动仅此一行。胜利横幅、比分、重开按钮、动作权限、游戏规则及牌面均使用原有逻辑。

## 新执行证据

运行环境：Node `v24.19.0`；复用现有依赖与构建缓存。

1. 修复前，仅添加 `FinishedSummary.test.tsx` 后执行 `npm test -- src/game/FinishedSummary.test.tsx`：6 项失败、2 项通过。6 项失败均是终局摘要仍显示行动或等待；2 项进行中摘要测试通过。
2. 修复后，在 `web/` 执行 `npm test -- src/game/FinishedSummary.test.tsx src/game/EmptyRegionSlots.test.tsx src/game/OrdinaryTargetRetention.test.tsx src/game/AutoPass.test.tsx`：4 个测试文件、58 项全部通过，其中新增终局摘要测试 8 项。
3. 在 `web/` 执行 `npm run build`：`tsconfig.app.json`、`tsconfig.node.json` 类型检查及 Vite 生产构建通过。

新增测试覆盖胜方房主、另一位玩家、无匹配座位且无手牌/合法动作的只读观察视角；覆盖终局首次显示、进行中到终局更新及 JSON 投影重新挂载后的显示；验证胜方标题及双方比分，并保留进行中摘要与房主合法重开行为。

这些测试只验证组件展示。观察视角为合成投影，不新增观战接口；重新挂载不等于浏览器刷新时的会话/座位持久性验证。未重复或代替父会话真实试玩，不宣称本候选已通过独立审查或父端验收。

## 本地日志指纹

日志存于独立工作区 `/workspace/qa-finished-summary-20261011/`，不纳入源码提交。

| 日志 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `red.log` | 5108 | `d25b3a2ec0876af095bebed86bcf8529de4a3b09be4be373f35944f18f4f6fac` |
| `ui-green.log` | 622 | `d2add5c42438e3afaf529e339732a7257c9c8b6d085fcd11a753bee7e5729968` |
| `build.log` | 689 | `fca457f8a23ed7be61bb4fc7ac52dc1f40b2e425ddc4dafd2276b013bbb3dc47` |

## 工作区与交付边界

修复前分支为 `codex/xq24-riot-police-review-20261011`，HEAD 为上述源码基线。未发现活动源码编辑、额外工作树或项目 `AGENTS.md`/技能分工文件。本候选使用独立普通审查分支 `codex/ui-finished-summary-review-20261011`；不创建 worktree，不改变原审查分支或 main，不发布网站。

原有未跟踪的 `docs/MAIN_764D494_QUALIFICATION_INDEX_2026-10-11.md` 保留且排除本提交：25585 字节，SHA-256 `689482da064eb0c63b9f10aae6c0e155ffc9bc992af71608223587e76a492f5b`。本任务未操作 Air，未改资格索引，未运行与此展示修复无关的 Rust/WASM/Worker 重构建。
