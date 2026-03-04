import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 60_000,
  expect: {
    timeout: 8_000,
  },
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "on-first-retry",
  },
  grepInvert: /@live/,
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "VITE_EXECUTION_MODE=mock npm run dev -- --host 127.0.0.1 --port 4173",
    url: "http://127.0.0.1:4173",
    timeout: 120_000,
    reuseExistingServer: !process.env.CI,
  },
});
