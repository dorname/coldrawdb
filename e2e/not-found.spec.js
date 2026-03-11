import { test, expect } from "@playwright/test";

test.describe("NotFound", () => {
  test("访问不存在路由出现 404 文案", async ({ page }) => {
    await page.goto("/non-existent");
    await expect(page.getByText(/looking for something/i)).toBeVisible();
  });
});

