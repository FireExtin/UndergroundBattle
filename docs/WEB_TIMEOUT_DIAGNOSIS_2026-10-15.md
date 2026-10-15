# Web超时的串行诊断与四文件最小修复（2026-10-15）

最终四文件修复 `a15321a91e7b712f4d38dda96676043de04c9868` 的完整Web为121文件、1145项通过，production build dry-run通过；待父会话串行独立复审。原4918的1142通过/3超时、第一次修复00c273的1144通过/WM056超时均保留。产品源码、产物字节及阈值未变化，本次没有推main或部署。

## 真相源与准确位置

- [原始完整回归失败与停止记录](MAIN4918_VALIDATION_BLOCKED_2026-10-15.md)：`V028DeckEntry.test.jsx:15`、`WM023Integration.test.jsx:16`、`WM038Integration.test.jsx:16`。
- 历史完整通过轮使用相同`--maxWorkers=1`，1145项、298.00秒；失败轮为464.44秒。两轮没有逐时资源采样，不能据此倒推并发争抢或CPU限流。
- 复现前保持4918源码干净。修复分支从仅含原失败证据的 `c9fbaa286144ee4aed620823f9f59d032b64edfe` 开始，未引入JC095后续QA分支。
- 本次仅改变V028、WM023、WM038、WM056四份测试中的查询/比较方式；没有修改产品源码、内核、WASM、Worker、fixture、配置、测试数量或超时阈值。

## 复现与根因证据

| 测试 | 固定阈值 | 修前采样轮 | 修后相同采样轮 |
| --- | --- | --- | --- |
| V028正常组卡、保存重载与四席入桌 | 20000ms | 20954.54ms，超时 | 3939.18ms，通过 |
| WM023完整牌面 | 5000ms | 5653.88ms，超时 | 447.39ms，通过 |
| WM038完整牌面 | 5000ms | 5261.18ms，超时 | 351.50ms，通过 |
| WM056完整牌面（另一个独立定向轮） | 5000ms | 5214.51ms，超时 | 338.55ms，通过 |

V028修前profile的role查询调用链约15.394秒，其中可访问名称调用链约14.137秒；WASM调用链约0.163秒。测试在完整真实目录上重复用全局`screen.getByRole('button', {name})`计算大量无关按钮的可访问名称。将添加按钮查询限定到目标卡行、保存按钮查询限定到保存控件后，role查询调用链降到约1.061秒，完整正常控件编辑、保存重载、四席入桌断言均保留，完整目录仍然渲染。

WM023、WM038分别对1,317,198和1,240,666字节的两个Buffer执行Vitest `toEqual`。修前profile中深比较调用链分别约4.320和3.772秒；改为`Buffer.equals`后分别约0.013和0.018秒。两项仍读取完整源图与网页图、检查原有固定SHA256、检查完整字节相等，并保留牌面文本、图片URL、交互和Native指令断言。未缩图、抽样、删除断言或扩大超时。

第一次修复后的完整轮在 `WM056Integration.test.jsx:15` 记录7484.28ms超时；原三项全部通过。WM056第25行仍在深比较1,302,421字节Buffer，源文件与4918一致，属于本轮新暴露的既有比较开销。一次单文件采样复现中，深比较调用链约3.615秒；只改为相同的`Buffer.equals`后约0.007秒。固定SHA256、完整字节检查及其余八项交互/指令/隐私测试全部保留。JC061、JC087、JC094的同类图片为约0.3MB，本完整轮均通过，未扩大到这些测试。

profile为统计采样，类别是包含子调用的时间、互有重叠，不能相加；采样本身会影响耗时。资源记录仅适用于本次复现：CPU配额4核、内存上限16GiB，修前轮没有新增CPU限流或OOM事件。它支持测试侧瓶颈的定位，不证明原完整回归时不存在额外环境因素。

## 最小验证步骤与证据

两个针对性轮均串行运行以下三个文件，使用原Vite配置、`--maxWorkers=1`，未设置`--testTimeout`：

```text
node tools/build-storage.mjs -- npm --prefix web test -- \
  src/game/V028DeckEntry.test.jsx \
  src/game/WM023Integration.test.jsx \
  src/game/WM038Integration.test.jsx \
  --maxWorkers=1 \
  --execArgv=--cpu-prof --execArgv=--cpu-prof-dir=<该轮独立目录> \
  --reporter=default --reporter=json --outputFile=<该轮独立JSON文件>
```

原三文件修前16通过、3失败；修后19通过、0失败。WM056另一个定向轮修前8通过、1失败；修后9通过、0失败，不与前三文件轮相加。独立日志、逐项JSON、每秒cgroup采样、八份压缩原始CPU profile及可复算脚本保存在 [evidence清单](evidence/web-timeout-diagnosis-2026-10-15/manifest.json)。[原三文件profile摘要](evidence/web-timeout-diagnosis-2026-10-15/profile-summary.json)及[WM056摘要](evidence/web-timeout-diagnosis-2026-10-15/wm056/profile-summary.json)保留未压缩文件SHA256与采样汇总。`analyze_profiles.py`只读取证据，不执行测试，支持压缩profile；传入`wm056`证据目录可复算该轮。

## 已完成的独立检查与下一步

以下均为在固定 `00c273d4a0ae4b75ab3f2a66be2e3eca96c75ede` 上实际执行的一次串行检查，原始日志与退出码保存在`verification-00c273/`，不补写成4918或最终四文件修复SHA的重新运行。

| 检查 | 实际结果 |
| --- | --- |
| 完整Web | 121文件，1144通过、WM056一项超时；退出1 |
| 完整Worker | 180通过，0失败/取消/跳过；退出0，700.479秒 |
| 牌面来源guards | 42通过，退出0 |
| fixture guards | 22通过，退出0 |
| P2 CLI契约 | 8通过，退出0 |
| JC046完整parity | 37场景、334步、1468份seat view、55非法状态/110入口检查；完整Native状态相等，拒绝旧76；退出0 |
| production build dry-run | 因Web失败未启动 |

非法状态manifest仍为`bedea2a79fc7cc98a6c1d0ed98a5d166396d638dba07060a91b6436ea5698661`，WASM仍为`b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`。Native1169是原4918完整回归结果，本次未重跑。

## 最终四文件批次的完整验证

以下两项在固定 `a15321a91e7b712f4d38dda96676043de04c9868`、干净工作区串行执行一次，完整Web未启用profiler，使用原阈值及`--maxWorkers=1`。原始日志、逐项JSON与命令结果保存在`verification-final/`。

| 检查 | 实际结果 |
| --- | --- |
| 完整Web | 121文件、1145通过、0失败；退出0，命令耗时379.752秒 |
| production build dry-run | 两个TypeScript配置、Vite生产构建、Wrangler `--dry-run`均通过；退出0，24.875秒；没有发布 |

V028、WM023、WM038、WM056在此完整轮的首项测试分别为3472.03、323.35、356.23、320.52ms，均通过。没有依赖只跑定向用例来补写完整轮，也没有在同一源码上重跑碰运气：00c273失败后新增了已复现的WM056一行比较修复，才验证新SHA。

从4918到a15321，文档之外只有四份测试变化，共12行新增、9行删除。`sites`、`rust-game`、`rust-game-wasm`和`tools`的Git树在4918、00c273、a15321分别完全一致，fixture与测试配置也未变化，见[最终内容身份核对](evidence/web-timeout-diagnosis-2026-10-15/scope-final-candidate.json)。因此Worker180、来源42、fixture22、CLI8、JC046完整parity及原Native1169按未变内容保留，没有重复运行或改写其执行SHA。最终Web树为`08bc0537480cd524a5a6fd113ad12c170fd10755`。

## 本地产物身份与交接边界

重建后，三个WASM均为2,909,006字节，SHA256仍为`b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`；Worker bundle为47,324字节，SHA256仍为`e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`。`web/dist`、`sites/public`、`sites/dist/client`各158个本地文件的路径、长度、SHA256与构建前完全相同，没有增加、删除或变化。[构建后产物核对](evidence/web-timeout-diagnosis-2026-10-15/artifact-identity-after-build.json)和[前端逐文件清单](evidence/web-timeout-diagnosis-2026-10-15/frontend-identity-after-build.json)只证明本地构建身份，不代表一次新部署或线上试玩。

交接前只读远端核对记录在[remote/CI审计](evidence/web-timeout-diagnosis-2026-10-15/pre-handoff-remote-ci-audit.json)：main仍为`3d642f0c1e6e5ac021cd40560086df35638bfe4c`；JC095证据分支仍为`f4754add6d6239b50234394055b8f9c8e80fbc2a`。a15321没有GitHub check run、workflow run或combined status记录，不能宣称CI通过；空statuses返回的`pending`也不是正在执行的CI。

最终SHA仍需父会话串行复审；原4918精确SHA授权不能直接解释为新提交的main授权。本次没有推main、部署、线上游戏命令或新的线上试玩。独立复审与之后的main决策是剩余边界，验证没有通过改阈值、删断言、减少用例或合计不同轮次数量来补齐。
