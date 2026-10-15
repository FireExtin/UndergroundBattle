# WIP：三项Web超时的串行诊断与最小测试修复（2026-10-15）

原候选 `4918cfb6372fb9ec900f037e8ed518385d6183e6` 的完整Web结果仍为1142通过、3项超时，不被覆盖。获授权后，在原阈值下对三个失败文件做了一次带CPU采样的针对性复现，复现相同三项失败；仅修改三份测试后，以相同参数运行一次，19项全部通过。完整Web及后续检查尚待运行；本分支不可直接推进main或部署。

## 真相源与准确位置

- [原始完整回归失败与停止记录](MAIN4918_VALIDATION_BLOCKED_2026-10-15.md)：`V028DeckEntry.test.jsx:15`、`WM023Integration.test.jsx:16`、`WM038Integration.test.jsx:16`。
- 历史完整通过轮使用相同`--maxWorkers=1`，1145项、298.00秒；失败轮为464.44秒。两轮没有逐时资源采样，不能据此倒推并发争抢或CPU限流。
- 复现前保持4918源码干净。修复分支从仅含原失败证据的 `c9fbaa286144ee4aed620823f9f59d032b64edfe` 开始，未引入JC095后续QA分支。
- 本次没有修改产品源码、内核、WASM、Worker、fixture、配置、测试数量或超时阈值。

## 复现与根因证据

| 测试 | 固定阈值 | 修前采样轮 | 修后相同采样轮 |
| --- | --- | --- | --- |
| V028正常组卡、保存重载与四席入桌 | 20000ms | 20954.54ms，超时 | 3939.18ms，通过 |
| WM023完整牌面 | 5000ms | 5653.88ms，超时 | 447.39ms，通过 |
| WM038完整牌面 | 5000ms | 5261.18ms，超时 | 351.50ms，通过 |

V028修前profile的role查询调用链约15.394秒，其中可访问名称调用链约14.137秒；WASM调用链约0.163秒。测试在完整真实目录上重复用全局`screen.getByRole('button', {name})`计算大量无关按钮的可访问名称。将添加按钮查询限定到目标卡行、保存按钮查询限定到保存控件后，role查询调用链降到约1.061秒，完整正常控件编辑、保存重载、四席入桌断言均保留，完整目录仍然渲染。

WM023、WM038分别对1,317,198和1,240,666字节的两个Buffer执行Vitest `toEqual`。修前profile中深比较调用链分别约4.320和3.772秒；改为`Buffer.equals`后分别约0.013和0.018秒。两项仍读取完整源图与网页图、检查原有固定SHA256、检查完整字节相等，并保留牌面文本、图片URL、交互和Native指令断言。未缩图、抽样、删除断言或扩大超时。

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

修前16通过、3失败；修后19通过、0失败。两轮独立日志、逐项JSON、每秒cgroup采样、六份压缩原始CPU profile及可复算脚本保存在 [evidence清单](evidence/web-timeout-diagnosis-2026-10-15/manifest.json)。[profile摘要](evidence/web-timeout-diagnosis-2026-10-15/profile-summary.json)保留原始未压缩文件SHA256与采样汇总。`analyze_profiles.py`只读取证据，不执行测试，支持直接读取压缩profile。

接着应在固定修复SHA上运行一次不带profiler的完整Web；Worker及其余适用检查单列，不补写为原4918已通过。Native1169是原4918完整回归结果，本次不因测试文件优化而重跑。当前未推main，未部署，也未对线上牌桌发送命令。
