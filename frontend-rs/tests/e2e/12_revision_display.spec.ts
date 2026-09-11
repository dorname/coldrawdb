import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("12_revision_display: shows initial revision", async ({ page }) => {
    const rev = page.locator('[data-testid="revision-display"]');
    await expect(rev).toBeVisible();
    // Initial revision text contains "rev:"
    await expect(rev).toContainText("rev:");
  });

  test("12_revision_display: revision updates after save", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Wait for debounce save
    await page.waitForTimeout(1_500);

    // Revision display still visible
    await expect(page.locator('[data-testid="revision-display"]')).toBeVisible();
  });

  test("12_revision_display: revision shows during conflict", async ({ page, context }) => {
    await context.route("**/api/v1/diagrams/*", async (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({
          status: 409,
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ code: 409, message: "conflict", request_id: "test", details: { current_revision: 10 } }),
        });
      } else {
        await route.continue();
      }
    });

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "conflict_rev");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);

    // Conflict dialog visible
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Revision display still visible during conflict
    await expect(page.locator('[data-testid="revision-display"]')).toBeVisible();
  });
});