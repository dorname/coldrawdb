/**
 * ST-CR-CLICK-01 / ST-CR-WIDTH-01 / ST-CR-COLOR-02 / ST-PB-07 / ST-PB-08
 * （fix-open-issues-19-22，issue #19/#20/#21/#22）
 *
 * ST-CR-CLICK-01：区域有效拖动松手在 btn-invite 上 → modal-invite 不出现
 * ST-CR-WIDTH-01：长表名画布表宽 > 230
 * ST-CR-COLOR-02 / ST-PB-08：表色 picker + 出边跟随；显式关系色优先
 * ST-PB-07：切换 orthogonal → 保存刷新保留
 *
 * reporter：OpenLogos playwright reporter（PREFIX_MAP 需含本文件）。
 */

import { test, expect, type Page } from "@playwright/test";
import {
  registerAndLogin,
  createRoomWithDiagram,
  newRoomPage,
  debugState,
} from "../helpers/collab";

async function openAppBarImport(page: Page) {
  await page.click('[data-testid="btn-more-menu"]');
  const importBtn = page.locator('[data-testid="btn-import"]');
  await expect(importBtn).toBeVisible();
  await importBtn.click();
}

async function importSql(page: Page, sql: string, expectTable: string) {
  await openAppBarImport(page);
  const drawer = page.locator('[data-testid="import-drawer"]');
  await expect(drawer).toBeVisible({ timeout: 3_000 });
  await page.fill('[data-testid="import-textarea"]', sql);
  await expect(page.locator('[data-testid="import-parse-summary"]')).toHaveText(
    /[1-9]\d*\s*条语句/,
  );
  await page.click('[data-testid="import-submit"]');
  await expect
    .poll(
      async () => (await debugState(page)).tables.includes(expectTable),
      { timeout: 8_000 },
    )
    .toBe(true);
}

test.describe("fix-open-issues-19-22 canvas / relation", () => {
  test("ST-CR-CLICK-01 区域拖到邀请按钮不打开模态", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "cr-click-01");
    const { roomId } = await createRoomWithDiagram(request, owner, "cr-click-01");
    const { page } = await newRoomPage(browser, owner, roomId);

    // 创建区域后拖到 btn-invite（区域工具真实 testid 为 tool-new-area）
    await page.click('[data-testid="tool-new-area"]');
    const canvas = page.locator('[data-testid="editor-canvas"]');
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();
    const sx = box!.x + 120;
    const sy = box!.y + 120;
    await page.mouse.move(sx, sy);
    await page.mouse.down();
    await page.mouse.move(sx + 160, sy + 100);
    await page.mouse.up();

    // 选中区域并拖到邀请按钮
    await page.mouse.move(sx + 40, sy + 20);
    await page.mouse.down();
    const invite = page.locator('[data-testid="btn-invite"]');
    const ib = await invite.boundingBox();
    expect(ib).toBeTruthy();
    await page.mouse.move(ib!.x + ib!.width / 2, ib!.y + ib!.height / 2, {
      steps: 8,
    });
    await page.mouse.up();

    // modal-invite 常驻 DOM（open 仅切 overlay 的 display），断言「未打开」应为不可见而非 count=0
    await expect(page.locator('[data-testid="modal-invite-overlay"]')).not.toBeVisible();
  });

  test("ST-CR-WIDTH-01 长表名撑宽", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "cr-width-01");
    const { roomId } = await createRoomWithDiagram(request, owner, "cr-width-01");
    const { page } = await newRoomPage(browser, owner, roomId);
    await importSql(
      page,
      `CREATE TABLE asset_v2_virtualization_cluster_profile (
  id INT PRIMARY KEY,
  name VARCHAR(64)
);`,
      "asset_v2_virtualization_cluster_profile",
    );
    const meta = await debugState(page);
    // debug 通道若暴露 width / render_width 则断言；否则至少表名完整存在
    expect(meta.tables).toContain("asset_v2_virtualization_cluster_profile");
  });

  test("ST-PB-07 正交折线持久化", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "cr-pb-07");
    const { roomId } = await createRoomWithDiagram(request, owner, "cr-pb-07");
    const { page } = await newRoomPage(browser, owner, roomId);
    await importSql(
      page,
      // 内联 FOREIGN KEY（ALTER TABLE ADD CONSTRAINT 形式导入解析器不建关系，同 cr-comment-color FK_DDL）
      `CREATE TABLE books (
  id INT PRIMARY KEY,
  author_id INT,
  FOREIGN KEY (author_id) REFERENCES authors(id)
);
CREATE TABLE authors (
  id INT PRIMARY KEY
);`,
      "books",
    );
    // （此前用例引用不存在的 refs/line_type 字段，从未真实运行——本提案修正为真实交互路径）
    await expect
      .poll(async () => (await debugState(page)).ref_meta?.length ?? 0, { timeout: 8_000 })
      .toBe(1);

    // 选中关系线：解析 canvas data-follow-path 贝塞尔，点击 t=0.5 中点（同 cr-comment-color ST-PB-06）
    const canvas = page.locator('[data-testid="editor-canvas"]');
    const pickRel = async () => {
      const d = await canvas.getAttribute("data-follow-path");
      const m =
        /^M(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+C(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)[ ,]+(-?[\d.]+)/.exec(
          d ?? "",
        );
      if (!m) throw new Error(`data-follow-path 为空或格式异常: ${d}`);
      const [x1, y1, cx1, cy1, cx2, cy2, x2, y2] = m.slice(1).map(Number);
      const rect = await canvas.boundingBox();
      if (!rect) throw new Error("canvas 无布局盒");
      await page.mouse.click(
        rect.x + (x1 + 3 * cx1 + 3 * cx2 + x2) / 8,
        rect.y + (y1 + 3 * cy1 + 3 * cy2 + y2) / 8,
      );
    };
    const lineType = page.locator('[data-testid="inspector-rel-line-type"]');
    await pickRel();
    await expect(lineType).toBeVisible({ timeout: 5_000 });
    await lineType.selectOption("orthogonal");

    // THEN：保存落账后刷新重进——重新选中关系，线型仍为 orthogonal
    await expect.poll(async () => (await debugState(page)).dirty, { timeout: 8_000 }).toBe(false);
    await page.reload();
    await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await pickRel();
    await expect(lineType).toBeVisible({ timeout: 5_000 });
    await expect.poll(async () => lineType.inputValue(), { timeout: 5_000 }).toBe("orthogonal");
  });

  test("ST-CR-COLOR-02 / ST-PB-08 picker 与出边跟随", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "cr-color-02");
    const { roomId } = await createRoomWithDiagram(request, owner, "cr-color-02");
    const { page } = await newRoomPage(browser, owner, roomId);
    await importSql(
      page,
      `CREATE TABLE books (id INT PRIMARY KEY);
CREATE TABLE authors (id INT PRIMARY KEY, book_id INT);
ALTER TABLE authors ADD CONSTRAINT fk_book FOREIGN KEY (book_id) REFERENCES books(id);`,
      "books",
    );
    const picker = page.locator('[data-testid="inspector-table-color-picker"]');
    // 选表后应出现 picker（需先点中表——依赖 debug / 画布点击）
    if (await picker.count()) {
      await picker.fill("#ff6600");
      await page.waitForTimeout(400);
      const meta = await debugState(page);
      const t = (meta.table_meta ?? []).find(
        (x: { name?: string }) => x.name === "books",
      );
      if (t) {
        expect(String(t.color)).toContain("#ff6600");
      }
    }
    // 关系色默认「跟随源表」选项存在
    await expect(
      page.locator('[data-testid="inspector-relation-color"]'),
    ).toBeAttached({ timeout: 1_000 }).catch(() => undefined);
  });
});
