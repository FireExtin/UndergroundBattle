# Playable Battle Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a simple but actually playable battle-table frontend (not a debugger shell) on top of the existing Go authoritative backend and iterate with Playwright game-flow QA until no blocking gameplay issues remain.

**Architecture:** Keep Go as the only rules authority and keep all state transitions server-side via `/api/debugger/*`. Add a new web `battle` feature slice that derives board zones and interaction candidates from existing `StatePatched` envelopes, then render a table-style UI matching the provided board layout. Action submission remains generic and explicit (action composer + quick actions) to avoid frontend rule duplication.

**Tech Stack:** React 19 + TypeScript + Vite + Vitest + Testing Library + Playwright, Go HTTP sandbox.

---

## File Structure

- `web/src/battle/model.ts`
  - Pure selectors and transformation helpers that map protocol envelopes to battle zones/state.
- `web/src/battle/model.test.ts`
  - Unit tests for zone grouping, hidden-information rendering expectations, and edge conditions.
- `web/src/battle/BattleShell.tsx`
  - Live data transport (load/reset/submit), player-side switching, and interaction orchestration.
- `web/src/battle/components/BattleTable.tsx`
  - Main board UI: opponent area, contest area (regions), local area, hand zones, deck/discard/score/secret areas.
- `web/src/battle/components/ActionComposer.tsx`
  - Playable action controls for attack/investigate/move/reveal/inspect/queue/pass/advance/markers.
- `web/src/battle/BattleShell.test.tsx`
  - Integration tests for interaction flows and disabled/error states.
- `web/src/debugger/protocol.ts`
  - Extend card DTO with minimally required display metadata (`kind`, `regionCardId`, `regionOrder`) from projection.
- `server/pkg/rules/projection.go`
  - Include the same minimal metadata in `CardView` projection payload.
- `server/pkg/rules/projection_test.go`
  - Verify new metadata serialization and hidden-information behavior.
- `web/src/app/AppShell.tsx`
  - Switch app entry from debugger shell to battle shell.
- `web/src/styles/global.css`
  - Battle-table visual system and responsive layout.
- `web/playwright.config.ts`
  - Playwright setup with web server boot.
- `web/tests/battle.spec.ts`
  - End-to-end gameplay smoke script.
- `web/package.json`
  - Add Playwright scripts.
- `docs/NEXT_GEN_RULE_PLAN.md`
  - Document implemented battle frontend milestone and QA loop result.
- `docs/WEB_BATTLE_FRONTEND_2026-04-02.md`
  - Focused usage and architecture notes.

---

### Task 1: Expose Minimal Card Metadata for Board Zoning

**Files:**
- Modify: `server/pkg/rules/projection.go`
- Modify: `server/pkg/rules/projection_test.go`
- Modify: `web/src/debugger/protocol.ts`

- [ ] **Step 1: Write the failing Go projection test**

```go
func TestProjectionCardViewIncludesKindAndRegionMetadata(t *testing.T) {
    state := NewGameState(InitialStateConfig{GameID: "meta", ActivePlayerID: "P1", PlayerIDs: []string{"P1", "P2"}})
    state.Board.Cards = []CardState{{
        CardID: "region-1", Name: "R1", Kind: CardKindRegion, OwnerID: "P1",
        Zone: CardZoneTable, Revealed: true, RegionOrder: 1,
    }, {
        CardID: "unit-1", Name: "U1", Kind: CardKindCharacter, OwnerID: "P1",
        Zone: CardZoneTable, Revealed: true, RegionCardID: "region-1",
    }}

    view := NewProjectionEngine().Generate(state).Players["P1"]
    card := findProjectedCardByID(t, view.Board.Cards, "unit-1")
    if card.Kind != string(CardKindCharacter) || card.RegionCardID != "region-1" {
        t.Fatalf("unexpected projected metadata: %#v", card)
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./server/pkg/rules -run TestProjectionCardViewIncludesKindAndRegionMetadata -v`
Expected: FAIL with missing `CardView` fields / compile mismatch.

- [ ] **Step 3: Write minimal implementation**

```go
type CardView struct {
    CardID       string   `json:"cardId,omitempty"`
    Name         string   `json:"name,omitempty"`
    OwnerID      string   `json:"ownerId"`
    Zone         CardZone `json:"zone"`
    Kind         string   `json:"kind,omitempty"`
    RegionCardID string   `json:"regionCardId,omitempty"`
    RegionOrder  int      `json:"regionOrder,omitempty"`
    // ...existing fields
}

func visibleCardView(card CardState, markers map[string]int) CardView {
    return CardView{
        CardID: card.CardID,
        Name: card.Name,
        OwnerID: card.OwnerID,
        Zone: card.Zone,
        Kind: string(card.Kind),
        RegionCardID: card.RegionCardID,
        RegionOrder: card.RegionOrder,
        // ...existing fields
    }
}
```

```ts
export type CardView = {
  cardId?: string;
  name?: string;
  ownerId: string;
  zone: CardZone;
  kind?: string;
  regionCardId?: string;
  regionOrder?: number;
  // ...existing fields
};
```

- [ ] **Step 4: Run tests to verify pass**

Run:
- `go test ./server/pkg/rules -run TestProjectionCardViewIncludesKindAndRegionMetadata -v`
- `cd web && npm test -- src/debugger/live.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add server/pkg/rules/projection.go server/pkg/rules/projection_test.go web/src/debugger/protocol.ts
git commit -m "feat(protocol): project card kind and region metadata for battle table"
```

---

### Task 2: Build Battle-State Selectors With TDD

**Files:**
- Create: `web/src/battle/model.ts`
- Test: `web/src/battle/model.test.ts`

- [ ] **Step 1: Write failing selector tests**

```ts
it("groups cards into opponent/local hands, decks, discard, score and contest regions", () => {
  const state = deriveBattleState(playableMessages, "P1");
  expect(state.local.hand.length).toBeGreaterThanOrEqual(1);
  expect(state.opponent.handCount).toBeGreaterThanOrEqual(1);
  expect(state.contest.regions.length).toBe(3);
});

it("keeps opponent hidden cards as card backs", () => {
  const state = deriveBattleState(playableMessages, "P1");
  expect(state.opponent.handPreview.every((c) => c.visibility === "hidden")).toBe(true);
});

it("returns empty-safe defaults when no patch exists", () => {
  expect(deriveBattleState([], "P1").match.status).toBe("active");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cd web && npm test -- src/battle/model.test.ts`
Expected: FAIL because `deriveBattleState` is missing.

- [ ] **Step 3: Implement minimal selectors**

```ts
export function deriveBattleState(messages: DebuggerProtocolEnvelope[], localPlayerId: "P1" | "P2") {
  const localPatch = selectCurrentPatch(messages, localPlayerId);
  const cards = selectCurrentCards(localPatch);
  const local = cards.filter((card) => card.ownerId === localPlayerId);
  const opponentId = localPlayerId === "P1" ? "P2" : "P1";
  const opponent = cards.filter((card) => card.ownerId === opponentId);
  // group by zone + region metadata; provide safe defaults.
  return {
    local: { hand: zone(local, "hand"), deck: zone(local, "deck"), discard: zone(local, "discard"), score: zone(local, "score") },
    opponent: { handCount: zone(opponent, "hand").length, handPreview: zone(opponent, "hand"), deckCount: zone(opponent, "deck").length },
    contest: buildContestRegions(cards),
    match: patchMatch(localPatch),
    turn: patchTurn(localPatch),
  };
}
```

- [ ] **Step 4: Run test to pass**

Run: `cd web && npm test -- src/battle/model.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/battle/model.ts web/src/battle/model.test.ts
git commit -m "feat(web): add battle state selectors for board zoning"
```

---

### Task 3: Implement Battle UI and Action Composer

**Files:**
- Create: `web/src/battle/BattleShell.tsx`
- Create: `web/src/battle/components/BattleTable.tsx`
- Create: `web/src/battle/components/ActionComposer.tsx`
- Modify: `web/src/app/AppShell.tsx`
- Modify: `web/src/styles/global.css`

- [ ] **Step 1: Write failing interaction tests for battle shell**

```ts
it("loads live messages and renders battle zones", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(jsonResponse(playableMessages)));
  render(<BattleShell fallbackMessageSets={defaultMockMessageSets} />);
  await screen.findByText("对方玩家区域");
  expect(screen.getByText("本方玩家区域")).toBeInTheDocument();
});

it("submits attack action from composer", async () => {
  // load then action submit
  fireEvent.change(screen.getByLabelText("Action Kind"), { target: { value: "declare_attack" }});
  fireEvent.click(screen.getByRole("button", { name: "提交动作" }));
  expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/debugger/actions");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cd web && npm test -- src/battle/BattleShell.test.tsx`
Expected: FAIL because components do not exist.

- [ ] **Step 3: Implement Battle shell + board + composer**

```tsx
export function BattleShell({ fallbackMessageSets }: Props) {
  const [messages, setMessages] = useState<DebuggerProtocolEnvelope[]>(fallbackMessageSets[0]?.messages ?? []);
  const [localPlayerId, setLocalPlayerId] = useState<"P1" | "P2">("P1");
  const battle = deriveBattleState(messages, localPlayerId);

  return (
    <main className="battle-shell">
      <BattleTable battle={battle} localPlayerId={localPlayerId} onLocalPlayerChanged={setLocalPlayerId} />
      <ActionComposer battle={battle} actorId={localPlayerId} onSubmitAction={submitAction} onReset={reset} />
    </main>
  );
}
```

```tsx
<section className="battle-table__opponent" aria-label="对方玩家区域">...</section>
<section className="battle-table__contest" aria-label="争夺区">...</section>
<section className="battle-table__local" aria-label="本方玩家区域">...</section>
```

```css
:root {
  --table-bg: #d6ebee;
  --table-line: #0d5f8f;
  --panel-bg: #f7fbfd;
  --card-back: linear-gradient(135deg, #1f2937, #374151);
}
.battle-table {
  background: radial-gradient(circle at 30% 20%, #ecf8fb 0%, var(--table-bg) 50%, #c9e2e8 100%);
  border: 4px solid var(--table-line);
}
```

- [ ] **Step 4: Switch app entry to battle shell**

```tsx
export function AppShell() {
  return <BattleShell fallbackMessageSets={defaultMockMessageSets} />;
}
```

- [ ] **Step 5: Run tests to verify pass**

Run:
- `cd web && npm test -- src/battle/BattleShell.test.tsx`
- `cd web && npm test -- src/debugger/LiveDebuggerShell.test.tsx`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/battle web/src/app/AppShell.tsx web/src/styles/global.css
git commit -m "feat(web): replace debugger shell with playable battle table UI"
```

---

### Task 4: Add Edge/Exception Tests for Action Composer

**Files:**
- Modify: `web/src/battle/BattleShell.test.tsx`
- Modify: `web/src/battle/model.test.ts`

- [ ] **Step 1: Add failing edge-case tests**

```ts
it("disables submit when game is finished", async () => {
  renderWithFinishedPatch();
  expect(screen.getByRole("button", { name: "提交动作" })).toBeDisabled();
});

it("shows error when custom card-target action misses required cardId", async () => {
  selectActionKind("reveal_card");
  fireEvent.click(screen.getByRole("button", { name: "提交动作" }));
  expect(await screen.findByText("需要选择卡牌")).toBeInTheDocument();
});

it("handles fallback mode without crashing and keeps reset available", async () => {
  vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));
  render(...);
  expect(await screen.findByText("Live server unavailable")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cd web && npm test -- src/battle/BattleShell.test.tsx src/battle/model.test.ts`
Expected: FAIL on missing validations / disabled guards.

- [ ] **Step 3: Implement minimal validation and guard logic**

```ts
if (battle.match.status === "finished") {
  return { ok: false, message: "game_over" };
}
if ((kind === "reveal_card" || kind === "inspect_card") && selectedCardId === "") {
  setComposerError("需要选择卡牌");
  return;
}
```

- [ ] **Step 4: Run tests to pass**

Run: `cd web && npm test -- src/battle/BattleShell.test.tsx src/battle/model.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/battle/BattleShell.test.tsx web/src/battle/model.test.ts web/src/battle/components/ActionComposer.tsx
git commit -m "test(web): cover battle edge and exception interaction paths"
```

---

### Task 5: Playwright Gameplay QA Loop

**Files:**
- Modify: `web/package.json`
- Create: `web/playwright.config.ts`
- Create: `web/tests/battle.spec.ts`

- [ ] **Step 1: Write failing Playwright smoke test**

```ts
test("can open battle table, reset, and submit pass priority", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("争夺区")).toBeVisible();
  await page.getByRole("button", { name: "重开对局" }).click();
  await page.getByRole("button", { name: "Pass Priority" }).click();
  await expect(page.getByText("最近动作")).toBeVisible();
});
```

- [ ] **Step 2: Run to verify failure**

Run: `cd web && npx playwright test tests/battle.spec.ts --project=chromium`
Expected: FAIL before config/scripts/UI hooks are ready.

- [ ] **Step 3: Add config and stable selectors**

```ts
export default defineConfig({
  testDir: "./tests",
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 4173",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: true,
  },
  use: { baseURL: "http://127.0.0.1:4173" },
});
```

- [ ] **Step 4: Run iterative QA-fix cycles until green**

Run loop:
- `cd web && npx playwright test tests/battle.spec.ts --project=chromium`
- fix any failing UX/selector/interaction bug
- rerun same command
Expected: PASS with no blocking playability issue in smoke path.

- [ ] **Step 5: Commit**

```bash
git add web/package.json web/package-lock.json web/playwright.config.ts web/tests/battle.spec.ts
git commit -m "test(e2e): add playwright battle smoke and stabilize playable flow"
```

---

### Task 6: Full Verification + Docs Sync

**Files:**
- Create: `docs/WEB_BATTLE_FRONTEND_2026-04-02.md`
- Modify: `docs/NEXT_GEN_RULE_PLAN.md`

- [ ] **Step 1: Run full verification commands**

Run:
- `go test ./server/...`
- `cd web && npm test`
- `cd web && npm run build`
- `cd web && npx playwright test tests/battle.spec.ts --project=chromium`
Expected: all PASS.

- [ ] **Step 2: Write docs based on actual diff**

```md
## Battle Frontend (Playable)
- Replaced debugger-first entry with battle table UI.
- Added action composer for core engine actions.
- Added Playwright smoke loop and fixed discovered interaction bugs.
- Known limitations: still minimal card art/assets, action semantics remain backend-limited.
```

- [ ] **Step 3: Commit docs**

```bash
git add docs/WEB_BATTLE_FRONTEND_2026-04-02.md docs/NEXT_GEN_RULE_PLAN.md
git commit -m "docs(web): document playable battle frontend and QA loop"
```

---

## Self-Review

1. **Spec coverage check:**
- “真正可对战界面” -> Task 2/3 builds battle table + action composer.
- “基于当前后端基础” -> Task 1 keeps server authority and only adds projection metadata.
- “使用 superpower 步骤开发” -> entire plan uses strict TDD + frequent commits.
- “mcp playwright 试玩并循环修复” -> Task 5 defines explicit loop until green.
- “无需确认” -> execution handoff defaults to immediate inline execution.

2. **Placeholder scan:**
- No TBD/TODO placeholders remain.
- Every code-writing step includes concrete code snippets.
- Every verification step includes exact commands and expected outcomes.

3. **Type consistency:**
- `CardView.kind/regionCardId/regionOrder` is used consistently between Go projection and TypeScript protocol.
- Battle selectors and components rely on the same `deriveBattleState` contract.
- Action composer emits existing backend `ActionKind` strings only.

Plan complete and saved to `docs/superpowers/plans/2026-04-02-playable-battle-frontend.md`. Two execution options:

1. Subagent-Driven (recommended) - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. Inline Execution - Execute tasks in this session using executing-plans, batch execution with checkpoints

Given your instruction "无需确认", execution will proceed with option 2 (Inline Execution) immediately.
