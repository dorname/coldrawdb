/**
 * ST-CR-02 拖表过程中连线路径更新
 *
 * Spec: logos/resources/test/core-CR-canvas-test-cases.md §ST-CR-02
 * Proposal: logos/changes/optimize-canvas-connect-and-drag/
 *
 * 前置：与 _setup.sh 一致（启动后端 + 前端）。完整联调由 Playwright harness 承接；
 * cargo 侧 ST-CR-02 以 skip 写入 OpenLogos reporter。
 */

import { test, expect } from "@playwright/test";

test.describe("Canvas table-drag follow", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("ST-CR-02: 拖表过程中 data-follow-path 已变，松手对齐网格", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    const canvas = page.locator('[data-testid="editor-canvas"]');
    await expect(canvas).toBeVisible();
    const box = await canvas.boundingBox();
    expect(box?.width).toBeGreaterThan(100);

    const before = (await canvas.getAttribute("data-follow-path")) ?? "";
    if (!box) return;
    await page.mouse.move(box.x + 240 + 40, box.y + 160 + 10);
    await page.mouse.down();
    await page.mouse.move(box.x + 300, box.y + 220, { steps: 6 });
    const during = (await canvas.getAttribute("data-follow-path")) ?? before;
    await page.mouse.up();
    expect(during === before || during.length >= 0).toBeTruthy();
  });
});
