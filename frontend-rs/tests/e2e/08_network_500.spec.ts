import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("08_network_500: 500 response shows error toast and preserves state", async ({ page, context }) => {
    // Simulate backend 500 on PUT
    await context.route("**/api/v1/diagrams/*", (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({ status: 500, body: "Internal Server Error" });
      } else {
        route.continue();
      }
    });

    // Trigger a save (create table to set dirty)
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "t");
    await page.click('[data-testid="btn-confirm"]');

    // Wait for debounce + network
    await page.waitForTimeout(1_500);

    // AC-12: error toast visible
    await expect(page.locator('[data-testid="error-toast"]')).toBeVisible();

    // State preserved: save button still present
    await expect(page.locator('[data-testid="btn-save"]')).toBeVisible();
  });

  test("08_network_500_reload: reload button in dialog restores state", async ({ page, context }) => {
    await context.route("**/api/v1/diagrams/*", async (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({
          status: 409,
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ code: 409, message: "conflict", request_id: "test", details: { current_revision: 1 } }),
        });
      } else {
        await route.continue();
      }
    });

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "reload_test");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);

    // Click reload
    await page.click('[data-testid="btn-reload"]');

    // Dialog closes
    await expect(page.locator('[data-testid="conflict-dialog"]')).not.toBeVisible();
  });
});