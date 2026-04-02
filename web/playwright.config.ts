import { defineConfig, devices } from "@playwright/test";

// Purpose: Runs battle-table smoke tests against real local Go sandbox + Vite web client.
export default defineConfig({
  testDir: "./tests",
  timeout: 60_000,
  expect: {
    timeout: 10_000
  },
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "on-first-retry"
  },
  webServer: [
    {
      command: "cd .. && PORT=8080 go run ./server/cmd/api",
      url: "http://127.0.0.1:8080/api/debugger/messages",
      timeout: 120_000,
      reuseExistingServer: true
    },
    {
      command: "npm run dev -- --host 127.0.0.1 --port 4173",
      url: "http://127.0.0.1:4173",
      timeout: 120_000,
      reuseExistingServer: true
    }
  ],
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"]
      }
    }
  ]
});
