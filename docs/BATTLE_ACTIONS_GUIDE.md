# 对战动作完整说明（Battle Action Guide）

此文档与前端 `/battle-action-guide.md` 同步，作为维护者与原型测试玩家的动作参考。

## 核心原则
- 前端负责展示和动作提交，不做本地裁决。
- 一切合法性、结算与终局判定以 Go 规则核为准。

## 字段约定
- `Action Kind`：动作类型。
- `Source Card`：动作发起牌（通常是本方可见牌）。
- `Target Card`：动作目标牌（通常是敌方牌或地区牌）。
- `Target Player`：玩家级标记动作使用。
- `Marker Type / Marker Amount`：标记类动作使用。
- `Operation Label`：队列操作的可选标签。

## 动作解释
- `pass_priority`：放弃当前优先权。
- `advance_phase`：推进阶段。
- `reveal_card`：公开卡牌。
- `inspect_card`：检查隐藏卡牌。
- `declare_attack`：角色攻击角色。
- `declare_investigation`：角色调查地区。
- `move_card`：在地区间移动驻场牌。
- `queue_operation`：提交卡牌操作（可入栈或直接结算）。
- `set_marker` / `remove_marker`：玩家标记增减。
- `set_card_marker` / `remove_card_marker`：卡牌标记增减。
- `set_face_down`：设置背面状态。
- `use_first_player_privilege`：使用先手特权。

## UI 交互约定
- 点击本方可见牌：优先自动填入 `Source Card`。
- 点击敌方牌或地区牌：优先自动填入 `Target Card`。
- 手工选择优先级更高，自动填充不会覆盖用户手工输入。

## 日志建议
- 先看 `ActionAccepted` / `ActionRejected` 判断动作是否生效。
- 再看 `StatePatched` 的 phase/priority/score 变化确认局面响应。
