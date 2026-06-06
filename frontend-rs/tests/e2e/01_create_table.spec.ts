import { test, expect } from "@playwright/test";

test.describe("5 Features E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("01_create_table", async ({ page }) => {
    // Click "建表" button
    await page.click('[data-testid="btn-create-table"]');

    // Fill table name
    await page.fill('[data-testid="table-name-input"]', "users");

    // Confirm
    await page.click('[data-testid="btn-confirm"]');

    // Assert table appears in left panel list
    const tableListItem = page.locator('[data-testid^="table-list-item-"]');
    await expect(tableListItem).toBeVisible();

    // Assert canvas rendered (editor-canvas present)
    await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
  });
});