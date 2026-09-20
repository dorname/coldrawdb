/**
 * ST-PB-05 / ST-RP-04（fix-remote-github-issues-7-18，issue #9/#11）
 *
 * ST-PB-05：e2e 目标表左移后连线换侧（core-PB §ST-PB-05）
 *  GIVEN：画布含 A→B 一条关系（SQL FK 导入落账），B 初始在 A 右侧（连线右出左进）
 *  WHEN：拖动目标表到源表左侧并松手落账
 *  THEN：连线路径 d 更新为左出右进（起点为源表左缘锚点）；刷新后关系仍在
 *  断言通道：canvas[data-follow-path]（ST-CR-02 调试属性，每帧写首条关系 calc_path 的 d）
 *
 * ST-RP-04：e2e 拖分隔条全程画布清晰（core-RP §ST-RP-04）
 *  WHEN：拖动 Inspector 分隔条往返 ≥10 次 → 松手
 *  THEN：每步 canvas.width == round(CSS宽 × dpr)（无位图拉伸帧）；全程未点击画布
 *
 * reporter：OpenLogos reporter 按 PREFIX_MAP 提取 ST-PB-05 / ST-RP-04 追加 jsonl。
 */

import { test, expect, type Page } from "@playwright/test";
import { registerAndLogin, createRoomWithDiagram, newRoomPage, debugState } from "../helpers/collab";

/** 导入按钮位于 AppBar 溢出菜单内 */
async function openAppBarImport(page: Page) {
  await page.click('[data-testid="btn-more-menu"]');
  const importBtn = page.locator('[data-testid="btn-import"]');
  await expect(importBtn).toBeVisible();
  await importBtn.click();
}

/** 解析 data-follow-path 的 d 属性起点 x（"M{x} {y} C..."） */
async function followPathStartX(page: Page): Promise<number> {
  const d = await page.locator('[data-testid="editor-canvas"]').getAttribute("data-follow-path");
  const m = /^M(-?[\d.]+) (-?[\d.]+)/.exec(d ?? "");
  if (!m) throw new Error(`data-follow-path 为空或格式异常: ${d}`);
  return Number(m[1]);
}

const FK_DDL = `CREATE TABLE books (
  id INT PRIMARY KEY,
  author_id INT,
  FOREIGN KEY (author_id) REFERENCES authors(id)
);
CREATE TABLE authors (
  id INT PRIMARY KEY
);`;

test.describe("fix-remote-github-issues-7-18 画布选侧与 ResizeObserver", () => {
  test("ST-PB-05: 目标表左移 → 连线换侧（左出右进）→ 刷新后关系仍在", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "pb05");
    const { roomId } = await createRoomWithDiagram(request, owner, "pb05-room");
    const { page } = await newRoomPage(browser, owner, roomId);
    const canvas = page.locator('[data-testid="editor-canvas"]');
    await expect(canvas).toBeVisible();

    // Step 1: SQL FK 导入 → books→authors 一条关系
    await openAppBarImport(page);
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible({ timeout: 3_000 });
    await page.fill('[data-testid="import-textarea"]', FK_DDL);
    await expect(page.locator('[data-testid="import-parse-summary"]')).toHaveText(/[1-9]\d*\s*条语句/);
    await page.click('[data-testid="import-submit"]');
    // 等导入落账：follow-path 出现（首条关系）
    await expect
      .poll(async () => (await page.locator('[data-testid="editor-canvas"]').getAttribute("data-follow-path"))?.length ?? 0, { timeout: 8_000 })
      .toBeGreaterThan(0);

    const rect = await canvas.boundingBox();
    if (!rect) throw new Error("canvas 无布局盒");

    // 表世界坐标经 debug hook 读取（OT 物化/防重叠错位后坐标不可预设）
    const tablePos = async () => {
      const s = await debugState(page);
      const books = s.table_pos?.find((t) => t.name === "books");
      const authors = s.table_pos?.find((t) => t.name === "authors");
      if (!books || !authors) throw new Error(`table_pos 缺表: ${JSON.stringify(s.table_pos)}`);
      return { books, authors };
    };

    // 拖表辅助：从表头 (x+10,y+10) 世界坐标拖到目标世界 x（y 不变），松手后等 OT ack 收敛
    const dragTableToX = async (name: "books" | "authors", toWorldX: number) => {
      const pos = (await tablePos())[name];
      const sx = rect.x + pos.x + 10;
      const sy = rect.y + pos.y + 10;
      const tx = rect.x + toWorldX + 10;
      await page.mouse.move(sx, sy);
      await page.mouse.down();
      const steps = 10;
      for (let i = 1; i <= steps; i++) {
        await page.mouse.move(sx + ((tx - sx) * i) / steps, sy);
        await page.waitForTimeout(30);
      }
      await page.mouse.up();
      await page.waitForTimeout(1_200); // 等 OT ack（ack 前坐标可能被服务器视图回写）
    };

    // Step 2（GIVEN 摆位）：保证 authors 在 books 右侧 → 右出左进
    // d 起点 x1 = books.x + 230（books 右缘锚点）
    let { books } = await tablePos();
    await dragTableToX("authors", books.x + 400);
    await expect
      .poll(async () => followPathStartX(page), { timeout: 8_000 })
      .toBe(books.x + 230);

    // Step 3（WHEN）：拖 authors 到 books 左侧 → 左出右进：x1 = books.x（左缘锚点）
    ({ books } = await tablePos());
    await dragTableToX("authors", books.x - 400);
    await expect
      .poll(async () => followPathStartX(page), { timeout: 8_000 })
      .toBe(books.x);

    // Step 4: 刷新后关系仍在（持久化）且仍为新选侧口径。
    // reload 落地房间列表（编辑器为 SPA 内导航态）——重新点卡片进编辑器
    await page.reload();
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await expect(canvas).toBeVisible({ timeout: 10_000 });
    const { books: booksAfter } = await tablePos();
    await expect
      .poll(async () => followPathStartX(page), { timeout: 10_000 })
      .toBe(booksAfter.x);
  });

  test("ST-RP-04: 拖 Inspector 分隔条往返 → 全程 canvas.width == CSS宽×dpr", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "rp04");
    const { roomId } = await createRoomWithDiagram(request, owner, "rp04-room");
    const { page } = await newRoomPage(browser, owner, roomId);
    const canvas = page.locator('[data-testid="editor-canvas"]');
    await expect(canvas).toBeVisible();

    // 画布 backing store 与 CSS×dpr 一致的等待断言（RO 回调异步，每步等收敛）
    const expectBackingSynced = async () => {
      await page.waitForFunction(
        () => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="editor-canvas"]');
          if (!c || !c.parentElement) return false;
          const dpr = window.devicePixelRatio || 1;
          return (
            c.width === Math.round(c.parentElement.clientWidth * dpr) &&
            c.height === Math.round(c.parentElement.clientHeight * dpr)
          );
        },
        undefined,
        { timeout: 3_000 },
      );
    };

    // 基线：未拖动前已同步
    await expectBackingSynced();

    // 打开 Inspector（点选一张表前无需画布内容——直接拖分隔条也可；splitter 常驻）
    const splitter = page.locator('[data-testid="splitter-inspector"]');
    await expect(splitter).toBeVisible({ timeout: 5_000 });
    const sb = await splitter.boundingBox();
    if (!sb) throw new Error("splitter 无布局盒");
    const sx = sb.x + sb.width / 2;
    const sy = sb.y + sb.height / 2;

    // 往返拖 10 次（左右各 5 次），每步松手后断言 backing store 已同步
    for (let round = 0; round < 10; round++) {
      const dx = round % 2 === 0 ? -120 : 120; // 交替拉宽/收窄 Inspector → 画布宽度随之变化
      await page.mouse.move(sx, sy);
      await page.mouse.down();
      for (let i = 1; i <= 4; i++) {
        await page.mouse.move(sx + (dx * i) / 4, sy);
        await page.waitForTimeout(20);
        // 拖动中亦不得出现拉伸帧（RO 每步同步）
        await expectBackingSynced();
      }
      await page.mouse.up();
      await expectBackingSynced();
    }
  });
});
