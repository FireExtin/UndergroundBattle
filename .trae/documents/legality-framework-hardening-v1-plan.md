# Legality Framework Hardening V1 实施计划

> **目标**: 稳定当前的 `XQ22` / `XQ31` 合法性表面，提取生产规则目录和共享合法性匹配器，不改变当前游戏玩法语义。

## 核心原则

- **TDD**: 先写测试，再实现代码
- **准确定义问题**: 使用形式化思维分析问题
- **严格建模**: 数字卡牌游戏容不得一丝宽松懈怠
- **遵循**: `/2026-04-01-legality-framework-hardening-v1.md` 进行开发。

## 范围边界

### 在范围内
- `XQ22` 和 `XQ31` 的生产级合法性规则目录
- `CardCondition` 和 actor scope 的共享匹配器辅助函数
- `queue_operation` 与角色动作边界的回归测试覆盖
- 新结构的文档同步

### 不在范围内
- `XQ01` 区域/能力类型语义
- 完整的 `XQ31` 数值光环 (`+1 defense`) 实现
- 附属/永久物生命周期 V1
- Web 调试器/传输/WebSocket/持久化工作

## 文件映射

- **创建**: `server/pkg/rules/legality_catalog.go`
- **创建**: `server/pkg/rules/legality_catalog_test.go`
- **创建**: `server/pkg/rules/legality_shared.go`
- **创建**: `server/pkg/rules/legality_shared_test.go`
- **修改**: `server/pkg/rules/prohibition.go`
- **修改**: `server/pkg/rules/target_legality.go`
- **修改**: `server/pkg/rules/prohibition_test.go`
- **修改**: `server/pkg/rules/role_actions_test.go`
- **修改**: `docs/HANDOVER_TRAE_2026-04-01.md`
- **修改**: `docs/NEXT_GEN_RULE_PLAN.md`

## 成功标准

1. `engine.go` 不再包含卡牌定义特定的合法性匹配逻辑
2. `XQ22` 和 `XQ31` 的生产规则注册集中在一个明显的文件中
3. `prohibition.go` 和 `target_legality.go` 不再各自保留 source-condition 和 actor-scope 匹配逻辑的副本
4. `declare_attack` 和 `declare_investigation` 不受 `XQ31` 影响
5. `queue_operation` 继续遵守 `XQ22` 和 `XQ31`
6. 所有测试通过: `go test ./server/...`, `cd tools/fixture-tools && npm test`, `cd web && npm test`

---

## 任务 1: 添加生产规则目录 API

**文件**:
- 创建: `server/pkg/rules/legality_catalog_test.go`
- 创建: `server/pkg/rules/legality_catalog.go`

### 步骤 1: 编写失败的目录测试

编写测试 `TestBuildProductionProhibitionRules` 和 `TestBuildProductionTargetLegalityRules`，验证:
- 生产禁止规则只有 XQ22，目标类型为 "事务"
- 生产目标合法性规则只有 XQ31，影响 "声望" 关键词的盟友

### 步骤 2: 运行测试确认失败

运行: `go test ./server/pkg/rules -run 'TestBuildProduction(Prohibition|TargetLegality)Rules' -count=1`

预期: FAIL - 函数未定义

### 步骤 3: 实现目录文件

创建 `legality_catalog.go`:
- `BuildProductionProhibitionRules()` - 返回 XQ22 规则
- `BuildProductionTargetLegalityRules()` - 返回 XQ31 规则

### 步骤 4: 将当前构建器连接到目录

修改 `prohibition.go` 和 `target_legality.go`:
- `BuildProhibitionChecker` 调用 `BuildProductionProhibitionRules()`
- `BuildTargetLegalityChecker` 调用 `BuildProductionTargetLegalityRules()`

### 步骤 5: 运行目标测试验证通过

运行: `go test ./server/pkg/rules -run 'TestBuildProduction(Prohibition|TargetLegality)Rules' -count=1`

预期: PASS

### 步骤 6: 提交

```bash
git add server/pkg/rules/legality_catalog.go server/pkg/rules/legality_catalog_test.go server/pkg/rules/prohibition.go server/pkg/rules/target_legality.go
git commit -m "refactor: add legality production rule catalog"
```

---

## 任务 2: 提取共享的源和范围匹配

**文件**:
- 创建: `server/pkg/rules/legality_shared_test.go`
- 创建: `server/pkg/rules/legality_shared.go`
- 修改: `server/pkg/rules/prohibition.go`
- 修改: `server/pkg/rules/target_legality.go`

### 步骤 1: 编写失败的辅助函数测试

编写测试:
- `TestCardMatchesDefinitionAndCondition` - 验证卡牌匹配定义ID和条件
- `TestScopeAppliesToActor` - 验证范围是否适用于actor

### 步骤 2: 运行测试确认失败

运行: `go test ./server/pkg/rules -run 'Test(CardMatchesDefinitionAndCondition|ScopeAppliesToActor)' -count=1`

预期: FAIL - 函数未定义

### 步骤 3: 实现共享辅助文件

创建 `legality_shared.go`:
- `cardMatchesDefinitionAndCondition(card, definitionID, condition)` - 检查卡牌是否匹配定义ID和条件
- `scopeAppliesToActor(sourceCard, actorID, scope)` - 检查范围是否适用于actor

### 步骤 4: 在评估器中替换重复逻辑

修改 `prohibition.go`:
- `matchesSourceCondition` 调用 `cardMatchesDefinitionAndCondition`
- `matchesScope` 调用 `scopeAppliesToActor`

修改 `target_legality.go`:
- `matchesSourceCondition` 调用 `cardMatchesDefinitionAndCondition`
- `matchesActorRestriction` 调用 `scopeAppliesToActor`

### 步骤 5: 运行辅助函数和现有合法性测试

运行: `go test ./server/pkg/rules -run 'Test(CardMatchesDefinitionAndCondition|ScopeAppliesToActor|ProhibitionChecker|TargetLegalityXQ31)' -count=1`

预期: PASS

### 步骤 6: 提交

```bash
git add server/pkg/rules/legality_shared.go server/pkg/rules/legality_shared_test.go server/pkg/rules/prohibition.go server/pkg/rules/target_legality.go
git commit -m "refactor: share legality source and scope matchers"
```

---

## 任务 3: 锁定 XQ31 的引擎边界

**文件**:
- 修改: `server/pkg/rules/role_actions_test.go`
- 修改: `server/pkg/rules/prohibition_test.go`

### 步骤 1: 添加 `declare_investigation` 的回归测试

编写测试 `TestDeclareInvestigationIgnoresXQ31TargetLegalityRestriction`:
- 设置状态: P2 的调查员，P1 的 XQ31 声望角色，区域卡
- 提交 `declare_investigation` 动作
- 验证: 动作成功，不受 XQ31 影响

### 步骤 2: 添加生产禁止构建器忽略测试规则的回归测试

编写测试 `TestBuildProhibitionCheckerIgnoresTestOnlyRules`:
- 在桌面上放置 TEST01 卡
- 使用生产构建器检查
- 验证: TEST01 被忽略，不触发禁止

### 步骤 3: 运行聚焦的回归测试

运行: `go test ./server/pkg/rules -run 'Test(DeclareInvestigationIgnoresXQ31TargetLegalityRestriction|BuildProhibitionCheckerIgnoresTestOnlyRules)' -count=1`

预期: PASS

### 步骤 4: 如果失败，修复最小边界代码

确保边界检查保持完整:
```go
if action.TargetCardID != "" && action.Kind == ActionKindQueueOperation {
    targetLegalityChecker := BuildTargetLegalityChecker(state)
    targetResult := targetLegalityChecker.CheckTargetCard(state, action.ActorID, action.TargetCardID)
    // ...
}
```

### 步骤 5: 提交

```bash
git add server/pkg/rules/role_actions_test.go server/pkg/rules/prohibition_test.go server/pkg/rules/engine.go
git commit -m "test: lock legality engine boundaries"
```

---

## 任务 4: 同步文档并运行完整验证

**文件**:
- 修改: `docs/HANDOVER_TRAE_2026-04-01.md`
- 修改: `docs/NEXT_GEN_RULE_PLAN.md`

### 步骤 1: 更新交接文档

添加:
- 合法性生产规则现在位于 `server/pkg/rules/legality_catalog.go`
- 共享的源条件和actor范围匹配现在位于 `server/pkg/rules/legality_shared.go`
- `XQ31` 保持仅 queue-operation 目标；角色动作保持在此门之外
- `XQ01` 保持推迟，等待区域/能力类型模型

### 步骤 2: 更新路线图文档

添加:
- 阶段 3 合法性强化现在有专门的生产规则目录
- 重复的合法性匹配器逻辑已折叠到共享辅助函数中
- 下一个跟进仍然是 `XQ31` 数值光环完成或 `XQ01` 先决条件设计，不是 UI 工作

### 步骤 3: 运行 Go 验证

运行: `go test ./server/...`

预期: PASS

### 步骤 4: 运行 TypeScript 验证

运行: `cd tools/fixture-tools && npm test`

预期: PASS

运行: `cd web && npm test`

预期: PASS

### 步骤 5: 提交

```bash
git add docs/HANDOVER_TRAE_2026-04-01.md docs/NEXT_GEN_RULE_PLAN.md
git commit -m "docs: sync legality framework hardening"
```

---

## 后续工作建议

完成此计划后，不要直接进入更多卡牌计数。推荐的下一个顺序是:

1. 完成 `XQ31` 的另一半，为盟友声望角色提供最小连续数值光环
2. 在接触代码之前，为 `XQ01` 区域范围能力沉默编写规范
3. 只有在那之后才开始 `Attachment / Permanent Model V1`，其中 `BQ022` 成为真正的桌面永久物，而不是附属追踪元数据

## 最终验证包

在交还工作之前运行:

```bash
go test ./server/...
cd tools/fixture-tools && npm test
cd ../web && npm test
```
