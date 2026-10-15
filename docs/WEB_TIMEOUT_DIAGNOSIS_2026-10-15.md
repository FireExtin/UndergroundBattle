# WIP：四份测试中Web超时的串行诊断与最小修复（2026-10-15）

原候选 `4918cfb6372fb9ec900f037e8ed518385d6183e6` 的完整Web结果仍为1142通过、3项超时，不被覆盖。原三项在定向采样轮复现并修复后，`00c273d4a0ae4b75ab3f2a66be2e3eca96c75ede` 的完整Web为1144通过、1失败，新增WM056同类大图片深比较超时。随后对WM056单独复现并最小修复，修后9项通过。最终改动为四份测试；最终四文件修复SHA的完整Web及构建尚待运行。本分支不可直接推进main或部署。

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

接着固定四文件修复SHA运行一次不带profiler的完整Web；通过后才执行production build dry-run。Worker、来源、fixture、CLI与parity输入均未变化，以上结果按内容身份保留，不重复运行。最终SHA仍需父会话串行复审；原4918精确SHA授权不能直接解释为新提交的main授权。当前未推main，未部署，也未对线上牌桌发送命令。
