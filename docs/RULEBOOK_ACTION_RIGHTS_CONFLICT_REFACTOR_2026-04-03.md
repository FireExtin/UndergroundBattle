# Rulebook Action Rights / Conflict / Ability Refactor（2026-04-03）

## 本轮目标

把 battle 主流程从“`main/end + 少量直接动作` 的原型模型”推进到更接近规则书的显式状态机，优先解决这几类长期漂移问题：

- 行动权没有按规则书步骤建模
- 对抗阶段仍依赖 `declare_investigation` / `declare_attack`
- 暗藏者现身缺少正式动作入口
- 调查奖励和战斗奖励没有 prompt 模型
- 调查奖励的私有信息会跨投影泄漏
- 前端无法明确区分“本方已暗置但可自知”的牌

## 已落地设计

### 1. `main` 拆成规则书两段行动步骤

- `TurnState.Phase.Step` 新增：
  - `first_player_action`
  - `second_player_action`
- 双方空栈连续让过后，不再直接跳整相位，而是：
  - 第一行动步骤结束 -> `advance_phase` 进入 `second_player_action`
  - 第二行动步骤结束 -> `advance_phase` 进入 `conflict`
- 与旧 battle shell 的兼容仍保留：
  - 直接在未结束步骤上调用 `advance_phase` 会走“跳过剩余流程”的兼容路径

### 2. `conflict` 成为正式 phase

- `TurnState.Phase.Name` 新增 `conflict`
- `TurnState.Conflict` 新增：
  - `regionOrder`
  - `regionCardId`
  - `stage`
  - `priorityLeaderPlayerId`
  - `pendingPromptId`
- 当前已实现的 stage：
  - `pre_investigation_fast`
  - `investigation_reward_prompt`
  - `post_investigation_fast`
  - `pre_battle_fast`
  - `battle_damage_prompt`
  - `post_battle_fast`
  - `pre_influence_fast`
  - `post_influence_fast`

### 3. 三类对抗开始按当前规则常量结算

- 调查对抗：
  - 只统计本地区未横置、正面朝上的角色调查图标
  - 暗藏者不参与
  - 不再横置参与角色
- 战斗对抗：
  - 只统计本地区未横置、正面朝上的角色战斗图标
  - 暗藏者不参与
- 势力对抗：
  - 只统计本地区未横置角色
  - 正面角色按 `EffectiveStats.Influence`
  - 暗藏者固定按 `1` 参与

### 4. Prompt 模型落地

- `TurnState.PendingPrompt` 成为强制交互真相源
- 已支持两类 prompt：
  - `investigation_reward`
  - `battle_damage`
- 新动作：
  - `resolve_prompt`
- 调查奖励流程：
  - 检视数量 = 差额
  - 选择哪些牌回顶、哪些牌入底
  - 最后恒抓 `1` 张
- 战斗奖励流程：
  - 赢家按差额分配伤害

### 5. 现身与行动能力有了正式动作入口

- 新动作：
  - `reveal_face_down`
  - `activate_ability`
- `reveal_face_down`
  - 检查本方暗藏者、资源与忠诚
  - 进入堆叠
  - 结算后翻正，保留原横置状态
- `activate_ability`
  - 第一版走显式 `AbilityRegistry`
  - 已录入 `JC003.quick.exhaust_target`
  - 支持资源费 + 横置费
  - 进入堆叠后结算

### 6. Prompt 私有信息不再泄漏

- projection 新增 viewer-aware turn projection
- `PendingPrompt.PeekCardIDs` / `EligibleTargetIDs` 只对提示拥有者可见
- 对手与观众仍能看到“当前存在一个 prompt”，但看不到调查检视牌

### 7. Web battle UI 已同步

- 顶栏新增：
  - `phase/step`
  - `conflict stage`
  - `priority window`
  - `priority leader`
  - `pending prompt`
- 牌桌上的暗藏牌现在有明确文案：
  - 本方可见暗藏牌显示“`（暗藏）`”
  - 非拥有者显示“暗藏者”
- 动作面板新增：
  - `reveal_face_down`
  - `activate_ability`
  - `resolve_prompt`
- 调查奖励 / 战斗伤害分配现在都有专门 prompt UI，而不是继续走旧的普通动作表单

## 本轮明确保留的降级点

- `declare_investigation` / `declare_attack` 仍保留在 battle composer 中作为调试入口，还没有完全从 UI 移除
- `activate_ability` 目前只接了显式注册表的第一张代表卡，不会自动解析全文本卡面
- 先手标志特权还没有完全并入新的 conflict tie prompt 流程，这轮先保留旧动作路径
- `capabilities` 尚未作为正式 projection 契约下发；前端当前仍是“prompt 专用分支 + metadata 预校验 + 服务端 authoritative legality”的组合态

## 测试护栏

本轮新增/更新的重点测试覆盖：

- `server/pkg/rules/rulebook_conflict_and_ability_test.go`
  - 主行动两步骤推进
  - 调查对抗不横置参与者
  - 调查奖励 prompt 打开 / 排序 / 抓 1
  - prompt 私有信息隔离
  - `reveal_face_down` 入栈并保留横置态
  - `activate_ability` 费用与入栈结算
- `server/pkg/rules/testdata/m0/*.json`
  - 基线步名改为 `first_player_action`
- `web/src/battle/BattleShell.test.tsx`
  - battle shell 与动作面板回归继续通过

## 验证命令

```bash
go test ./server/... -count=1
cd web && npm test
cd web && npx playwright test tests/battle.spec.ts --project=chromium --config=playwright.local.config.ts
```

注：

- 默认 `playwright.config.ts` 会自己拉起 API 和 Vite；若本机已有 `:8080` API 占用，可用 `playwright.local.config.ts` 复用现有 API，只单独启动前端。
