import { expect, test, type Page } from "@playwright/test";

const owner = process.env.GH_TEST_OWNER ?? "";
const repo = process.env.GH_TEST_REPO ?? "";

test.describe("@live frontend live flow", () => {
  test.skip(process.env.GH_CLIENT_LIVE_TEST !== "1", "set GH_CLIENT_LIVE_TEST=1");
  test.skip(!owner || !repo, "set GH_TEST_OWNER and GH_TEST_REPO");

  test("read flow via bridge", async ({ page }) => {
    await page.goto("/");

    await setContext(page, owner, repo, "admin");

    await runCommand(page, "auth.status");
    await runCommand(page, "repo.list");

    await page.goto("/pull-requests");
    await setContext(page, owner, repo, "admin");
    await runCommand(page, "pr.list");

    await page.goto("/issues");
    await setContext(page, owner, repo, "admin");
    await runCommand(page, "issue.list");

    await page.goto("/actions");
    await setContext(page, owner, repo, "admin");
    await runCommand(page, "run.list");

    await page.goto("/releases");
    await setContext(page, owner, repo, "admin");
    await runCommand(page, "release.list");

    await page.goto("/settings");
    await setContext(page, owner, repo, "admin");
    await runCommand(page, "settings.collaborators.list");

    await page.goto("/p2");
    await setContext(page, owner, repo, "admin");
    await runCommandOptional(page, "projects.list", ["read:project", "permission_denied", "auth_required"]);
  });

  test("write flow opt-in", async ({ page }) => {
    test.skip(process.env.GH_CLIENT_LIVE_WRITE_TEST !== "1", "set GH_CLIENT_LIVE_WRITE_TEST=1");

    await page.goto("/issues");
    await setContext(page, owner, repo, "write");

    const title = `gh-client-ui-live-${Date.now()}`;

    await runCommand(page, "issue.create", {
      title,
      body: "created by frontend live e2e",
    });

    const created = await openLatestResponse(page, "issue.create");
    const createdPayload = JSON.parse(created) as { number?: number };
    const issueNumber = createdPayload.number;
    expect(issueNumber).toBeGreaterThan(0);

    await runCommand(page, "issue.comment", {
      number: String(issueNumber),
      body: "frontend live e2e comment",
    });

    await runCommand(page, "issue.close", {
      number: String(issueNumber),
      reason: "completed",
    });

    await runCommand(page, "issue.reopen", {
      number: String(issueNumber),
      comment: "frontend live e2e reopen",
    });
  });
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
    await expect(field).toBeVisible();

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
  await expect(card.getByRole("button", { name: /Response|レスポンス/ })).toBeVisible({ timeout: 45_000 });
}

async function runCommandOptional(
  page: Page,
  commandId: string,
  allowErrorPatterns: string[],
): Promise<void> {
  const card = page.locator(`[data-testid=\"command-${commandId}\"]`);
  await expect(card).toBeVisible({ timeout: 30_000 });
  await card.getByRole("button", { name: /Execute|実行/ }).first().click();

  const responseButton = card.getByRole("button", { name: /Response|レスポンス/ });
  const errorText = card.locator(".error-text").first();

  const settled = await Promise.race([
    responseButton.waitFor({ state: "visible", timeout: 45_000 }).then(() => "success" as const),
    errorText.waitFor({ state: "visible", timeout: 45_000 }).then(() => "error" as const),
  ]);

  if (settled === "success") {
    return;
  }

  const message = ((await errorText.textContent()) ?? "").toLowerCase();
  if (allowErrorPatterns.some((pattern) => message.includes(pattern.toLowerCase()))) {
    return;
  }

  throw new Error(`optional command ${commandId} failed: ${message}`);
}

async function openLatestResponse(page: Page, commandId: string): Promise<string> {
  const card = page.locator(`[data-testid=\"command-${commandId}\"]`);
  await card.getByRole("button", { name: /Response|レスポンス/ }).click();
  const block = page.locator(".drawer .json-block");
  await expect(block).toBeVisible();
  const text = await block.innerText();
  await page.getByRole("button", { name: /Close|閉じる/ }).click();
  return text;
}
