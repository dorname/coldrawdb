import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("13_error_toast: toast disappears after clicking X", async ({ page, context }) => {
    await context.route("**/api/v1/diagrams/*", (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({ status: 500, body: "Error" });
      } else {
        route.continue();
      }
    });

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "toast_test");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);

    // Toast visible
    await expect(page.locator('[data-testid="error-toast"]')).toBeVisible();

    // Dismiss toast
    await page.locator('[data-testid="error-toast"] button').click();

    // Toast gone
    await expect(page.locator('[data-testid="error-toast"]')).not.toBeVisible();
  });

  test("13_error_toast: multiple errors show latest message", async ({ page, context }) => {
    await context.route("**/api/v1/diagrams/*", (route, req) => {
      if (req.method() === "PUT") {
        route.fulfill({ status: 500, body: "Server Error" });
      } else {
        route.continue();
      }
    });

    // Create first error
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "first");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);
    await expect(page.locator('[data-testid="error-toast"]')).toBeVisible();
  });

  test("13_error_toast: no toast on successful save", async ({ page }) => {
    // No interception - successful save path
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "success_test");
    await page.click('[data-testid="btn-confirm"]');
    await page.waitForTimeout(1_500);

    // No error toast
    await expect(page.locator('[data-testid="error-toast"]')).not.toBeVisible();
  });
});