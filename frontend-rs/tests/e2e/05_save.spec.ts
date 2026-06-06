import { test, expect } from "@playwright/test";

test.describe("5 Features E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("05_save", async ({ page }) => {
    // Capture network requests
    const putRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
        putRequests.push(req.url());
      }
    });

    // Step 1: Create a table (triggers dirty state + debounce)
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Step 2: Wait for debounce (1.1s) + network request
    await page.waitForTimeout(1_100);

    // Assert: at least one PUT was issued
    expect(putRequests.length).toBeGreaterThanOrEqual(1);

    // Assert: revision display is visible
    await expect(page.locator('[data-testid="revision-display"]')).toBeVisible();

    // Assert: no error toast
    const errorToast = page.locator('[data-testid="error-toast"]');
    await expect(errorToast).not.toBeVisible();
  });
});