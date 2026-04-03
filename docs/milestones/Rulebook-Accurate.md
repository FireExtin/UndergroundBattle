Rulebook-Accurate Action Rights / Conflict Flow / Ability Activation Plan
Summary
当前 battle 主流程的根问题不是单点 bug，而是规则核仍停留在“main/end + 若干直接动作”的原型模型，尚未把“行动权、堆叠、响应、对抗子步骤、奖励结算、行动能力”建成统一状态机。
本轮按规则书重构为“行动权驱动”的流程，并同时修复你指出的 6 个问题：

暗藏者前端可见性不足：补成明确的面朝下/暗藏态投影与 UI 呈现
调查阶段错误横置：删除把调查当成主动横置动作的旧语义
行动阶段 / 对抗阶段缺少快速行动与响应窗口：改为显式窗口状态机
调查奖励缺失：加入“检视 X、顶/底排序、抓 1”强制奖励步骤
行动能力缺失：加入通用 activate_ability 框架
行动权/堆叠返回规则不准确：按你补充的规则书条文重建 priority ownership
本轮默认采用以下规则常量：

暗藏者不参与调查对抗和战斗对抗
暗藏者在势力对抗中按 1 势力 参与
除非其他卡牌/持续效果改变该结论
Public API / State Changes
后端与前端协议统一扩容，避免再靠 UI 猜规则：

TurnState.phase.name 新增 conflict
TurnState.phase.step 从当前极简 action/ended 扩成规则书步骤名
至少包含：first_player_action、second_player_action
TurnState 新增 conflict
regionOrder
regionCardId
stage
priorityLeaderPlayerId
pendingPromptId
Action 新增
abilityId
promptId
topCardIds
bottomCardIds
damageAssignments
PlayerViewState 新增
pendingPrompt
capabilities
CardView 新增
abilities
faceDownRole
RulesMetadata 保留静态 actionPolicies，但复杂 timing 不再只靠静态 policy；UI 以 capabilities + pendingPrompt 为准
pendingPrompt 统一承载两类强制交互：

调查奖励排序
战斗奖励分伤
Implementation Changes
1. 重建行动权与堆叠核心
在 server/pkg/rules/types.go、server/pkg/rules/engine.go 一侧把当前“优先权 + pass + 空栈结束步骤”升级成显式 action-right state machine：

main phase 拆成两个步骤
first_player_action
second_player_action
绝大多数步骤开始时由当前回合先手玩家先获得行动权
second_player_action 是唯一例外
步骤开始由后手玩家先获得行动权
任一卡牌/能力结算后，也由后手玩家先拿行动权
双方连续让过：
若栈非空，结算栈顶，再按该步骤的优先权领头者重开行动权
若栈为空，推进到下一个规则步骤
可被响应并进入堆叠的对象只包括
打出卡牌
秘密派遣
付费令暗藏者现身
行动能力
触发能力
不进入堆叠的对象保持非响应语义
建立资产
持续能力
额外费用
纯“获得费用/减费”的能力
2. 把对抗阶段改成规则书子步骤，而不是直接动作
移除 battle UI 对 declare_investigation / declare_attack 的依赖；这两个动作保留为 debug 通道即可，不再代表正式对抗流程。

conflict phase 固定按地区顺序运行，每个地区的流程为：

pre_investigation_fast
resolve_investigation
investigation_reward_prompt（若有赢家）
post_investigation_fast
pre_battle_fast
resolve_battle
battle_damage_prompt（若有赢家且有合法目标）
post_battle_fast
pre_influence_fast
resolve_influence
post_influence_fast
next_region_or_end_conflict
规则细节锁定为：

调查/战斗/势力对抗都按地区内参与者的对应图标总和结算
已横置角色不能参与任何对抗
暗藏者：
不参与调查
不参与战斗
参与势力，固定按 1 势力 计算
调查对抗不会横置参与角色
战斗奖励不再是“单体 declare_attack 直接打人”，而是按差额进入分伤 prompt
调查奖励是强制结算，不入栈
战斗奖励分伤不入栈
先手标志特权只在 tie 时插入既有 use_first_player_privilege 路径，不入栈
3. 加入现身与行动能力的正式动作模型
新增正式动作：

reveal_face_down
只允许本方暗藏者
检查忠诚与费用
进入堆叠，可被响应
结算后翻为正面，保留原横置/重置状态
activate_ability
基于显式 AbilityRegistry
支持 action 与 quick 两种速度
支持费用组合：资源、横置、弃牌、牺牲、移除标记
支持上堆叠与非上堆叠能力两类
与现有 PaymentEngine 和 card permission/prohibition 统一走同一 legality 入口
能力来源不做文本自动解析，第一版使用显式结构化注册表。
注册表字段至少包括：

abilityId
sourceDefinitionId
zoneConstraint
speed
requiresStack
cost
targetSchema
effectKind
perTurnLimit
第一版只录 battle 需要的代表性能力，但接口按通用框架设计。
Battle UI 只展示当前 capabilities.activatableAbilities，不再硬编码“哪些卡大概能发动”。

4. 补齐前端投影与交互
在 web/src/battle/* 这侧做三类改动：

BattleTable
面朝下卡牌加明显 badge / 文案
本方视角显示“暗藏中 + 正面身份可自知”
对手/观众只显示“暗藏者”
Battle header / logs
显示当前 phase、step、conflict stage、行动权领头者、priority window
ActionComposer
删除 declare_investigation / declare_attack 正式入口
改成按 capabilities 渲染可提交动作
在 prompt 阶段切换为专用交互
调查奖励：顶/底排序面板
战斗奖励：差额伤害分配面板
在快速窗口里允许
快速行动能力
现身
响应牌
Test Plan
Server
新增/更新规则核测试，至少覆盖：

行动权
first_player_action 开始由先手拿权
second_player_action 开始与每次结算后由后手拿权
双方连续让过 + 非空栈时只结算栈顶，不结束步骤
双方连续让过 + 空栈时推进到下一步骤
响应窗口
打出卡牌后进入 response
现身进入 response
activate_ability 的 quick/action 两种速度都按规则进入正确窗口
对抗阶段
调查不会横置角色
暗藏者不参与调查/战斗，只参与势力且为 1
调查奖励可见牌数 = 差额，抓牌恒为 1
战斗奖励按差额进入分伤 prompt
tie + 先手特权
Prompt / hidden info
调查奖励 prompt 只对赢家暴露检视牌
opponent/spectator 看不到 deck-top hidden payload
能力框架
资源费 / 横置费 / 牺牲费 legality
不上堆叠能力不可被响应
上堆叠能力目标失效时正常 fizzles
回归
现有 play_card、build_asset、PaymentEngine、projection 合约测试继续通过
Web
新增/更新前端测试，至少覆盖：

暗藏者卡牌 badge 与 owner/opponent 视图差异
conflict stage banner 与日志文案
ActionComposer 在
主行动步骤
快速窗口
响应窗口
prompt 窗口
下只展示合法入口
调查奖励排序 UI 提交正确 action payload
战斗分伤 UI 提交正确 assignments
E2E
补一条真实 battle trace 回归：

双方秘密派遣
在调查前快速窗口现身/发动 quick ability
调查奖励排序并抓 1
战斗前再次快速响应
战斗分伤
势力对抗结算
日志与阶段 banner 全程正确
Assumptions / Defaults
declare_investigation、declare_attack、set_face_down 保留为 debug-only，不再作为正式 battle UI 行为
现有 end -> main 抓牌/恢复逻辑继续沿用，本轮重点不重写 recovery
行动能力第一版采用显式注册表，不做 cards.json 文本自动解析
未录入注册表的行动能力在 UI 中明确显示为 unsupported，而不是静默消失
暗藏者规则按你刚确认的版本实现：
不参与调查对抗
不参与战斗对抗
势力对抗中按 1 势力角色 参与
