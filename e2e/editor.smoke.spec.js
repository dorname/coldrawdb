import { test, expect } from "@playwright/test";

test.describe("Editor smoke", () => {
  test("访问 /editor 页面不白屏", async ({ page }) => {
    await page.goto("/editor");
    await expect(page.locator(".theme").first()).toBeVisible();
  });
});

