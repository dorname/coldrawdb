/**
 * ST-PE-07（fix-remote-github-issues-7-18，issue #13/#14/#15）
 *
 * e2e 长房间名 AppBar 布局（core-05「AppBar 统一工作空间信息分层补充」§ST-PE-07）
 *  GIVEN：房间名 ≥24 字符
 *  WHEN：进入房间编辑器（视口 1440 → 降至 1280）
 *  THEN：
 *   1. room-badge 文本省略截断（strong scrollWidth > clientWidth），title 悬停全文兜底
 *   2. btn-invite 可见且可点击（点击打开 modal-invite，未被挤出/遮挡）
 *   3. 返回按钮「← 空间」图标与文字单一横排（中心线对齐、无换行堆叠），可点高 ≥36px
 *   4. diagram-title 带全文 title
 *
 * reporter：OpenLogos reporter 按 PREFIX_MAP（"appbar-truncate.spec.ts": "ST-"）提取
 * ST-PE-07 追加 jsonl。
 */

import { test, expect, type Page } from "@playwright/test";
import { registerAndLogin, createRoomWithDiagram, newRoomPage } from "../helpers/collab";

/** 31 字符长房间名（≥24 字符规格下限；后端按字节限 64，中英混排 51 字节） */
const LONG_NAME = "数仓核心域协作房间-Warehouse-Core-Model";

/** 返回按钮图标与文字同一横排（中心线偏差 <4px，图标在左、文字在右），按钮高 ≥36px */
async function expectBackButtonSingleRow(page: Page) {
  const btn = page.locator('[data-testid="btn-back-to-rooms"]');
  await expect(btn).toBeVisible();
  const bb = await btn.boundingBox();
  const iconBox = await page.locator('[data-testid="btn-back-to-rooms"] svg').boundingBox();
  // IconBox 图标外也有 <span class="cdb-icon-wrap"> 包裹——取按钮的最后一个直接子 span（文字）
  const textBox = await page
    .locator('[data-testid="btn-back-to-rooms"] > span')
    .last()
    .boundingBox();
  if (!bb || !iconBox || !textBox) throw new Error("返回按钮缺布局盒");
  expect(bb.height, "R-AB-04：返回按钮可点高度 ≥36px").toBeGreaterThanOrEqual(36);
  const iconCy = iconBox.y + iconBox.height / 2;
  const textCy = textBox.y + textBox.height / 2;
  expect(
    Math.abs(iconCy - textCy),
    `R-AB-01：图标与文字必须同一横排（中心线偏差 ${Math.abs(iconCy - textCy)}px ≥ 4px 视为堆叠）`,
  ).toBeLessThan(4);
  expect(iconBox.x + iconBox.width, "图标必须在文字左侧（横排）").toBeLessThanOrEqual(
    textBox.x + 1,
  );
}

test.describe("fix-remote-github-issues-7-18 AppBar 长房间名", () => {
  test("ST-PE-07: 长房间名 → badge 截断+title 全文+邀请可点+返回横排（1440→1280）", async ({
    browser,
    request,
  }) => {
    const owner = await registerAndLogin(request, "pe07");
    const { roomId } = await createRoomWithDiagram(request, owner, LONG_NAME);
    const { page } = await newRoomPage(browser, owner, roomId);
    const appBar = page.locator('[data-testid="app-bar"]');
    await expect(appBar).toBeVisible();

    // ── Step 1：1440 视口——返回按钮单一横排；badge 与邀请按钮同一行 ──
    await page.setViewportSize({ width: 1440, height: 900 });
    await expectBackButtonSingleRow(page);
    const badge = page.locator('[data-testid="room-badge"]');
    const invite = page.locator('[data-testid="btn-invite"]');
    await expect(badge).toBeVisible();
    await expect(invite).toBeVisible();
    const badgeBox0 = await badge.boundingBox();
    const inviteBox0 = await invite.boundingBox();
    if (!badgeBox0 || !inviteBox0) throw new Error("badge/invite 缺布局盒");
    expect(
      Math.abs(badgeBox0.y + badgeBox0.height / 2 - (inviteBox0.y + inviteBox0.height / 2)),
      "R-AB-08：room-badge 与邀请按钮必须同一行（无换行挤出）",
    ).toBeLessThan(4);

    // ── Step 2：降至 1280 视口——badge 省略截断 + title 全文兜底 ──
    await page.setViewportSize({ width: 1280, height: 720 });
    // 等布局稳定（badge 宽度收敛）
    await page.waitForTimeout(300);
    const badgeStrong = page.locator('[data-testid="room-badge"] strong');
    await expect(badgeStrong).toBeVisible();
    const truncated = await badgeStrong.evaluate(
      (el) => el.scrollWidth > el.clientWidth,
    );
    expect(truncated, "R-AB-06：25 字符房间名在 1280 视口必须省略截断").toBe(true);
    await expect(badge).toHaveAttribute("title", `${LONG_NAME}（点击返回房间列表）`);
    await expect(page.locator('[data-testid="diagram-title"]')).toHaveAttribute(
      "title",
      LONG_NAME,
    );

    // ── Step 3：1280 视口下邀请按钮仍可见可点（点击打开 modal-invite）──
    await expect(invite).toBeVisible();
    await expect(invite).toBeEnabled();
    await expectBackButtonSingleRow(page); // 降视口后返回按钮仍单一横排
    await invite.click();
    await expect(page.locator('[data-testid="modal-invite"]')).toBeVisible({
      timeout: 5_000,
    });
  });
});
