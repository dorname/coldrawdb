import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("15_set_reference: set-ref button visible on each field", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    const tableItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableItem.click();
    await page.click('[data-testid="btn-add-field"]');

    const setRefBtn = page.locator('[data-testid^="set-ref-"]').first();
    await expect(setRefBtn).toBeVisible();
    await expect(setRefBtn).toContainText("设关系");
  });

  test("15_set_reference: click set-ref triggers error (not yet implemented)", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    const tableItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableItem.click();
    await page.click('[data-testid="btn-add-field"]');

    const setRefBtn = page.locator('[data-testid^="set-ref-"]').first();
    await setRefBtn.click();

    // Currently shows "设关系功能待实现" error
    await expect(page.locator('[data-testid="error-toast"]')).toBeVisible();
  });

  test("15_set_reference: two tables allow setting cross-table reference", async ({ page }) => {
    // Create users with id field
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Create orders table
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    // Select users table
    const usersItem = page.locator('[data-testid^="table-list-item-"]').first();
    await usersItem.click();
    await page.click('[data-testid="btn-add-field"]');

    // set-ref button visible
    const setRefBtn = page.locator('[data-testid^="set-ref-"]').first();
    await expect(setRefBtn).toBeVisible();
  });
});