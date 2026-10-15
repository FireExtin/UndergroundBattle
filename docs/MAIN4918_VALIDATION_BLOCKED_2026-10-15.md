# WIP：4918固定候选的main验证被Web完整回归阻断（2026-10-15）

固定候选 `4918cfb6372fb9ec900f037e8ed518385d6183e6` **没有推送到main**。本轮一次串行完整验证中，Native通过1169项；Web通过1142项、失败3项，均为超时。队列在Web退出码1处停止，没有继续Worker或构建，没有修改候选或重跑测试，也没有部署。

## 输入与选择范围

- 获授权的固定候选：`4918cfb6372fb9ec900f037e8ed518385d6183e6`，树 `3678ce1106d2001a62c96dc7a3d493a3a3b4e6b2`。
- 远端main：`3d642f0c1e6e5ac021cd40560086df35638bfe4c`。它是候选祖先，候选尚不是main祖先；完整范围为28个提交、504个变化文件。
- 已完成的线上JC095与回执核对保存在独立分支 `codex/sites54-jc095-gap-closure-review-20261015@f4754add6d6239b50234394055b8f9c8e80fbc2a`；其后续9个QA提交未选入main候选，本次未改动该分支。
- 测试开始时，三个现存WASM均匹配已审查的 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`；现存Worker bundle匹配 `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`。
- 候选Web树 `d2f2c681476749dd1b63e1e4553ceac2989fd77b` 与此前记录的1145项通过树一致。历史通过结果不替代本轮失败结果。

完整提交清单、文件差异、输入身份和原始日志保存在 [evidence目录](evidence/main4918-validation-2026-10-15/manifest.json)。本提交仅新增验证说明与文本证据。

## 本轮结果

| 检查 | 实际结果 |
| --- | --- |
| `node tools/build-storage.mjs -- cargo test --workspace --locked` | 一次完整运行，退出0；1169通过，0失败、0忽略、0过滤；库测试1102项、其他集成67项 |
| `node tools/build-storage.mjs -- npm --prefix web test -- --maxWorkers=1` | 一次完整运行，退出1；1142通过、3失败，共1145项；118个文件通过、3个失败，共121个文件；Vitest耗时464.44秒 |
| Worker完整回归、牌面来源guards、fixture guards、P2 CLI contract、JC046完整parity、production build dry-run | 均未启动，不能标为通过 |
| GitHub checks / workflow runs / combined statuses | 均为0；combined status返回`pending`且statuses为空，不能解释为正在运行或已经通过的CI |

原始记录：[Native日志](evidence/main4918-validation-2026-10-15/native-full.log)、[Native结果](evidence/main4918-validation-2026-10-15/native-result.json)、[Web日志](evidence/main4918-validation-2026-10-15/web-full.log)、[串行停止结果](evidence/main4918-validation-2026-10-15/remaining-results.json)。日志逐字节保留并核对SHA256；Native原始日志的末尾空行也未裁剪，其余新增文件通过空白检查。

## 三项失败及只读分析

| 文件 / 测试 | 日志所报耗时 | 超时阈值 |
| --- | --- | --- |
| `web/src/game/V028DeckEntry.test.jsx:15`：正常控件编辑九张v028卡、保存重载及四席入桌 | 29496ms | 显式20000ms |
| `web/src/game/WM023Integration.test.jsx:16`：完整WM023牌面与付费横置能力 | 7388ms | 5000ms |
| `web/src/game/WM038Integration.test.jsx:16`：完整WM038四条规则与鲜血身份 | 5101ms | 5000ms |

三项日志都只指向测试回调入口，没有逐步计时，也没有具体断言不一致记录。当前不能据此认定业务缺陷，也不能认定只是环境慢。

WM023第24行、WM038第23行都对两个完整图片Buffer执行Vitest `toEqual` 深比较。另行只读字节核对确认源图与网页图相同：WM023为1,317,198字节，SHA256 `b209fe484e37b930ce4d64be078bd3e72a10bdff5250bfd38ba97cb7648cee85`；WM038为1,240,666字节，SHA256 `847dabd861d6350ae21df04ac548a975e99a7cfd4814f213444409d4c42d7acd`。深比较是可排查的耗时位置，**尚未通过性能采样证明是超时原因**。

V028回调依次进行12种卡的添加和数量编辑、边界校验、保存重载、四席本地内核入桌及开局选择。组件会在编辑时筛选并渲染目录，测试多次使用无作用域的`screen.getByRole`。这些是静态排查线索；没有证据定位到某一步，也没有更改选择器、缩小业务覆盖、提高超时或过滤重跑。

## 停止边界与剩余工作

2026-10-15 14:50:46 UTC只读复核时，main仍是 `3d642f0c1e6e5ac021cd40560086df35638bfe4c`，候选工作区干净。main无必需status context或branch rule；本轮未修改任何权限。无CI结果不豁免本地失败门禁。见[停止审计](evidence/main4918-validation-2026-10-15/failure-stop-audit.json)。

后续应由父会话串行安排三项超时的定因及必要修复审查，再决定验证范围。若源码或测试发生变化，新提交不能继承本次固定SHA的授权或被称为已经验证。当前Worker、其余检查与构建仍待完成，main推进被阻断；不应以历史绿色、局部运行或本地浏览器结果补齐。

本次没有main push、merge、测试失败后的重跑、线上游戏命令、Sites保存/发布或exchange部署。原有JC095结果及Site版本未在本任务中改变。
