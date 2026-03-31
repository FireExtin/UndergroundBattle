# 项目交接文档

> 《隐秘世界》数字化项目 - Codex 会话交接
> 填写说明：本模板由下一个 AI 会话填写，请先读取项目当前状态

---

## 1. 项目概览

- **项目名称**：__PROJECT_NAME__
- **项目目标**：__PROJECT_GOAL__
- **技术栈**：__TECH_STACK__
- **项目阶段**：__CURRENT_PHASE__
- **交接时间**：__DATE__

---

## 2. 当前进度

### 2.1 已完成

- [ ] __PHASE_1_DONE__
- [ ] __PHASE_2_DONE__
- [ ] __PHASE_3_DONE__
- [ ] __PHASE_4_DONE__
- [ ] __PHASE_5_DONE__
- [ ] __PHASE_6_DONE__
- [ ] __PHASE_7_DONE__
- [ ] __PHASE_8_DONE__
- [ ] __LATEST_COMMIT_FEATURE__

### 2.2 正在进行

- **当前任务**：__CURRENT_TASK__
- **相关文件**：__RELATED_FILES__
- **开始时间**：__START_DATE__

### 2.3 待完成

- [ ] __NEXT_PHASE_1__
- [ ] __NEXT_PHASE_2__
- [ ] __NEXT_PHASE_3__

---

## 3. 核心文件结构

```
/UndergroundBattle
├── server/                           # Go 规则服务端
│   ├── cmd/api/main.go               # 入口
│   ├── internal/api/                 # HTTP / Session
│   └── pkg/rules/                    # 规则核
│       ├── engine.go                 # 规则引擎核心
│       ├── m0.go                     # M0 初始状态
│       ├── role_actions.go           # 角色动作
│       ├── stack.go                  # 堆叠引擎
│       ├── priority.go               # 优先级引擎
│       ├── projection.go             # 投影视图
│       ├── continuous.go             # 持续效果
│       ├── dsl.go                    # DSL 解析
│       └── __OTHER_KEY_FILES__
│
├── web/                              # TypeScript 前端
│   ├── src/debugger/                 # 调试器
│   └── __OTHER_WEB_FILES__
│
├── shared/                           # 共享协议
│   ├── schemas/                      # JSON Schemas
│   ├── contracts/fixtures/           # 契约测试 fixtures
│   └── protocol/                    # 协议定义
│
└── docs/                             # 文档
    ├── README.md
    ├── CODEX_PROMPTS_PHASES.md
    └── __OTHER_DOCS__
```

---

## 4. 关键设计决策

### 4.1 架构原则

| 决策项 | 选择 | 原因 |
|-------|------|------|
| 规则权威 | __GO_IS_AUTHORITY__ | __REASON__ |
| 状态可回play | __REPLAY_ENABLED__ | __REASON__ |
| 隐藏信息 | __HIDDEN_INFO_STRATEGY__ | __REASON__ |
| 持续效果 | __CONTINUOUS_STRATEGY__ | __REASON__ |
| 测试策略 | __TEST_STRATEGY__ | __REASON__ |

### 4.2 Hook/Modifier 分层

```
1. Legality Hooks     → __EXAMPLES__
2. Modifier Hooks    → __EXAMPLES__
3. Replacement/Prevention → __EXAMPLES__
4. Trigger Hooks     → __EXAMPLES__
5. Cleanup Hooks     → __EXAMPLES__
```

### 4.3 持续效果 Layer

```
__LAYER_DEFINITIONS__
```

---

## 5. 当前工作细节

### 5.1 最新实现的功能

**文件**：__IMPLEMENTED_FILE__

**功能描述**：__FEATURE_DESCRIPTION__

**关键代码片段**：

```__LANGUAGE__
__CODE_SNIPPET__
```

**测试覆盖**：__TEST_COVERAGE__

### 5.2 数据结构

**关键类型定义**：

```__LANGUAGE__
__TYPE_DEFINITIONS__
```

---

## 6. 已知问题与风险

### 6.1 待解决

| 问题 | 位置 | 状态 | 备注 |
|-----|------|-----|------|
| __ISSUE_1__ | __LOCATION__ | __STATUS__ | __NOTE__ |
| __ISSUE_2__ | __LOCATION__ | __STATUS__ | __NOTE__ |

### 6.2 已知限制

- __LIMIT_1__
- __LIMIT_2__

### 6.3 风险提示

- __RISK_1__
- __RISK_2__

---

## 7. 下一步任务

### 7.1 第一优先级

**任务**：__TASK_1__
**目标**：__GOAL_1__
**步骤**：
1. __STEP_1__
2. __STEP_2__
3. __STEP_3__

### 7.2 第二优先级

**任务**：__TASK_2__
**目标**：__GOAL_2__

### 7.3 第三优先级

**任务**：__TASK_3__
**目标**：__GOAL_3__

---

## 8. 测试命令

### 8.1 Go 测试

```bash
# 运行所有 Go 测试
go test ./...

# 运行特定模块
go test ./server/pkg/rules/...

# 监听模式
go test -v -run __TEST_NAME__
```

### 8.2 TypeScript 测试

```bash
cd web
npm test
```

### 8.3 启动服务

```bash
# Go 服务
go run ./server/cmd/api

# 前端开发
cd web && npm run dev
```

---

## 9. 重要文档索引

| 文档 | 路径 | 用途 |
|------|------|------|
| __DOC_1__ | __PATH_1__ | __PURPOSE_1__ |
| __DOC_2__ | __PATH_2__ | __PURPOSE_2__ |
| __DOC_3__ | __PATH_3__ | __PURPOSE_3__ |

---

## 10. 交接人信息

- **交接人**：__PREVIOUS_AGENT__
- **交接时间**：__DATE__
- **联系方式**：__CONTACT__

---

## 11. 附：最近提交记录

```
__GIT_LOG__
```

---

## 12. 注意事项

1. **__RULE_1__**
2. **__RULE_2__**
3. **__RULE_3__**

---

## 填写指南

请按以下步骤填写本模板：

1. 读取 `README.md` 获取项目背景和架构
2. 运行 `git log --oneline -10` 获取最近提交
3. 检查 `server/pkg/rules/` 下的核心文件，确定当前实现状态
4. 读取 `docs/milestones/m0-sandbox.md` 获取 M0 基线
5. 读取 `docs/NEXT_STEP_EXECUTION_PLAN_*.md` 获取下一步计划
6. 检查 `web/src/debugger/` 确定前端状态
7. 填写所有 `__XXX__` 占位符
8. 删除所有空项或不适用的项

---

*请在开始工作前阅读此文档，如有问题请联系交接人。*
