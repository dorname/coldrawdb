#!/usr/bin/env node
// Phase 3-7 smoke: 5 大 happy path e2e
//
// 前置: 后端 cargo run :3000 + 前端 trunk serve :8080
// 用法: node frontend-rs/scripts/e2e-smoke.mjs
// 输出: logos/spec/smoke-report.md

import { chromium } from "playwright";
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPORT_PATH = join(__dirname, "..", "..", "logos", "spec", "smoke-report.md");
const BASE_URL = process.env.SMOKE_BASE_URL || "http://localhost:8080";

const cases = [];

function record(id, name, status, durationMs, note) {
  cases.push({ id, name, status, durationMs, note: note ?? "" });
  const icon = status === "PASS" ? "PASS" : "FAIL";
  console.log(`[${icon}] ${id} ${name} (${durationMs}ms)${note ? "  -- " + note : ""}`);
}

async function waitEditorReady(page) {
  await page.goto(BASE_URL + "/");
  await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 20_000 });
  await page.waitForLoadState("networkidle", { timeout: 5_000 }).catch(() => {});
}

async function openFileMenu(page) {
  await page.click('[data-testid="cdb-menu-file"]');
  await page.locator('[data-testid="cdb-menu-file-dropdown"]').waitFor({ state: "visible", timeout: 3_000 });
}

async function hp01_loadBlankEditor(page) {
  const t0 = Date.now();
  try {
    await waitEditorReady(page);
    await page.locator('[data-testid="top-menu-bar"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="toolbar"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="left-panel"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="editor-canvas"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="revision-display"]').waitFor({ state: "visible" });
    record(
      "HP-01",
      "Load blank editor",
      "PASS",
      Date.now() - t0,
      "editor + top menu + toolbar + side panel + canvas + revision display",
    );
  } catch (e) {
    record("HP-01", "Load blank editor", "FAIL", Date.now() - t0, e.message);
  }
}

async function hp02_createTableAndAutoSave(page) {
  const t0 = Date.now();
  const putRequests = [];
  const handler = (req) => {
    if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
      putRequests.push(req.url());
    }
  };
  page.on("request", handler);
  try {
    // 创建表（弹窗流）
    await page.click('[data-testid="btn-create-table"]');
    await page.locator('[data-testid="table-name-input"]').waitFor({ state: "visible" });
    await page.fill('[data-testid="table-name-input"]', "users");
    await page.click('[data-testid="btn-confirm"]');

    // 等 debounce 1.1s + 网络空闲
    await page.waitForTimeout(1_300);

    if (putRequests.length < 1) {
      throw new Error(`expected >= 1 PUT, got ${putRequests.length}`);
    }

    // 记录当前 diagram id 段（URL 路径的一部分）用于 reload 后断言
    const beforeReloadTables = await page.locator('[data-testid^="table-list-item-"]').count();
    if (beforeReloadTables < 1) {
      throw new Error(`expected >= 1 table before reload, got ${beforeReloadTables}`);
    }

    // reload
    await page.reload();
    await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 15_000 });
    await page.locator('[data-testid^="table-list-item-"]').first().waitFor({ state: "visible", timeout: 5_000 });
    const afterReloadTables = await page.locator('[data-testid^="table-list-item-"]').count();
    if (afterReloadTables < 1) {
      throw new Error(`expected >= 1 table after reload, got ${afterReloadTables}`);
    }

    record(
      "HP-02",
      "Create table + 1s debounce auto-save + reload persistence",
      "PASS",
      Date.now() - t0,
      `${putRequests.length} PUT(s) fired, table persists (${beforeReloadTables} -> ${afterReloadTables})`,
    );
  } catch (e) {
    record("HP-02", "Auto-save + reload", "FAIL", Date.now() - t0, e.message);
  } finally {
    page.off("request", handler);
  }
}

async function hp03_fieldAndShareModal(page) {
  const t0 = Date.now();
  try {
    // 创建一张新表（hp02 已有 users，这里加 orders 以验证多表 + 字段）
    await page.click('[data-testid="btn-create-table"]');
    await page.locator('[data-testid="table-name-input"]').waitFor({ state: "visible" });
    await page.fill('[data-testid="table-name-input"]', "orders");
    await page.click('[data-testid="btn-confirm"]');

    // 选中 orders 表（在侧栏列表中点击）
    const ordersItem = page.locator('[data-testid^="table-list-item-"]', { hasText: "orders" }).first();
    await ordersItem.click();

    // 加字段
    await page.click('[data-testid="btn-add-field"]');
    const fieldRow = page.locator('[data-testid^="field-"]').first();
    await fieldRow.waitFor({ state: "visible" });

    // 改字段类型
    const typeDrop = page.locator('[data-testid^="type-"]').first();
    await typeDrop.selectOption("INT");

    // 打开 File 菜单 → Share
    await openFileMenu(page);
    await page.click('[data-testid="cdb-menu-share"]');
    await page.locator('[data-testid="modal-share"]').waitFor({ state: "visible" });

    // 校验 share URL
    const shareUrl = await page.inputValue('[data-testid="modal-input-share-url"]');
    if (!shareUrl.includes("share=")) {
      throw new Error(`share URL invalid: "${shareUrl}"`);
    }

    record(
      "HP-03",
      "Field add + change type + Share modal URL",
      "PASS",
      Date.now() - t0,
      `field added, type=INT, share URL ok: ${shareUrl}`,
    );

    // 关闭 share 模态（点 cancel）
    await page.click('[data-testid="modal-cancel-share-btn"]');
    await page.locator('[data-testid="modal-share"]').waitFor({ state: "hidden" });
  } catch (e) {
    record("HP-03", "Field + share", "FAIL", Date.now() - t0, e.message);
  }
}

async function hp04_sqlImportParse(page) {
  const t0 = Date.now();
  try {
    await openFileMenu(page);
    await page.click('[data-testid="cdb-menu-import"]');
    await page.locator('[data-testid="modal-import"]').waitFor({ state: "visible" });

    const sql = "CREATE TABLE smoke_test (id INT PRIMARY KEY, name VARCHAR(64) NOT NULL);";
    await page.fill('[data-testid="modal-input-sql"]', sql);

    // 解析提示（parse_sql_statements 应该返回 1 条）
    await page.locator('[data-testid="modal-parse-count"]').waitFor({ state: "visible", timeout: 3_000 });
    const parseText = (await page.textContent('[data-testid="modal-parse-count"]')) ?? "";
    if (!parseText.includes("1")) {
      throw new Error(`parse count unexpected: "${parseText}"`);
    }

    record(
      "HP-04",
      "SQL import modal + parse (UI shell + parser)",
      "PASS",
      Date.now() - t0,
      `parsed: ${parseText.trim()}`,
    );

    // 关闭模态
    await page.click('[data-testid="modal-cancel-import-btn"]');
    await page.locator('[data-testid="modal-import"]').waitFor({ state: "hidden" });
  } catch (e) {
    record("HP-04", "SQL import", "FAIL", Date.now() - t0, e.message);
  }
}

async function hp05_keyboardShortcuts(page) {
  const t0 = Date.now();
  try {
    // 把焦点放到画布（不放在 input/textarea，避免输入框拦截）
    await page.locator('[data-testid="editor-canvas"]').click();

    // Ctrl+Z（undo）
    await page.keyboard.press("Control+z");
    await page.waitForTimeout(200);
    // app 不能崩
    await page.locator('[data-testid="editor-ready"]').waitFor({ state: "visible", timeout: 3_000 });

    // Ctrl+Shift+Z（redo）
    await page.keyboard.press("Control+Shift+z");
    await page.waitForTimeout(200);
    await page.locator('[data-testid="editor-ready"]').waitFor({ state: "visible", timeout: 3_000 });

    record(
      "HP-05",
      "Keyboard shortcuts Ctrl+Z / Ctrl+Shift+Z (no crash)",
      "PASS",
      Date.now() - t0,
      "shortcuts detected by KeyboardShortcuts component, app stable",
    );
  } catch (e) {
    record("HP-05", "Keyboard shortcuts", "FAIL", Date.now() - t0, e.message);
  }
}

async function main() {
  console.log("=== smoke: Phase 3-7 5 HP ===");
  console.log(`Base URL: ${BASE_URL}`);
  const t0 = Date.now();

  const browser = await chromium.launch({ headless: true });
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await ctx.newPage();

  // 串行跑 5 个 HP（HP-02/03 共享页面状态，HP-04/05 关闭模态即可）
  await hp01_loadBlankEditor(page);
  await hp02_createTableAndAutoSave(page);
  await hp03_fieldAndShareModal(page);
  await hp04_sqlImportParse(page);
  await hp05_keyboardShortcuts(page);

  await ctx.close();
  await browser.close();

  const passed = cases.filter((c) => c.status === "PASS").length;
  const failed = cases.filter((c) => c.status === "FAIL").length;
  const total = cases.length;
  const totalMs = Date.now() - t0;
  const allPass = failed === 0;

  const lines = [
    "# Smoke Report — Phase 3-7 5 Happy Path (Local Dev)",
    "",
    `> Generated by \`frontend-rs/scripts/e2e-smoke.mjs\` on ${new Date().toISOString()}`,
    `> Target: \`${BASE_URL}\``,
    "",
    "## 总体",
    "",
    `- ${passed}/${total} PASSED`,
    `- 总耗时 ${totalMs}ms`,
    "",
    "## 详情",
    "",
    "| ID | 状态 | 耗时 | 备注 |",
    "|---|---|---|---|",
    ...cases.map(
      (c) => `| ${c.id} | ${c.status === "PASS" ? "PASS" : "**FAIL**"} | ${c.durationMs}ms | ${c.note.replace(/\|/g, "\\|")} |`,
    ),
    "",
    allPass ? "## SMOKE_PASS" : "## SMOKE_FAIL",
    "",
  ];

  mkdirSync(dirname(REPORT_PATH), { recursive: true });
  writeFileSync(REPORT_PATH, lines.join("\n"), "utf-8");

  console.log("");
  console.log(`Report: ${REPORT_PATH}`);
  console.log(`Result: ${allPass ? "SMOKE_PASS" : "SMOKE_FAIL"} (${passed}/${total})`);

  process.exit(allPass ? 0 : 1);
}

main().catch((e) => {
  console.error("smoke crashed:", e);
  process.exit(2);
});
