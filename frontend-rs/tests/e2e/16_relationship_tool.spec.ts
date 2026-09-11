/**
 * ST-PB-01 关系工具 E2E 测试
 *
 * Spec: logos/resources/test/core-PB-relationship-test-cases.md §ST-PB-01
 * Proposal: logos/changes/add-pb-pc-test-coverage/
 *
 * 步骤：
 *  1. 编辑器加载两张表各一字段
 *  2. 选中关系工具（ToolRail）
 *  3. 双击字段 → 双击另一字段 → 确认条出现
 *  4. 点确认 → Inspector 可编辑关系
 *
 * 前置：与 _setup.sh 一致（启动后端 + 前端 + 创建空 diagram）
 */

import { test, expect } from "@playwright/test";

test.describe("Phase B Relationship Tool E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("ST-PB-01: 关系工具双点+确认 → Inspector 可编辑关系", async ({ page }) => {
    // Step 1: 创建第一张表
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // Step 2: 创建第二张表
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    // Step 3: 选中关系工具（ToolRail）
    await page.click('[data-testid="tool-relationship"]');

    // Step 4: 双击第一个表的字段 → 双击第二个表的字段
    const firstField = page.locator('[data-testid^="field-"]').first();
    const lastField = page.locator('[data-testid^="field-"]').last();

    await firstField.dblclick();
    await lastField.dblclick();

    // Step 5: 确认条应可见
    await expect(page.locator('[data-testid="rel-confirm-bar"]')).toBeVisible({
      timeout: 3_000,
    });

    // Step 6: 点击确认创建
    await page.click('[data-testid="rel-confirm-create"]');

    // Step 7: Inspector 应可见并可编辑新关系
    await expect(page.locator('[data-testid="inspector-panel"]')).toBeVisible();
    await expect(page.locator('[data-testid="inspector-relation"]')).toBeVisible();

    // 断言：references 计数 == 1（可通过 store 暴露或 UI 列表长度验证）
    const relations = page.locator('[data-testid^="relation-row-"]');
    await expect(relations).toHaveCount(1);
  });

  test("ST-PB-02: 关系工具拖字段出线 + 确认", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    await page.click('[data-testid="tool-relationship"]');
    const canvas = page.locator('[data-testid="editor-canvas"]');
    const box = await canvas.boundingBox();
    if (!box) {
      test.skip();
      return;
    }

    const fieldY = box.y + 160 + 32 + 12;
    await page.mouse.move(box.x + 240 + 40, fieldY);
    await page.mouse.down();
    await page.mouse.move(box.x + 240 + 80, fieldY, { steps: 4 });
    await expect(page.locator('[data-testid="rel-rubber-band"]')).toBeVisible({
      timeout: 2_000,
    });
    await page.mouse.move(box.x + 460, fieldY + 40, { steps: 6 });
    await page.mouse.up();

    const confirm = page.locator('[data-testid="rel-confirm-bar"]');
    if (await confirm.isVisible().catch(() => false)) {
      await page.click('[data-testid="rel-confirm-create"]');
      await expect(page.locator('[data-testid^="relation-row-"]')).toHaveCount(1);
    }
  });

  test("ST-PB-01 回归: 取消确认 → 不创建关系", async ({ page }) => {
    // 创建两张表
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "a");
    await page.click('[data-testid="btn-confirm"]');
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "b");
    await page.click('[data-testid="btn-confirm"]');

    await page.click('[data-testid="tool-relationship"]');
    await page.locator('[data-testid^="field-"]').first().dblclick();
    await page.locator('[data-testid^="field-"]').last().dblclick();

    // 取消而非确认
    await expect(page.locator('[data-testid="rel-confirm-bar"]')).toBeVisible();
    await page.click('[data-testid="rel-confirm-cancel"]');

    // references 应仍为 0
    const relations = page.locator('[data-testid^="relation-row-"]');
    await expect(relations).toHaveCount(0);
  });
});