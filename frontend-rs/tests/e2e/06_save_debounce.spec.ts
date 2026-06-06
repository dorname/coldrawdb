import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("06_save_debounce: 5 rapid changes trigger only 1 PUT after 1.1s", async ({ page }) => {
    const putRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
        putRequests.push(req.url());
      }
    });

    // 5 rapid table creations
    for (let i = 0; i < 5; i++) {
      await page.click('[data-testid="btn-create-table"]');
      await page.fill('[data-testid="table-name-input"]', `t${i}`);
      await page.click('[data-testid="btn-confirm"]');
      await page.waitForTimeout(50); // fast, < debounce
    }

    // Wait for debounce to fire (1s debounce + 200ms buffer)
    await page.waitForTimeout(1_200);

    // AC-10: exactly 1 PUT should fire
    expect(putRequests.length).toBe(1);
  });

  test("06_save_debounce_continuous: rapid type changes still debounce", async ({ page }) => {
    const putRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
        putRequests.push(req.url());
      }
    });

    // Create table + field
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');
    const tableItem = page.locator('[data-testid^="table-list-item-"]').first();
    await tableItem.click();
    await page.click('[data-testid="btn-add-field"]');

    // Rapid type changes
    const typeDrop = page.locator('[data-testid^="type-"]').first();
    await typeDrop.selectOption("INT");
    await page.waitForTimeout(100);
    await typeDrop.selectOption("BIGINT");
    await page.waitForTimeout(100);
    await typeDrop.selectOption("VARCHAR(255)");
    await page.waitForTimeout(100);

    await page.waitForTimeout(1_200);
    expect(putRequests.length).toBeLessThanOrEqual(1);
  });
});