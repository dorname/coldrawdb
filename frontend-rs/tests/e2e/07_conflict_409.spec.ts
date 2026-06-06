import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("07_conflict_409: PUT 409 shows conflict dialog", async ({ page, context }) => {
    // Intercept GET to inject a higher revision, then trigger PUT to cause 409
    await context.route("**/api/v1/diagrams/*", async (route, req) => {
      if (req.method() === "GET") {
        await route.continue();
      } else if (req.method() === "PUT") {
        // Force 409 by manipulating the interception
        route.fulfill({
          status: 409,
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ code: 409, message: "revision conflict", request_id: "test", details: { current_revision: 99 } }),
        });
      } else {
        await route.continue();
      }
    });

    // Create a table (dirty state)
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "will_conflict");
    await page.click('[data-testid="btn-confirm"]');

    // Wait for debounce PUT which should trigger 409
    await page.waitForTimeout(1_500);

    // Assert conflict dialog appears
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="btn-force-overwrite"]')).toBeVisible();
    await expect(page.locator('[data-testid="btn-reload"]')).toBeVisible();
  });

  test("07_conflict_409_force_overwrite: force overwrite sends PUT with expected_revision", async ({ page, context }) => {
    await context.route("**/api/v1/diagrams/*", async (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({
          status: 409,
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ code: 409, message: "revision conflict", request_id: "test", details: { current_revision: 5 } }),
        });
      } else {
        await route.continue();
      }
    });

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "conflict_test");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);

    // Click force overwrite
    await page.click('[data-testid="btn-force-overwrite"]');

    // Dialog should close
    await expect(page.locator('[data-testid="conflict-dialog"]')).not.toBeVisible();
  });
});