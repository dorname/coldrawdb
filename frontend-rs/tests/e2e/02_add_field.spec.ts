import { test, expect } from "@playwright/test";

test.describe("5 Features E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("02_add_field", async ({ page }) => {
    // Step 1: Create a table first
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Step 2: Select the table in left panel
    const tableListItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableListItem.click();

    // Step 3: Click "加字段" button
    await page.click('[data-testid="btn-add-field"]');

    // Assert: at least one field row appears (field-{id} pattern)
    const fieldRow = page.locator('[data-testid^="field-"]');
    await expect(fieldRow.first()).toBeVisible();
  });
});