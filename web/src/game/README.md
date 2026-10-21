# React玩家客户端

默认入口`/`提供大厅、命名牌组、邀请入席、牌桌与对局管理；旧Go调试器和`/legacy-debugger`已经退役。完整启动、当前卡池与验证状态见[根README](../../../README.md)。

## 本地开发

从仓库根目录执行：

```sh
npm --prefix web ci
npm --prefix web run dev -- --host 127.0.0.1
```

Vite将`/api`代理至`127.0.0.1:8090`，须另行运行Native服务。`npm --prefix web run build`执行TypeScript检查与Vite构建；`npm --prefix web test -- --maxWorkers=2`运行Vitest。许多组件测试读取实际当前WASM及精确历史内核，先按根README准备输入。jsdom组件测试不是浏览器试玩。

## 权威与恢复

- [useGame.ts](useGame.ts)使用认证HTTP轮询；前台正常间隔约1.5秒，后台降频，恢复可见时同步。已确认暂停后停止轮询，可手动刷新。它不调用SSE主循环；[api.ts](api.ts)仍保留`streamEvents`作为已有接口辅助。
- 本机保存席位凭据及草稿，不保存可作为权威的完整Game。刷新时请求所属房间的本人View和目录；旧桌目录来自旧内核，不能由新全局目录覆盖。
- 原始未确认命令按房间、席位和commandId分别保存，重试携带同一意图；结果不确定时阻止新操作。不要把历史`hegemony.pending.v1`单条记录当作当前完整协议。
- `legalActions`、响应报价、费用、有效目标、即时图标和胜负均来自服务端。客户端只选择、提交并解释返回结果，不按牌面自然语言补造动作。
- 私密`pendingChoice`只给本人；`waitingChoice`显示全桌当前选择者。暗牌按投影授权展示，队友不因此获得其牌面。

## 交互入口

| 模块 | 范围 |
| --- | --- |
| [deckLibrary.ts](deckLibrary.ts) | 命名牌组草稿、目录准入提示、存储与版本信息；最终构筑由服务端校验 |
| [ChoicePanel.tsx](ChoicePanel.tsx) | 一般选择、排序、调查、伤害分配及明确允许的空确认；选择草稿不替代服务端结果 |
| [ResponseWindow.tsx](ResponseWindow.tsx) | 服务端响应窗口、报价和提交意图；不以客户端倒计时决定最终到期 |
| [RoomManagement.tsx](RoomManagement.tsx) | 暂停/恢复、刷新、退出/重开等现有房间操作 |
| [AutoPass.tsx](AutoPass.tsx) | 用户可关闭的自动让过偏好及状态；实际提交仍经统一命令确认路径 |

牌面、目标和选择中的实例标识用于当前交互，不能将离场前的旧实例当作新牌。卡图存在并不代表已开放；公开目录与牌组准入提示分别说明可组牌/只读/未实现范围。

布局与选择回归在同目录的`*.test.tsx`、`*.test.jsx`中。主要QA标识包括`data-card-instance`、`data-action-id`、`data-choice-id`等；它们不是接口稳定性或完整浏览器对局的保证。新增规则需同时核验服务端命令链，不能只凭组件显示判断规则正确。
