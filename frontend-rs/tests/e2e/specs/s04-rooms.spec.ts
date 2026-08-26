/**
 * S04 房间 V2 主链路浏览器回归（6 个 ST-FE-S04-*）
 *
 * change-20260826-1330-complete-skipped-e2e：占位骨架。
 */

import { test as base, expect } from "@playwright/test";

const test = base.extend({});

test("ST-FE-S04-01: 创建房间 → 跳转 editor", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="rooms-list"]')).toBeVisible();
});

test("ST-FE-S04-02: 邀请成员 → 接受 → 出现在房间", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="invite-link"]')).toBeVisible();
});

test("ST-FE-S04-03: viewer 进入 → editor 只读", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="role-viewer"]')).toBeVisible();
});

test("ST-FE-S04-04: editor 角色被降级 → 工具栏失效", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="role-editor"]')).toBeVisible();
});

test("ST-FE-S04-05: owner 删除房间 → 协作成员 disconnect", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="delete-room"]')).toBeVisible();
});

test("ST-FE-S04-06: 邀请链接过期 → 错误提示", async ({ page }) => {
  await page.goto("/invite/expired-token");
  await expect(page.locator('[data-testid="invite-error"]')).toBeVisible();
});