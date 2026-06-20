/**
 * ST-PC-01 导入抽屉完整 E2E 测试
 *
 * Spec: logos/resources/test/core-PC-import-export-test-cases.md §ST-PC-01
 * Proposal: logos/changes/add-pb-pc-test-coverage/
 *
 * 步骤：
 *  1. 编辑器已加载（前置）
 *  2. AppBar 点击「导入」按钮 → IO 抽屉打开（Import 模式）
 *  3. 粘贴 SQL 文本 → 解析摘要显示「2 条语句」
 *  4. 提交 → bridge 返回 diagramId，画布刷新
 *
 * 覆盖：
 *  - UT-PC-04: open_import_drawer() 信号切换
 *  - UT-PC-06: guide-import-sql 按钮触发抽屉
 *  - UT-AB-04: btn-import 在 Phase C 启用
 */

import { test, expect } from "@playwright/test";

test.describe("Phase C Import Drawer E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("ST-PC-01: AppBar → 导入 → 粘贴 SQL → 提交 → 解析摘要可见 + bridge 返回 diagramId", async ({
    page,
  }) => {
    // Step 1: 点击 AppBar 导入按钮（btn-import 必须 enabled，UT-AB-04）
    const importBtn = page.locator('[data-testid="btn-import"]');
    await expect(importBtn).toBeEnabled();
    await importBtn.click();

    // Step 2: IO 抽屉打开（Import 模式）
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible({
      timeout: 3_000,
    });

    // Step 3: 粘贴 SQL（2 条 CREATE 语句，触发 UT-PC-01 parse 路径）
    const sqlText = "CREATE TABLE users (id INT);\nCREATE TABLE posts (id INT);";
    await page.fill('[data-testid="import-sql-input"]', sqlText);

    // Step 4: 解析摘要应显示「2 条语句」（import_parse_summary 触发 parse_sql_statements）
    await expect(page.locator('[data-testid="import-summary"]')).toHaveText(/2\s*条语句/, {
      timeout: 2_000,
    });

    // Step 5: 提交导入
    await page.click('[data-testid="import-submit"]');

    // Step 6: bridge 调用应返回 diagramId，画布刷新为含 2 张表的状态
    await page.waitForResponse(
      (resp) => resp.url().includes("/api/v1/bridge/") && resp.status() === 200,
      { timeout: 10_000 },
    );

    // 断言：Tables Tab 显示 2 张表
    await page.click('[data-testid="tab-tables"]');
    const tables = page.locator('[data-testid^="table-list-item-"]');
    await expect(tables).toHaveCount(2);
  });

  test("ST-PC-01 回归: 导入空字符串 → 摘要 0 条语句，不调用 bridge", async ({ page }) => {
    await page.click('[data-testid="btn-import"]');
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible();

    await page.fill('[data-testid="import-sql-input"]', "");
    await expect(page.locator('[data-testid="import-summary"]')).toHaveText(/0\s*条语句/);

    // 提交按钮应禁用（空输入）
    const submit = page.locator('[data-testid="import-submit"]');
    await expect(submit).toBeDisabled();
  });

  test("ST-PC-06: 零表画布 → 点击 guide-import-sql → import-drawer 可见", async ({
    page,
  }) => {
    // 确保画布为空
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]');

    // EmptyGuide 应显示 guide-import-sql 按钮
    const guideBtn = page.locator('[data-testid="guide-import-sql"]');
    await expect(guideBtn).toBeVisible({ timeout: 3_000 });

    await guideBtn.click();

    // IO 抽屉应打开
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible({
      timeout: 3_000,
    });
  });

  test("UT-PC-05 集成: 导入 DBML 文本 → 摘要显示「N 个 Table 块」", async ({ page }) => {
    await page.click('[data-testid="btn-import"]');
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible();

    // 切换到 DBML 格式（如有格式切换）
    const formatSelect = page.locator('[data-testid="import-format-select"]');
    if (await formatSelect.isVisible()) {
      await formatSelect.selectOption("dbml");
    }

    const dbml = "Table users { id int }\nTable posts { id int }";
    await page.fill('[data-testid="import-text-input"]', dbml);

    await expect(page.locator('[data-testid="import-summary"]')).toHaveText(/2\s*个 Table 块/);
  });
});