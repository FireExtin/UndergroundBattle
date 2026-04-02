# 隐秘世界 (Underground Battle) - GEMINI Context

This project is a digital implementation of the "Underground Battle" (隐秘世界) board game, designed for **AI-collaborative development** (vibe coding). It prioritizes a clear architectural boundary between an authoritative Go rules engine and a TypeScript-driven frontend/tooling ecosystem.

## 1. Project Architecture & Principles

- **Go as Truth Source:** The Go backend (`/server`) is the sole authority for game rules, legality, and state.
- **Projection System:** The server generates per-player `PlayerViewState` to ensure hidden information safety. Clients never receive the full `GameState`.
- **Replayability:** Every state change produces a new `revision`. The system is designed to be fully replayable from action logs.
- **Test-Driven:** Testing is "first-class." No complex logic should be merged without accompanying unit tests in Go and Vitest.
- **Schema-Driven UI:** The frontend (`/web`) uses `rulesMetadata` and `actionPolicies` from the server to drive UI behavior (e.g., disabling buttons), rather than hardcoding rules.

## 2. Technical Stack

- **Backend:** Go 1.25+ (Rules Engine, Match Service, Projection, Replay).
- **Frontend:** TypeScript, React 19, Vite 8, Vitest (Battle Table, Debugger, DSL Tools).
- **Data:** JSON Schemas (Card DSL, Protocol) found in `/shared`.
- **Infrastructure:** REST/WebSocket for communication; Go-hosted static assets for deployment.

## 3. Key Commands

### Backend (Go)
- `go run ./server/cmd/api`: Start the authoritative sandbox server (Default: `:8080`).
- `go test ./server/...`: Run all backend tests (Rules, API, Projections).

### Frontend (Web)
- `cd web && npm run dev`: Start Vite development server with API proxy.
- `cd web && npm run build`: Build the production frontend.
- `cd web && npm test`: Run Vitest unit tests.
- `cd web && npm run test:e2e`: Run Playwright E2E tests (requires Go server running).

### Tools
- `tools/card-importer`: Normalizes raw card data into DSL-compliant JSON.
- `tools/fixture-tools`: Validates consistency between Go and TS implementations.

## 4. Development Workflow & Conventions

1. **Research & Strategy:** Read `docs/NEXT_GEN_RULE_PLAN.md` for current implementation status.
2. **Implementation:** 
   - New rules MUST be added to `server/pkg/rules`.
   - Card data changes MUST go through the `shared/contracts/fixtures`.
3. **Validation:**
   - Go rules must pass `go test ./server/pkg/rules`.
   - Schema changes must pass `cd web && npm test`.
   - Full flow should be verified via Playwright: `npm run test:e2e`.
4. **Safety:** Never commit secrets. Protect the `.env` and `.git` directories.

## 5. Current Milestone (Phase 3)
The project is currently implementing **Region Scoring, Role Actions, and Session Lifecycle**.
- **Active:** Payment engine prototype, attack/investigation mechanics, automated match reports.
- **Stable:** Sandbox reset, priority/stack engine, basic continuous effects.

## 6. Important Files
- `README.md`: The "Grand Charter" of the project.
- `docs/CARD_DSL.md`: Contract for card logic definitions.
- `server/pkg/rules/engine.go`: Entry point for action orchestration.
- `shared/schemas/card.schema.json`: The source of truth for card data structure.
