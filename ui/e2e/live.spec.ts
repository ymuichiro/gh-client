import { expect, test, type Page } from "@playwright/test";

const owner = process.env.GH_TEST_OWNER ?? "";
const repo = process.env.GH_TEST_REPO ?? "";

test.describe("@live frontend live flow", () => {
  test.skip(process.env.GH_CLIENT_LIVE_TEST !== "1", "set GH_CLIENT_LIVE_TEST=1");
  test.skip(!owner || !repo, "set GH_TEST_OWNER and GH_TEST_REPO");

  test("read flow via bridge", async ({ page }) => {
    await page.addInitScript(
      ([scopeOwner, scopeRepo]) => {
        const payload = {
          orgs: [scopeOwner],
          repositories: [
            {
              owner: scopeOwner,
              repo: scopeRepo,
              viewerPermission: "admin",
            },
          ],
          updatedAt: new Date().toISOString(),
        };
        localStorage.setItem("gh-client-repository-scope", JSON.stringify(payload));
      },
      [owner, repo],
    );

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Issues" })).toBeVisible();

    await page.goto("/settings");
    await expect(page.getByRole("heading", { name: /対象リポジトリ設定|Repository Scope/ })).toBeVisible();

    await page.goto("/issues");
    await expect(page.getByRole("heading", { name: "Issues" })).toBeVisible();
    await page.getByRole("button", { name: /一覧を更新|Refresh list/ }).click();
    await expect(page.locator(".queue-item").first()).toBeVisible();

    await page.goto("/pull-requests");
    await expect(page.getByRole("heading", { name: "Pull Requests" })).toBeVisible();
    await page.getByRole("button", { name: /一覧を更新|Refresh list/ }).click();
    await expect(page.locator(".queue-item").first()).toBeVisible();
  });
});
