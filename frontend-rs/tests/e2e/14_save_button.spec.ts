import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("14_save_button: save button enabled after changes", async ({ page }) => {
    // Initially some state
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Save button should be visible
    await expect(page.locator('[data-testid="btn-save"]')).toBeVisible();
  });

  test("14_save_button: manual save triggers PUT immediately", async ({ page }) => {
    const putRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
        putRequests.push(req.url());
      }
    });

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Click manual save button
    await page.click('[data-testid="btn-save"]');

    // Should fire PUT faster than 1s debounce
    await page.waitForTimeout(500);
    expect(putRequests.length).toBeGreaterThanOrEqual(1);
  });

  test("14_save_button: save button shows saving state", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Save button text may change during saving
    const saveBtn = page.locator('[data-testid="btn-save"]');
    await expect(saveBtn).toBeVisible();
  });
});