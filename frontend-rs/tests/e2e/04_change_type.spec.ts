import { test, expect } from "@playwright/test";

test.describe("5 Features E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("04_change_type", async ({ page }) => {
    // Step 1: Create table with a field
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Step 2: Select table
    const tableListItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableListItem.click();

    // Step 3: Add a field (which gets default type VARCHAR(255))
    await page.click('[data-testid="btn-add-field"]');

    // Step 4: Change type dropdown from VARCHAR(255) to INT
    const typeDropdown = page.locator('[data-testid^="type-"]').first();
    await typeDropdown.selectOption("INT");

    // Assert: dropdown value is now INT
    await expect(typeDropdown).toHaveValue("INT");
  });
});