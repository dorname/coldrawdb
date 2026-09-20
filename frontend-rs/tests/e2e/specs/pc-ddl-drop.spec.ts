/**
 * ST-PC-09：e2e 拖入 .ddl 文件 → textarea 填充 → 摘要非 0 → 导入到画布成功
 *
 * Spec: logos/resources/test/core-PC-import-export-test-cases.md §ST-PC-09
 * Proposal: logos/changes/fix-remote-github-issues-7-18/（issue #7/#8）
 *
 * GIVEN：编辑器已加载；构造 schema.ddl（含 1 张带中文 COMMENT 的表）
 * WHEN：向 io-dropzone 派发 dragover + drop（DataTransfer 注入文件）→ 点「导入到画布」
 * THEN：textarea 已填充；摘要非 0；导入后画布新增该表；Table.comment 非空
 *
 * reporter：OpenLogos reporter 按 PREFIX_MAP 提取 ST-PC-09 追加 jsonl。
 * 鉴权：沿用 collab.ts 真实注册/建房/注入会话模式（编辑器需登录态）。
 */

import { test, expect, type Page } from "@playwright/test";
import { registerAndLogin, createRoomWithDiagram, newRoomPage } from "../helpers/collab";

/** 导入按钮位于 AppBar 溢出菜单内（沿用 17_import_drawer.spec.ts 口径） */
async function openAppBarImport(page: Page) {
  await page.click('[data-testid="btn-more-menu"]');
  const importBtn = page.locator('[data-testid="btn-import"]');
  await expect(importBtn).toBeVisible();
  await importBtn.click();
}

const DDL = `CREATE TABLE cms_users (
  id INT PRIMARY KEY COMMENT '主键',
  nickname VARCHAR(64) COMMENT '昵称'
) COMMENT='用户表';`;

async function dispatchDropWithFile(page: Page, name: string, text: string) {
  const dropzone = page.locator('[data-testid="io-dropzone"]');
  await expect(dropzone).toBeVisible();
  const dataTransfer = await page.evaluateHandle(
    async ([n, t]) => {
      const dt = new DataTransfer();
      dt.items.add(new File([t as string], n as string, { type: "text/plain" }));
      return dt;
    },
    [name, text],
  );
  await dropzone.dispatchEvent("dragover", { dataTransfer });
  await dropzone.dispatchEvent("drop", { dataTransfer });
}

test.describe("fix-remote-github-issues-7-18 IO 文件拖放", () => {
  test("ST-PC-09: 拖入 .ddl → 摘要非 0 → 导入画布 → Table.comment 非空", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "pc09-a");
    const { roomId } = await createRoomWithDiagram(request, owner, "pc09-room-a");
    const { page } = await newRoomPage(browser, owner, roomId);

    // Step 1: 打开导入抽屉
    await openAppBarImport(page);
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible({ timeout: 3_000 });

    // Step 2: 向 dropzone 派发 dragover + drop（DataTransfer 注入 schema.ddl）
    await dispatchDropWithFile(page, "schema.ddl", DDL);

    // Step 3: textarea 已填充（File.text → content signal）
    await expect(page.locator('[data-testid="import-textarea"]')).toHaveValue(/cms_users/, {
      timeout: 3_000,
    });

    // Step 4: 摘要非 0（.ddl 归 SQL Tab，import_parse_summary → parse_sql_statements）
    await expect(page.locator('[data-testid="import-parse-summary"]')).toHaveText(/[1-9]\d*\s*条语句/, {
      timeout: 2_000,
    });

    // Step 5: 点「导入到画布」
    await page.click('[data-testid="import-submit"]');

    // Step 6: 画布新增该表（切 ListView 全屏视图，树节点按表名定位）
    await page.click('[data-testid="btn-list-view"]');
    const node = page.locator('[data-testid="list-tree-node-cms_users"]');
    await expect(node).toBeVisible({ timeout: 5_000 });

    // Step 7: Table.comment 非空（中文 COMMENT 随导入落账——UT-PC-29 注释行 testid 口径）
    await expect(node.locator('[data-testid^="list-table-comment-"]')).toHaveText(/用户表/);
  });

  test("ST-PC-09 回归: 拖入非白名单 .txt → inline 拒绝，不写内容", async ({ browser, request }) => {
    const owner = await registerAndLogin(request, "pc09-b");
    const { roomId } = await createRoomWithDiagram(request, owner, "pc09-room-b");
    const { page } = await newRoomPage(browser, owner, roomId);

    await openAppBarImport(page);
    await expect(page.locator('[data-testid="import-drawer"]')).toBeVisible({ timeout: 3_000 });

    await dispatchDropWithFile(page, "notes.txt", "not a ddl");

    // inline 提示白名单口径，textarea 保持为空
    await expect(page.locator(".cdb-form-error")).toContainText("仅支持 .sql / .ddl / .dbml / .json");
    await expect(page.locator('[data-testid="import-textarea"]')).toHaveValue("");
  });
});
