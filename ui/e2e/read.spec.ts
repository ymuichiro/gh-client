import { expect, test, type Page } from "@playwright/test";

test("mock read flow across feature pages", async ({ page }) => {
  await page.goto("/");

  await setContext(page, "mock-user", "gh-client", "admin");

  await runCommand(page, "auth.status");
  await runCommand(page, "repo.list");

  await page.goto("/repositories");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "repo.branches.list");

  await page.goto("/pull-requests");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "pr.list");

  await page.goto("/issues");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "issue.list");

  await page.goto("/actions");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "workflow.list");
  await runCommand(page, "run.list");

  await page.goto("/releases");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "release.list");

  await page.goto("/settings");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "settings.collaborators.list");

  await page.goto("/p2");
  await setContext(page, "mock-user", "gh-client", "admin");
  await runCommand(page, "projects.list");
  await runCommand(page, "discussions.list");
  await runCommand(page, "insights.views.get");

  await page.goto("/console");
  await expect(page.getByRole("heading", { name: "Command Console" })).toBeVisible();
  await runCommand(page, "repo.branch.ref.get", { args: "repos/mock-user/gh-client/git/ref/heads/main" });
});

async function setContext(page: Page, owner: string, repo: string, permission: "viewer" | "write" | "admin") {
  const context = page.locator(".context-grid");
  await context.locator("input").nth(0).fill(owner);
  await context.locator("input").nth(1).fill(repo);
  await context.locator("select").first().selectOption(permission);
}

async function runCommand(
  page: Page,
  commandId: string,
  fields: Record<string, string | boolean> = {},
): Promise<void> {
  const card = page.locator(`[data-testid=\"command-${commandId}\"]`);
  await expect(card).toBeVisible({ timeout: 30_000 });

  for (const [name, value] of Object.entries(fields)) {
    const field = card.locator(`[data-field=\"${name}\"]`).first();

    if (typeof value === "boolean") {
      if (value) {
        await field.check();
      } else {
        await field.uncheck();
      }
      continue;
    }

    const tagName = await field.evaluate((element) => element.tagName.toLowerCase());
    if (tagName === "select") {
      await field.selectOption(value);
      continue;
    }

    await field.fill(value);
  }

  await card.getByRole("button", { name: /Execute|実行/ }).first().click();
  await expect(card.getByRole("button", { name: /Response|レスポンス/ })).toBeVisible({ timeout: 30_000 });
}
