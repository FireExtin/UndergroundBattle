# AGENTS.md - Agentic Coding Guidelines

This file provides guidance for AI agents working on the Underground Battle project.

---

## 1. Project Overview

- **Name**: 隐秘世界 (Underground Battle) - Digital board game implementation
- **Tech Stack**: Go 1.25+ (backend/rules engine) + TypeScript/React 19 (frontend)
- **Architecture**: Go is the sole authority for game rules. TypeScript frontend uses server-provided `rulesMetadata` and `actionPolicies` for UI behavior (schema-driven, not hardcoded).

---

## 2. Build, Lint & Test Commands

### Backend (Go)

```bash
# Run server
go run ./server/cmd/api

# Run all tests
go test ./server/...

# Run single test (example)
go test -v -run TestFunctionName ./server/pkg/rules/

# Run tests with coverage
go test -cover ./server/...
```

### Frontend (TypeScript/Web)

```bash
# Install dependencies
cd web && npm install

# Development server
cd web && npm run dev

# Production build
cd web && npm run build

# Type checking
cd web && npm run typecheck

# Unit tests (Vitest)
cd web && npm test

# Single test file
cd web && npm test --run src/path/to/file.test.ts

# E2E tests (requires Go server running)
cd web && npm run test:e2e
```

### Tools

```bash
# Card importer normalizes raw card data
cd tools/card-importer && npm install && npm run build

# Fixture tools validate consistency
cd tools/fixture-tools && npm install && npm test
```

---

## 3. Code Style Guidelines

### Go (Backend)

- **Formatting**: Use `gofmt` (included with Go). Run `go fmt ./...` before committing.
- **Naming**: 
  - PascalCase for types, interfaces, and exported functions
  - snake_case for variables and unexported functions
  - Use descriptive names (e.g., `CheckLegality`, `action_permission_flow.go`)
- **Imports**: Group in order: standard library, external packages, internal packages
- **Error Handling**: Return structured `LegalityResult` instead of plain text errors. Use the `legalityFailure()` helper with reason codes.
- **Testing**: Follow table-driven test patterns. Test files end with `_test.go`.
- **Architecture**: 
  - Rules logic lives in `server/pkg/rules/`
  - API handlers in `server/internal/api/`
  - Contract parsing in `server/pkg/contracts/`

### TypeScript (Frontend)

- **Formatting**: Automatic via Vite/TypeScript
- **Naming**: 
  - PascalCase for components and interfaces
  - camelCase for variables and functions
  - File names: `kebab-case.tsx` for components, `snake-case.test.ts` for tests
- **Types**: Always use explicit types. Avoid `any`.
- **Testing**: Use Vitest with `@testing-library/react`. Test files end with `.test.ts` or `.test.tsx`.

### Shared/Contracts

- JSON schemas in `/shared/schemas/` define the contract between Go and TypeScript
- Card fixtures in `/shared/contracts/fixtures/` are required for new cards
- Schema changes must pass validation in both Go and TypeScript

---

## 4. Key Architectural Principles

1. **Go as Truth Source**: All game rules, legality, and state are authoritative in Go. Frontend must not implement rule logic.

2. **Projection System**: Server generates per-player `PlayerViewState` to ensure hidden information safety. Clients never receive full `GameState`.

3. **Test-Driven**: No complex logic merged without unit tests. Tests are first-class citizens.

4. **Schema-Driven UI**: Frontend reads `rulesMetadata.actionPolicies` from server for UI behavior (enabling/disabling buttons), not hardcoded rules.

5. **Replayability**: Every state change produces a new `revision`. System must be fully replayable from action logs.

---

## 5. Important Files & Directories

- `server/pkg/rules/engine.go` - Main rules pipeline entry point
- `server/pkg/rules/types.go` - Core type definitions and constants
- `server/pkg/rules/projection.go` - Player view projection logic
- `shared/schemas/card.schema.json` - Card data structure contract
- `docs/CARD_DSL.md` - Card DSL specification
- `GEMINI.md` - Project context and current milestone

---

## 6. Development Workflow

1. Read `docs/NEXT_GEN_RULE_PLAN.md` for current implementation status
2. Implement rules in `server/pkg/rules/` with accompanying `_test.go` files
3. Update fixtures in `shared/contracts/fixtures/` for new cards
4. Validate: Go tests pass, TypeScript typecheck passes, E2E tests pass
5. Never commit secrets; protect `.env` and credential files

---

## 7. Current Milestone (Phase 3)

Active development on:
- Region scoring mechanics
- Role actions system
- Session lifecycle management
- Payment engine prototype

Stable features:
- Priority/stack engine
- Basic continuous effects
- Sandbox reset functionality
- Projection system for hidden information
