/**
 * ST-CR-COMMENT-01 / ST-CR-COLOR-01 / ST-PB-06（fix-remote-github-issues-7-18，issue #10/#12）
 *
 * ST-CR-COMMENT-01（core-01a §1.5 / core-01 §5.8 R-CMT-01~04）
 *  GIVEN：SQL 导入含中文 COMMENT 的表（表注释 + 字段注释）
 *  THEN：debug 通道 table_meta 有 comment；默认 name+comment 模式画布截图像素 ≠
 *        name 模式（注释被渲染/消失）；切到 comment 模式再变；localStorage 持久化；
 *        刷新重进后 comment 仍在
 *
 * ST-CR-COLOR-01（core-01a §1.6 / R-COLOR-01）
 *  GIVEN：画布一张表
 *  WHEN：Inspector 改表颜色 → 表头/边框即时更新（canvas 截图像素变化 + table_meta 落账）
 *  THEN：保存刷新后保留；导出 JSON 含 color/comment；再导入该 JSON 颜色仍在
 *
 * ST-PB-06（core-01b §4.1 / R-COLOR-02）
 *  GIVEN：books→authors 一条关系
 *  WHEN：Inspector 修改关系颜色 → 画布截图像素变化（该线变色）+ ref_meta 落账
 *  THEN：保存刷新后保留
 *
 * 断言通道：__cdb_debug_state().table_meta/ref_meta（数据层事实）+
 * canvas 元素截图像素对比（渲染层事实，自绘 canvas 无 DOM 可断言）。
 * reporter：OpenLogos reporter 按 PREFIX_MAP（"cr-comment-color.spec.ts": "ST-"）提取 ID。
 */

import { test, expect, type Page } from "@playwright/test";
import {
  registerAndLogin,
  createRoomWithDiagram,
  newRoomPage,
  debugState,
} from "../helpers/collab";

/** 导入按钮位于 AppBar 溢出菜单内 */
async function openAppBarImport(page: Page) {
  await page.click('[data-testid="btn-more-menu"]');
  const importBtn = page.locator('[data-testid="btn-import"]');
  await expect(importBtn).toBeVisible();
  await importBtn.click();
}

/** SQL/JSON 导入并等落账（debug tables 含目标表名） */
async function importSql(
  page: Page,
  sql: string,
  expectTable: string,
  format: "SQL" | "JSON" = "SQL",
) {
  await openAppBarImport(page);
  const drawer = page.locator('[data-testid="import-drawer"]');
  await expect(drawer).toBeVisible({ timeout: 3_000 });
  if (format === "JSON") {
    await drawer.locator('[data-testid="io-format-tabs"] button', { hasText: "JSON" }).click();
  }
  await page.fill('[data-testid="import-textarea"]', sql);
  // 摘要文案随格式而异：SQL/DBML 为「N 条语句」，JSON 为「N 张表」
  await expect(page.locator('[data-testid="import-parse-summary"]')).toHaveText(
    /[1-9]\d*\s*(条语句|张表)/,
  );
  await page.click('[data-testid="import-submit"]');
  await expect
    .poll(
      async () => (await debugState(page)).tables.includes(expectTable),
      { timeout: 8_000 },
    )
    .toBe(true);
}

/** canvas 元素截图（像素对比通道） */
async function canvasShot(page: Page): Promise<Buffer> {
  return page.locator('[data-testid="editor-canvas"]').screenshot();
}

/** 等画布稳定两帧后截图（精灵重光栅是异步的） */
async function stableShot(page: Page): Promise<Buffer> {
  await page.waitForTimeout(400);
  return canvasShot(page);
}

const COMMENT_DDL = `CREATE TABLE cms_users (
  id INT PRIMARY KEY COMMENT '用户编号',
  name VARCHAR(64) COMMENT '用户昵称'
) COMMENT='用户表';`;

const FK_DDL = `CREATE TABLE books (
  id INT PRIMARY KEY,
  author_id INT,
  FOREIGN KEY (author_id) REFERENCES authors(id)
);
CREATE TABLE authors (
  id INT PRIMARY KEY
);`;

test.describe("fix-remote-github-issues-7-18 画布注释与颜色", () => {
  test("ST-CR-COMMENT-01: 中文 COMMENT 导入 → 三态切换画布注释显隐 → 刷新仍在", async ({
    browser,
    request,
  }) => {
    const owner = await registerAndLogin(request, "cmt01");
    const { roomId } = await createRoomWithDiagram(request, owner, "cmt01-room");
    const { page } = await newRoomPage(browser, owner, roomId);
    await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();

    // GIVEN：导入含中文 COMMENT 的 SQL
    await importSql(page, COMMENT_DDL, "cms_users");
    const meta = (await debugState(page)).table_meta?.find((t) => t.name === "cms_users");
    expect(meta?.comment, "表注释必须落账（解析层）").toBe("用户表");

    // 显示开关存在且默认「英文名+注释」（R-CMT-04 缺省）
    const toggle = page.locator('[data-testid="canvas-comment-display"]');
    await expect(toggle).toBeVisible();
    await expect(toggle).toHaveText("注释：英文名+注释");

    // THEN 1：name+comment（有注释）与 name（无注释）画布像素必须不同——注释被真实渲染。
    // next() 循环：name → name+comment → comment → name；从默认 name+comment 出发
    // click ×2 到 name（仅英文名）。
    const withComment = await stableShot(page);
    await toggle.click(); // → comment
    await toggle.click(); // → name（仅英文名）
    await expect(toggle).toHaveText("注释：仅英文名");
    const nameOnly = await stableShot(page);
    expect(
      withComment.equals(nameOnly),
      "R-CMT-01/02：name+comment 与 name 模式画布必须不同（注释参与渲染）",
    ).toBe(false);

    // THEN 2：切到 comment（仅注释）——主文本换成注释，像素再变
    await toggle.click(); // → name+comment
    await toggle.click(); // → comment（仅注释）
    await expect(toggle).toHaveText("注释：仅注释");
    const commentOnly = await stableShot(page);
    expect(commentOnly.equals(nameOnly), "comment 模式主文本必须变为注释").toBe(false);

    // THEN 3：localStorage 持久化视图偏好（不落库 diagram）。
    // 循环 name → name+comment → comment → name：从 comment 回 name+comment 需 click ×2
    await toggle.click(); // → name
    await toggle.click(); // → name+comment
    await expect(toggle).toHaveText("注释：英文名+注释");
    expect(
      await page.evaluate(() => localStorage.getItem("cdb.comment-display")),
      "切换必须写回 localStorage",
    ).toBe("name+comment");

    // THEN 4：刷新重进——comment 数据仍在（持久化）且视图偏好保持
    await page.reload();
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await expect
      .poll(async () => {
        const s = await debugState(page);
        return s.table_meta?.find((t) => t.name === "cms_users")?.comment;
      }, { timeout: 10_000 })
      .toBe("用户表");
    await expect(page.locator('[data-testid="canvas-comment-display"]')).toHaveText(
      "注释：英文名+注释",
    );
  });

  test("ST-CR-COLOR-01: Inspector 改表颜色 → 画布即时更新 → 刷新保留 → JSON 导出导入保留", async ({
    browser,
    request,
  }) => {
    const owner = await registerAndLogin(request, "color01");
    const { roomId } = await createRoomWithDiagram(request, owner, "color01-room");
    const { page } = await newRoomPage(browser, owner, roomId);
    await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
    await importSql(page, COMMENT_DDL, "cms_users");

    // 选中表（点表头区域）→ Inspector 表 section
    const pos = (await debugState(page)).table_pos?.find((t) => t.name === "cms_users");
    if (!pos) throw new Error("table_pos 缺 cms_users");
    const rect = await page.locator('[data-testid="editor-canvas"]').boundingBox();
    if (!rect) throw new Error("canvas 无布局盒");
    await page.mouse.click(rect.x + pos.x + 60, rect.y + pos.y + 20);
    const colorSelect = page.locator('[data-testid="inspector-table-color"]');
    await expect(colorSelect).toBeVisible({ timeout: 5_000 });

    // WHEN：改表颜色（青绿色）→ 画布即时更新（表头渐变 + 边框）
    const before = await stableShot(page);
    await colorSelect.selectOption("rgba(79,209,197,.18)");
    await expect
      .poll(async () => {
        const s = await debugState(page);
        return s.table_meta?.find((t) => t.name === "cms_users")?.color;
      }, { timeout: 5_000 })
      .toBe("rgba(79,209,197,.18)");
    const after = await stableShot(page);
    expect(after.equals(before), "R-COLOR-01：改色后表头/边框必须即时变化").toBe(false);

    // THEN 1：等保存落账后刷新重进——颜色保留
    await expect.poll(async () => (await debugState(page)).dirty, { timeout: 8_000 }).toBe(false);
    await page.reload();
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await expect
      .poll(async () => {
        const s = await debugState(page);
        return s.table_meta?.find((t) => t.name === "cms_users")?.color;
      }, { timeout: 10_000 })
      .toBe("rgba(79,209,197,.18)");

    // THEN 2：导出 JSON 含 color 与 comment
    await page.click('[data-testid="btn-more-menu"]');
    await page.click('[data-testid="btn-export"]');
    const exportDrawer = page.locator('[data-testid="export-drawer"]');
    await expect(exportDrawer).toBeVisible({ timeout: 3_000 });
    await exportDrawer.locator('[data-testid="io-format-tabs"] button', { hasText: "JSON" }).click();
    const preview = await page.locator('[data-testid="export-preview"]').textContent();
    const doc = JSON.parse(preview ?? "{}") as {
      tables?: Array<{ name: string; color?: string; comment?: string }>;
    };
    const exported = doc.tables?.find((t) => t.name === "cms_users");
    expect(exported?.color, "导出 JSON 必须保留表颜色").toBe("rgba(79,209,197,.18)");
    expect(exported?.comment, "导出 JSON 必须保留表注释").toBe("用户表");
    // 关闭导出抽屉（头部 × 按钮，无 testid——取抽屉内首个 icon 按钮）
    await exportDrawer.locator("button.cdb-btn--icon").first().click();
    await expect(exportDrawer).toBeHidden({ timeout: 3_000 });

    // THEN 3：再导入该 JSON（新房间）→ 颜色/注释回填
    const { roomId: room2 } = await createRoomWithDiagram(request, owner, "color01-room-2");
    await page.goto("/");
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${room2}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await importSql(page, preview ?? "", "cms_users", "JSON");
    await expect
      .poll(async () => {
        const s = await debugState(page);
        const t = s.table_meta?.find((t) => t.name === "cms_users");
        return t ? `${t.color}|${t.comment}` : "";
      }, { timeout: 8_000 })
      .toBe("rgba(79,209,197,.18)|用户表");
  });

  test("ST-PB-06: Inspector 修改关系颜色 → 该线变色 → 保存刷新后保留", async ({
    browser,
    request,
  }) => {
    const owner = await registerAndLogin(request, "pb06");
    const { roomId } = await createRoomWithDiagram(request, owner, "pb06-room");
    const { page } = await newRoomPage(browser, owner, roomId);
    const canvas = page.locator('[data-testid="editor-canvas"]');
    await expect(canvas).toBeVisible();
    await importSql(page, FK_DDL, "books");
    await expect
      .poll(async () => (await debugState(page)).ref_meta?.length ?? 0, { timeout: 8_000 })
      .toBe(1);

    // 选中关系线：解析 data-follow-path 的贝塞尔（坐标可空格或逗号分隔），点击 t=0.5 中点
    const d = await canvas.getAttribute("data-follow-path");
    const m = /^M(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+C(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)/.exec(d ?? "");
    if (!m) throw new Error(`data-follow-path 为空或格式异常: ${d}`);
    const [x1, y1, cx1, cy1, cx2, cy2, x2, y2] = m.slice(1).map(Number);
    const midX = (x1 + 3 * cx1 + 3 * cx2 + x2) / 8;
    const midY = (y1 + 3 * cy1 + 3 * cy2 + y2) / 8;
    const rect = await canvas.boundingBox();
    if (!rect) throw new Error("canvas 无布局盒");
    await page.mouse.click(rect.x + midX, rect.y + midY);
    const relColor = page.locator('[data-testid="inspector-relation-color"]');
    await expect(relColor).toBeVisible({ timeout: 5_000 });

    // WHEN：改关系颜色（琥珀色）→ 仅该线变色（画布像素变化 + ref_meta 落账）
    const before = await stableShot(page);
    await relColor.selectOption("#f2b84b");
    await expect
      .poll(async () => (await debugState(page)).ref_meta?.[0]?.color, { timeout: 5_000 })
      .toBe("#f2b84b");
    const after = await stableShot(page);
    expect(after.equals(before), "R-COLOR-02：改色后关系线必须变色").toBe(false);

    // THEN：等保存落账后刷新重进——关系颜色保留
    await expect.poll(async () => (await debugState(page)).dirty, { timeout: 8_000 }).toBe(false);
    await page.reload();
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await expect
      .poll(async () => (await debugState(page)).ref_meta?.[0]?.color, { timeout: 10_000 })
      .toBe("#f2b84b");
  });
});
