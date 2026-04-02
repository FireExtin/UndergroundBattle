import { expect, test } from "@playwright/test";

// Purpose: Exercises a minimal real gameplay loop from the battle table UI.

test("battle table smoke: reset + attack + pass", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "对方玩家区域" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "争夺区" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "本方玩家区域" })).toBeVisible();
  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();

  await page.getByRole("button", { name: "重开对局" }).click();
  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();

  await page.getByLabel("Action Kind").selectOption("declare_attack");
  await page.getByLabel("Source Card").selectOption("P1-TABLE-1");
  await page.getByLabel("Target Card").selectOption("P2-TABLE-1");
  await page.getByRole("button", { name: "提交动作" }).click();

  await expect(page.getByText("Source: Live Sandbox")).toBeVisible();

  await page.getByRole("button", { name: "Pass Priority" }).click();
  await expect(page.getByText("本方玩家区域")).toBeVisible();
});
