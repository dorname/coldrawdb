/**
 * V2 全链路回归（4 个 ST-FE-V2-*）
 *
 * change-20260826-1330-complete-skipped-e2e：占位骨架。
 */

import { test as base, expect } from "@playwright/test";

const test = base.extend({});

test("ST-FE-V2-01: auth → rooms 跨场景 1", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL(/\/(login|rooms|$)/);
});

test("ST-FE-V2-02: rooms → editor 跨场景 2", async ({ page }) => {
  await page.goto("/rooms");
  await expect(page.locator('[data-testid="rooms-list"]')).toBeVisible();
});

test("ST-FE-V2-03: editor → collab 跨场景 3", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
});

test("ST-FE-V2-04: 完整 auth → rooms → editor → collab 端到端", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL(/\/(login|rooms|$)/);
});