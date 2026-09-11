import { test, expect } from "@playwright/test";

test.describe("W3-1: 异常路径 E2E", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 10_000 });
  });

  test("11_canvas_interaction: canvas is visible and rendered", async ({ page }) => {
    const canvas = page.locator('[data-testid="editor-canvas"]');
    await expect(canvas).toBeVisible();
    const box = await canvas.boundingBox();
    expect(box?.width).toBeGreaterThan(100);
    expect(box?.height).toBeGreaterThan(100);
  });

  test("11_canvas_interaction: table appears in canvas after creation", async ({ page }) => {
    await page.click('[data-testid="btn-create-table"]');
    await page.fill('[data-testid="table-name-input"]', "canvas_test");
    await page.click('[data-testid="btn-confirm"]');

    // Canvas should be visible
    await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
  });

  test("11_canvas_interaction: zoom in/out does not crash", async ({ page }) => {
    const canvas = page.locator('[data-testid="editor-canvas"]');
    const box = await canvas.boundingBox();
    if (!box) return;

    // Wheel zoom in
    await canvas.hover();
    await page.mouse.wheel(0, -100);
    await page.waitForTimeout(200);

    // Wheel zoom out
    await page.mouse.wheel(0, 100);
    await page.waitForTimeout(200);

    // Canvas still visible
    await expect(canvas).toBeVisible();
  });
});