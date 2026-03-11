import { test, expect } from "@playwright/test";

test.describe("Templates", () => {
  test("访问 /templates 并成功加载列表", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("link", { name: "Templates" }).first().click();

    // 冒烟级断言：前端路由成功切换到 /templates
    await expect(page).toHaveURL(/\/templates$/);
  });
});

