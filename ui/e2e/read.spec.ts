import { expect, test, type Page } from "@playwright/test";

test("mock read flow for issues, pull requests and settings", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: /対象リポジトリ設定|Repository Scope/ })).toBeVisible();
  await configureRepositoryScope(page, "mock-user", "gh-client");
  await expect(page.getByRole("heading", { name: "Issues" })).toBeVisible();

  await page.goto("/settings");
  await expect(page.getByRole("heading", { name: /対象リポジトリ設定|Repository Scope/ })).toBeVisible();

  await page.goto("/issues");
  await expect(page.getByRole("heading", { name: "Issues" })).toBeVisible();
  await page.getByRole("button", { name: "一覧を更新" }).click();
  await expect(page.locator(".queue-item").first()).toBeVisible();

  const mockUserIssue = page
    .locator(".queue-item", { hasText: "mock-user/gh-client" })
    .filter({ hasText: "Mock issue" })
    .first();
  const mockUserClosedIssue = page
    .locator(".queue-item", { hasText: "mock-user/gh-client" })
    .filter({ hasText: "Closed issue sample" })
    .first();

  await mockUserIssue.click();
  await page.keyboard.press("a");
  await expect(page.getByRole("heading", { name: "Approve 実行確認" })).toHaveCount(0);
  await page.getByRole("button", { name: /一覧に戻る|Back to list/ }).click();

  await mockUserClosedIssue.click();
  await expect(page.getByRole("button", { name: "close (C)" })).toBeDisabled();
  await page.keyboard.press("r");
  await expect(page.getByRole("heading", { name: "コメント投稿" })).toBeVisible();
  await page.getByRole("button", { name: "キャンセル" }).click();
  await page.getByRole("button", { name: /一覧に戻る|Back to list/ }).click();

  await page.getByRole("button", { name: "表示中 open を一括 close" }).click();
  await expect(page.getByRole("heading", { name: "一括 close 実行" })).toBeVisible();

  await page.getByRole("button", { name: "一括 close 実行" }).click();
  await expect(page.locator(".inbox-modal-panel .error-text").first()).toBeVisible();

  const token = (await page.getByText(/^BATCH:CLOSE:/).innerText()).trim();
  await page.getByPlaceholder(/BATCH:CLOSE/).fill(token);
  await page.getByRole("button", { name: "一括 close 実行" }).click();

  await expect(page.getByText(/一括 close 完了:/)).toBeVisible();

  await page.goto("/pull-requests");
  await expect(page.getByRole("heading", { name: "Pull Requests" })).toBeVisible();
  await page.getByRole("button", { name: "一覧を更新" }).click();
  await expect(page.locator(".queue-item").first()).toBeVisible();

  const mockUserPullRequest = page
    .locator(".queue-item", { hasText: "mock-user/gh-client" })
    .filter({ hasText: "Mock pull request" })
    .first();

  await mockUserPullRequest.click();
  await page.keyboard.press("a");
  await expect(page.getByRole("heading", { name: "Approve 実行確認" })).toBeVisible();
  await page.getByRole("button", { name: "キャンセル" }).click();

  await page.locator(".thread-item", { hasText: "ui/src/pages/InboxPage.tsx" }).click();
  await expect(page.locator(".diff-file.active", { hasText: "ui/src/pages/InboxPage.tsx" })).toBeVisible();
});

async function configureRepositoryScope(page: Page, owner: string, repo: string): Promise<void> {
  await page.getByRole("button", { name: /org 候補を読み込む|Load organizations/ }).click();
  await page.locator(".scope-option", { hasText: owner }).locator('input[type="checkbox"]').check();
  await page
    .getByRole("button", {
      name: /repo 候補を更新|Update repositories from checked organizations/,
    })
    .click();
  await page.locator(".scope-option", { hasText: repo }).locator('input[type="checkbox"]').check();
  await page.getByRole("button", { name: /取得対象として確定|Confirm/ }).click();
}
