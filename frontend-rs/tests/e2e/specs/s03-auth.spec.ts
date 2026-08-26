/**
 * S03 鉴权 V2 主链路浏览器回归（5 个 ST-FE-S03-*）
 *
 * change-20260826-1330-complete-skipped-e2e：本文件对应 ST-FE-S03-01~05。
 * 每个 test title 必须以 ST-FE-S03-NN 开头（OpenLogos reporter 提取 ID）。
 * 当前为占位骨架——真实流程待 Playwright + 后端联调实现。
 */

import { test as base, expect } from "@playwright/test";

const test = base.extend({});

test("ST-FE-S03-01: register → 跳转 home", async ({ page }) => {
  // TODO: 实现 register 流程
  await page.goto("/register");
  await expect(page.locator('[data-testid="register-form"]')).toBeVisible();
});

test("ST-FE-S03-02: login → user-menu 显示用户名", async ({ page }) => {
  await page.goto("/login");
  await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
});

test("ST-FE-S03-03: refresh token → 401 → 自动重新登录", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL(/\/(login|$)/);
});

test("ST-FE-S03-04: logout → token 失效 → 受保护路由跳转", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL(/\/login/);
});

test("ST-FE-S03-05: 未登录访问 /rooms → 跳 /login", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page).toHaveURL(/\/login/);
});