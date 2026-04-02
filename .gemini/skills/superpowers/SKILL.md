---
name: superpowers
description: Use the "superpower" specialized development workflow for Underground Battle, including Subagent-Driven Development (SDD) and Plan-Based Execution. Use when executing implementation plans in `docs/superpowers/plans/` or when performing high-volume coding tasks.
---

# Superpowers Skill

This skill codifies the specialized development workflows used in the Underground Battle project. It emphasizes structured plan execution, subagent delegation, and strict technical integrity.

## Core Workflows

### 1. Plan-Based Execution (`executing-plans`)
When an implementation plan exists in `docs/superpowers/plans/` (e.g., `2026-04-01-legality-framework-hardening-v1.md`), follow these rules:

- **Source of Truth:** The plan is the authoritative guide for the task. Read it entirely before starting.
- **Sequential Tasking:** Execute tasks and steps strictly in order. Do not skip ahead unless explicitly instructed.
- **Checkbox Tracking:** Use the `- [ ]` syntax to track progress. After completing a step, update the plan file (or a copy) to `- [x]`.
- **Validation-First:** For bug fixes or feature additions, always reproduce the current state (or failure) with a test case before applying changes.

### 2. Subagent-Driven Development (`subagent-driven-development`)
To preserve the main context window and ensure high-signal output, delegate high-volume execution to the `generalist` subagent.

- **Orchestration:** As the main agent, you focus on high-level strategy, plan management, and final validation.
- **Delegation:** Use the `generalist` tool to perform repetitive or high-volume tasks (e.g., "Refactor these 5 files according to the plan," "Run all tests and fix any failures").
- **Context Compression:** Subagent results are consolidated into a single summary in your history, keeping the main loop lean.

### 3. Technical Integrity & TDD
- **Test-Driven Development:** Write tests first. A task is incomplete without verification logic.
- **Frequent Commits:** Commit after every successful sub-task or significant step (once requested by the user).
- **No Hacks:** Adhere to Go and TypeScript idioms. Never suppress linter warnings or bypass type safety.
- **Authoritative Rules:** All rule logic belongs in Go (`server/pkg/rules`). The frontend only interprets metadata.

## Usage Guidelines

- **When to Activate:** 
  - When the user mentions "superpower," "SDD," or "follow the plan."
  - When starting a task that has a corresponding file in `docs/superpowers/plans/`.
  - When performing batch refactoring or complex, multi-file changes.

- **How to Execute:**
  1. Identify the relevant plan in `docs/superpowers/plans/`.
  2. Break down the next step into a clear instruction for a subagent.
  3. Invoke the `generalist` subagent with the instruction and relevant context.
  4. Validate the subagent's work (tests, linting).
  5. Mark the step as complete in the plan.
