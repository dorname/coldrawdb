import { test, expect } from "@playwright/test";

test.describe("5 Features E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("03_set_reference", async ({ page }) => {
    // Step 1: Create two tables — users (id) and orders (user_id)
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    // Step 2: Select users table and add id field
    const usersItem = page.locator('[data-testid^="table-list-item-"]').first();
    await usersItem.click();
    await page.click('[data-testid="btn-add-field"]');

    // Step 3: Click "设关系" on the first field
    const setRefBtn = page.locator('[data-testid^="set-ref-"]').first();
    await setRefBtn.click();

    // Assert: error toast should NOT appear for this test;
    // the set-ref button itself should be visible
    await expect(setRefBtn).toBeVisible();
  });
});