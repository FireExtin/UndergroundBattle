import { expect, test, type APIRequestContext } from "@playwright/test";

// Purpose: Exercises a minimal real gameplay loop from the battle table UI.

test("battle table combo actions: attack + investigation + move + marker + pass", async ({ page, request }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "对方玩家区域" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "争夺区" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "本方玩家区域" })).toBeVisible();
  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();
  await expect(page.getByRole("button", { name: "The Silver District" })).toBeVisible();
  await expect(page.getByRole("button", { name: "The Ash Quarter" })).toBeVisible();
  await expect(page.getByRole("button", { name: "The Gilded Gate" })).toBeVisible();

  await page.getByRole("button", { name: "重开对局" }).click();
  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();

  await postAction(request, {
    id: "act-e2e-combo-pass-to-p2",
    actorId: "P1",
    kind: "pass_priority"
  });
  await postAction(request, {
    id: "act-e2e-combo-investigate",
    actorId: "P2",
    kind: "declare_investigation",
    cardId: "P2-TABLE-1",
    targetCardId: "REGION-2"
  });
  await postAction(request, {
    id: "act-e2e-combo-move",
    actorId: "P2",
    kind: "move_card",
    cardId: "P2-TABLE-1",
    targetCardId: "REGION-3"
  });
  await postAction(request, {
    id: "act-e2e-combo-pass-to-p1",
    actorId: "P2",
    kind: "pass_priority"
  });
  await postAction(request, {
    id: "act-e2e-combo-attack",
    actorId: "P1",
    kind: "declare_attack",
    cardId: "P1-TABLE-1",
    targetCardId: "P2-TABLE-1"
  });

  await page.getByRole("button", { name: "刷新状态" }).click();
  await expect(page.getByText("Actor: P1")).toBeVisible();

  await page.getByLabel("Action Kind").selectOption("set_marker");
  await page.getByLabel("Target Player").selectOption("P1");
  await page.getByLabel("Marker Type").fill("secret_society");
  await page.getByLabel("Marker Amount").fill("1");
  await page.getByRole("button", { name: "提交动作" }).click();

  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();

  await page.getByRole("button", { name: "Pass Priority" }).click();
  await expect(page.getByText(/ACCEPTED/).first()).toBeVisible();
  await expect(page.getByText("本方玩家区域")).toBeVisible();
});

test("finished match disables actions and latest report endpoint is readable", async ({ page, request }) => {
  const resetResponse = await request.post("/api/debugger/reset");
  expect(resetResponse.ok()).toBeTruthy();

  await postAction(request, {
    id: "act-e2e-report-investigate-1",
    actorId: "P1",
    kind: "declare_investigation",
    cardId: "P1-TABLE-1",
    targetCardId: "REGION-1"
  });

  for (const action of [
    { id: "act-e2e-phase-1", actorId: "P1", kind: "advance_phase" },
    { id: "act-e2e-phase-2", actorId: "P1", kind: "advance_phase" },
    { id: "act-e2e-phase-3", actorId: "P2", kind: "advance_phase" },
    { id: "act-e2e-phase-4", actorId: "P2", kind: "advance_phase" },
    { id: "act-e2e-phase-5", actorId: "P1", kind: "advance_phase" },
    { id: "act-e2e-phase-6", actorId: "P1", kind: "advance_phase" }
  ]) {
    await postAction(request, action);
  }

  await page.goto("/");
  await page.getByRole("button", { name: "刷新状态" }).click();

  await expect(page.getByText("Game over. Winner: P1").first()).toBeVisible();
  await expect(page.getByRole("button", { name: "提交动作" })).toBeDisabled();

  const reportResponse = await request.get("/api/debugger/reports/latest");
  expect(reportResponse.ok()).toBeTruthy();
  const report = (await reportResponse.json()) as {
    winnerPlayerId?: string;
    content?: string;
  };
  expect(report.winnerPlayerId).toBe("P1");
  expect(report.content ?? "").toContain("# Match Report");
});

async function postAction(
  request: APIRequestContext,
  action: {
    id: string;
    actorId: "P1" | "P2";
    kind: string;
    cardId?: string;
    targetCardId?: string;
    targetPlayerId?: string;
  }
) {
  const response = await request.post("/api/debugger/actions", {
    data: action
  });
  expect(response.ok()).toBeTruthy();
}
