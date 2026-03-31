# NEXT_STEP_EXECUTION_PLAN_2026-03-31

Purpose: turns the broad next-generation rules roadmap into the concrete next execution order after the live sandbox milestone.

## Inputs

- 基础路线图来自 [NEXT_GEN_RULE_PLAN.md](/Users/ddd/Downloads/UndergroundBattle/docs/NEXT_GEN_RULE_PLAN.md)
- 当前实现状态来自 [LIVE_SANDBOX_2026-03-31.md](/Users/ddd/Downloads/UndergroundBattle/docs/LIVE_SANDBOX_2026-03-31.md)
- 外部建议中“先冻结 M0、先建回归护城河、不要立刻继续堆 UI/扩卡”这一点是对的

## My Judgment

### 我认同的部分

- 先把当前 sandbox 固化成可回归基线，而不是立刻继续堆更多 UI
- 先补 golden scenarios、replay、一致性不变量、projection 泄露测试
- 先做第一套完整可玩规则闭环，再扩卡池
- 把当前网页定位成 rules-core 的回归面板，而不是产品页面

### 我不完全认同的部分

- 真实 WebSocket 联机现在还偏早
  - 规则闭环还没有完成
  - 角色动作和地区争夺还没有形成完整一局
- OpenTelemetry 现在也偏早
  - 当前服务形态仍然是单进程、单 session 的 M0 sandbox
  - 先把会发生什么事件稳定下来，再决定埋点粒度更划算
- Playwright 也应该后置
  - 先有稳定的 golden/browser smoke 目标，再上 e2e 更合适

## Decision

下一步不做“更多按钮、更多卡、更多 transport”，而是做：

**Phase 1：冻结 M0 sandbox 基线**

然后紧接着做：

**Phase 2：补回归护城河，并把当前 sandbox 升级成规则回归机**

只有这两步完成后，才进入：

**Phase 3：第一套完整可玩规则闭环**

之后才是：

**Phase 4：首发卡池工程化，再之后才考虑联机与观测**

## Recommended Sequence

### Phase 1: M0 Baseline Freeze

Goal:
- 把当前 live sandbox 变成一个明确的、可长期回归验证的最小基线

Deliverables:
- `docs/milestones/m0-sandbox.md`
- 当前支持动作与规则边界清单
- 8-12 个固定 baseline scenarios 清单
- baseline 的预期结果定义：
  - final revision
  - final stack
  - final priority
  - final per-player views
  - final action log

Why first:
- 如果没有这个基线，接下来的角色动作、完整对局、扩卡都会缺少稳定参照物

Exit Criteria:
- 当前 sandbox 的可见行为被明确冻结
- 后续大改都能先拿这组 baseline 做回归判断

### Phase 2: Regression Harness

Goal:
- 让当前规则核变成“可安全重构”的系统

Scope:
- golden tests
- replay consistency tests
- projection leak tests
- legality reason-code coverage
- continuous recalculation idempotency tests
- RNG seed consistency tests
- targeted fuzz tests for fast deterministic surfaces

Why second:
- 这一步是之后继续改 legality、stack、projection、continuous 的护城河

Important constraint:
- fuzzing 只接快速且确定性的规则表面，不要一开始就把整个浏览器或整套对局脚本塞进去

Exit Criteria:
- `go test ./...` 内存在稳定的 baseline / replay / invariants 防线
- 重构 rules core 时能知道自己有没有把东西改坏

### Phase 3: First Playable Rules Loop

Goal:
- 从“动作演示 sandbox”推进到“能完整打一局的极小格式”

Scope:
- 角色动作入口：`declare_attack`、`declare_investigation`
- 回合推进的实际目标：地区争夺、得分、胜利条件
- 持续效果生命周期补完：source 离场导致 effect 清理
- 最小触发/替代只做第一批必要闭环，不展开复杂泛化

Why third:
- 当前 rules core 已经有 stack / priority / projection / continuous，但还没有完整对局闭环

Exit Criteria:
- 能从开局打到胜负
- replay 可完整重放
- P1 / P2 / spectator 仍不泄露隐藏信息

### Phase 4: First Maintainable Card Pool

Goal:
- 先把扩卡流程工程化，再扩卡

Scope:
- 第一批真实可玩 fixture
- compatibility matrix
- complexity tags
- 20-40 张首发卡池，不追求更大

Why fourth:
- 如果在规则闭环前猛加卡，只会把 bug 面积放大

Exit Criteria:
- 每张卡都有 fixture gate
- Go / TS 双端契约通过后才准入

## Immediate Task List

接下来建议直接拆成下面 8 项，不再继续做 transport/UI 扩张：

1. 写 `docs/milestones/m0-sandbox.md`
2. 固定 8-12 个 M0 golden scenarios
3. 为每个 scenario 产出 JSON baseline 结果
4. 写 Go golden tests 读取并断言 baseline
5. 写 replay consistency tests
6. 写 projection leak tests
7. 写 legality reason-code coverage tests
8. 写 continuous / RNG / revision invariants tests

## Explicitly Deferred

这几项现在不做，避免打乱优先级：

- 继续堆新的 sandbox 按钮
- 立刻扩大量卡池
- 真实 WebSocket room
- OpenTelemetry tracing / metrics / logs
- Playwright e2e
- 美术、动效、桌面壳

## Next Review Point

当且仅当下面这两条同时成立时，再进入完整可玩闭环和更大规模扩卡：

- `M0 sandbox` baseline 已冻结
- baseline / replay / invariants / leak tests 已经成为默认回归门槛
