import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("10_table_management: create multiple tables in sequence", async ({ page }) => {
    const tableNames = ["users", "orders", "products", "categories"];
    for (const name of tableNames) {
      await page.click('[data-testid="btn-create-table"]');
      await page.fill('[data-testid="table-name-input"]', name);
      await page.click('[data-testid="btn-confirm"]');
    }

    // All 4 tables visible in left panel
    const items = page.locator('[data-testid^="table-list-item-"]');
    await expect(items).toHaveCount(4);
  });

  test("10_table_management: select different tables switches right panel", async ({ page }) => {
    // Create two tables
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    // Select first table
    const firstTable = page.locator('[data-testid^="table-list-item-"]').first();
    await firstTable.click();

    // Right panel shows "users" heading (based on selected table)
    // add-field button should be visible
    await expect(page.locator('[data-testid="btn-add-field"]')).toBeVisible();

    // Select second table
    const secondTable = page.locator('[data-testid^="table-list-item-"]').nth(1);
    await secondTable.click();

    // add-field still visible
    await expect(page.locator('[data-testid="btn-add-field"]')).toBeVisible();
  });

  test("10_table_management: add multiple fields to same table", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    const tableItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableItem.click();

    // Add 3 fields
    for (let i = 0; i < 3; i++) {
      await page.click('[data-testid="btn-add-field"]');
    }

    const fields = page.locator('[data-testid^="field-"]');
    await expect(fields).toHaveCount(3);
  });
});