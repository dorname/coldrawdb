import { test, expect } from "@playwright/test";

test.describe("Landing", () => {
  test("访问 / 首屏可见", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/drawDB/i);
    // 兜底：页面至少有一个可见的导航/链接
    await expect(page.getByRole("link").first()).toBeVisible();
  });
});

