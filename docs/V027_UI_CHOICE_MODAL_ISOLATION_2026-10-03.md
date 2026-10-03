# v027 起手及选择弹层：背景隔离、嵌套阅读与焦点恢复

输入：父线程在公网四席自然验收中发现起手选择弹层的可访问性问题，随后明确授权本地修复。直接切独立会话的本地提交 `5ff8fe2fa1893dbb2cf2a554f5dfb4cb89364c53` 是本次基线；公开实施基线仍为 `97d6c82dc920e0b7f7cc10141e1831b2f055cb84`。输出位于 `codex/v027-ui-qa-fixes-local-20261003` 的同一独立 UI 批次，本次不推送、不部署、不进入父线程现场房。

## 改动与事务语义

从已有 ReadModal/Help 提取局部 `useDialogFocus`，由 Table/GameApp 的现有 React 所有权决定唯一活动层，不增加全局弹窗栈或 provider。

- Table 的本人 pendingChoice 使用 dialog/aria-modal；沿当前层祖先隔离其它兄弟分支，以 `inert` 禁止实际背景点击/聚焦，并以 `aria-hidden` 隔离可访问树。结束后恢复各分支原先的属性值。父分支新增或替换时继续隔离。
- 活动层管理初始焦点、正反 Tab 与越界 focus。选择暂停给嵌套阅读时只释放隔离，不把焦点退回牌桌；阅读成为最上层，关闭后恢复原阅读按钮并保留选牌状态。指南开启时 Table 的选择和阅读暂停，覆盖同步到来的选择及牌桌元素替换。
- 选择层消费 Escape，不取消、不提交、不推进。阅读和指南继续用 Escape 关闭。保留全部手牌仍明确提交 `selected: []`；确认选择仍使用原 server-authored action 和现有 buildChoiceAction。
- 他席 waitingChoice 继续允许正常浏览。ChoicePanel 单独嵌入时默认保持原先非模态行为，仅 Table 明确启用模态。

生产改动只在 GameApp、Table、ChoicePanel、ReadModal、Help 与新增焦点 hook。未修改规则、AutoPass 自动提交判断、存储、网络、服务端或卡池。

## 红绿与真实浏览器证据

`docs/evidence/v027-choice-modal-2026-10-03/before-fix.txt`：先添加 5 项回归，原代码 4 失败/1 通过；首项直接证明背景 AutoPass 仍在可访问树。旧三视口 Tab 越界证据保留在上一提交的 mulligan-probe/result.json。

最终前端默认集：20 文件、177 项通过；TypeScript/Vite 构建通过，git diff --check 通过。两条旧断言曾在阅读打开期间查询背景按钮；现在先明确断言背景不可访问，再正常关闭阅读并验证原内容/属性，未删除原恢复或隐私断言。中间失败日志也保留，避免将断言适配当成新的生产缺陷。

`browser-probe/` 包含 localhost 合成组件夹具、脚本、JSON、运行日志及 6 张截图。Chromium 在 React.StrictMode 下以 1536×960、1366×768、1280×800 三视口进行真实点击/键盘操作，每视口验证 42 次选择 Tab、24 次阅读 Tab、18 次指南 Tab；CDP 可访问树中只有当前最上层 dialog，背景无 Switch。验证零选保留、明确选牌确认、两种阅读关闭方式、已选项不丢、焦点回原阅读按钮、选择 Escape 零提交、指南期间迟到选择以及结束后全部 inert 清除。浏览器无 pageerror。脚本评价仅用于读取几何/焦点/可访问树和断言，不注入状态或代发游戏 API。

实际查看了原始 JC125《无知路人》牌图及 1366×768 嵌套阅读截图；图像 SHA256 为 `6392c2cdc2d9ea43deca6b26594618a2093ae887bd03d5ad29d130c7e329993a`，来源为既有公开基线牌图，无新增卡。

## 保留与验收边界

`preservation.json` 重新从 Git 对照公开实施基线：252 个 Rust/server/WASM/D1/QA 工具/公开素材文件逐字节一致，六旧核加 v027 七核仍完整。

Site 源码 checkout 保持 `8b262c7c97c76d4c296ba8ed0dfbb6a04b2a1d19` 且 clean；另一本地卡牌批次 checkout 保持 `618065cc578fdabd31166c8e16e85b97b66edd6d` 且 clean。本次未作任何 Sites 保存、上传、部署或公网房间动作。

本地合成组件浏览器验证不替代公网自然游戏验收。父线程继续拥有已开四席现场桌与最终 UI 审核/统一发布；待该本地批次审阅并获发布指令后才能进入线上验收。本次浏览器和临时 HTTP 服务已经关闭。
