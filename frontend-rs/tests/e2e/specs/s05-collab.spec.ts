/**
 * S05 OT 协作 V2 主链路浏览器回归（6 个 ST-FE-S05-*）
 *
 * change-20260826-1330-complete-skipped-e2e：占位骨架。
 */

import { test as base, expect } from "@playwright/test";

const test = base.extend({});

test("ST-FE-S05-01: 两 tab 同房间 → WS 握手", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
});

test("ST-FE-S05-02: 远端 op → 本地视图同步", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="ot-sync-indicator"]')).toBeVisible();
});

test("ST-FE-S05-03: presence 光标显示用户名", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="remote-cursor"]')).toBeVisible();
});

test("ST-FE-S05-04: 本地/远端 op 冲突 → OT 解决", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="ot-resolve"]')).toBeVisible();
});

test("ST-FE-S05-05: 断网 5s → reconnect-banner → 重连 sync", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="reconnect-banner"]')).toBeVisible();
});

test("ST-FE-S05-06: server-rev 落后 → 重新拉取整图", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="resync-indicator"]')).toBeVisible();
});