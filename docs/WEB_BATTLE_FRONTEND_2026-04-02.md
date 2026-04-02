# WEB Battle Frontend（2026-04-02）

## 目标
- 在现有 Go 权威规则核基础上，提供一个“可操作的对战牌桌界面”，替代仅调试器风格的主入口。
- 保持规则裁决仍由后端完成，前端只负责动作构造、提交与状态投影展示。

## 本轮落地
- 应用入口切换为 battle shell：
  - `web/src/app/AppShell.tsx`
- 新增 battle 数据层与界面层：
  - `web/src/battle/model.ts`
  - `web/src/battle/components/BattleTable.tsx`
  - `web/src/battle/components/ActionComposer.tsx`
  - `web/src/battle/BattleShell.tsx`
- 视觉结构改为牌桌语义：
  - 对方玩家区域 / 争夺区 / 本方玩家区域
  - 牌库、手牌、墓地、计分区、秘社标记、地区槽位
- 动作提交支持（最小可玩集合）：
  - `pass_priority`
  - `advance_phase`
  - `reveal_card`
  - `inspect_card`
  - `declare_attack`
  - `declare_investigation`
  - `move_card`
  - `queue_operation`
  - `set_marker` / `remove_marker`
  - `set_face_down`
  - `use_first_player_privilege`
  - `set_card_marker` / `remove_card_marker`

## 协议补充
- 为地区/卡牌分组显示，`CardView` 投影新增字段：
  - `kind`
  - `regionCardId`
  - `regionOrder`
- 同步修改：
  - `server/pkg/rules/projection.go`
  - `server/pkg/rules/projection_test.go`
  - `web/src/debugger/protocol.ts`

## 测试与试玩
- 单元测试新增：
  - `web/src/battle/model.test.ts`
  - `web/src/battle/BattleShell.test.tsx`
- Playwright smoke：
  - `web/playwright.config.ts`
  - `web/tests/battle.spec.ts`
- npm 脚本新增：
  - `npm run test:e2e`
  - `npm run test:e2e:headed`

## 运行方式
1. `go run ./server/cmd/api`
2. `cd web && npm run dev`
3. 打开 `http://127.0.0.1:5173`

或直接执行 e2e（自动拉起双服务）：
- `cd web && npm run test:e2e -- tests/battle.spec.ts --project=chromium`

## 当前边界
- 仍属于“基础包优先”的最小可玩前端，不含扩展包特有视觉/机制。
- 界面已可操作，但不是最终美术化客户端（卡背/插画/复杂动画未接入）。
- 规则合法性、结算、终局判断依然全部以后端投影为准。
