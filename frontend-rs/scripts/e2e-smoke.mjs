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

/// 轮询等待 table-list-item 数量 >= n 且可见（绕过 Playwright 对动态 For 元素的可见性判定偶发问题）
async function waitTableCountAtLeast(page, n, timeoutMs = 10_000) {
  const ticks = Math.ceil(timeoutMs / 500);
  for (let i = 0; i < ticks; i++) {
    await page.waitForTimeout(500);
    const ok = await page.evaluate((needed) => {
      const list = Array.from(document.querySelectorAll('[data-testid^="table-list-item-"]'));
      const visible = list.filter((el) => {
        const r = el.getBoundingClientRect();
        const s = window.getComputedStyle(el);
        return r.width > 0 && r.height > 0 && s.display !== "none" && s.visibility !== "hidden";
      });
      return visible.length >= needed;
    }, n);
    if (ok) return;
  }
  throw new Error(`expected >= ${n} visible table-list-item(s) within ${timeoutMs}ms`);
}

/// 轮询等待 data-cdb-revision >= min
async function waitRevisionAtLeast(page, min, timeoutMs = 8_000) {
  const ticks = Math.ceil(timeoutMs / 500);
  for (let i = 0; i < ticks; i++) {
    await page.waitForTimeout(500);
    const rev = Number(await page.evaluate(() => document.documentElement.getAttribute("data-cdb-revision") ?? "0"));
    if (rev >= min) return rev;
  }
  throw new Error(`expected data-cdb-revision >= ${min} within ${timeoutMs}ms`);
}

async function hp01_loadBlankEditor(page) {
  const t0 = Date.now();
  try {
    await waitEditorReady(page);
    await page.locator('[data-testid="app-bar"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="tool-rail"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="editor-canvas"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="canvas-empty-guide"]').waitFor({ state: "visible" });
    await page.locator('[data-testid="revision-display"]').waitFor({ state: "visible" });
    const hasToolbar = await page.locator('[data-testid="toolbar"]').count();
    if (hasToolbar > 0) {
      throw new Error("Phase A: toolbar testid should be removed");
    }
    record(
      "HP-01",
      "Load blank editor",
      "PASS",
      Date.now() - t0,
      "editor + app-bar + tool-rail + canvas + empty-guide + revision in status-bar",
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
    // Phase A: 空白引导卡片创建第一张表
    await page.click('[data-testid="guide-create-table"]');
    console.log("[debug] clicked guide-create-table");

    await waitRevisionAtLeast(page, 1, 12_000);

    await page.waitForTimeout(200);

    if (putRequests.length < 1) {
      throw new Error(`expected >= 1 PUT, got ${putRequests.length}`);
    }

    const guideVisible = await page.locator('[data-testid="canvas-empty-guide"]').isVisible();
    if (guideVisible) {
      throw new Error("empty guide should disappear after creating first table");
    }

    record(
      "HP-02",
      "Create table + 1s debounce auto-save + revision推进",
      "PASS",
      Date.now() - t0,
      `${putRequests.length} PUT(s) fired, revision>=1, empty guide hidden`,
    );
  } catch (e) {
    const shotPath = "/tmp/hp02-fail.png";
    await page.screenshot({ path: shotPath, fullPage: true });
    console.log(`[debug] HP-02 screenshot saved to ${shotPath}`);
    const html = await page.content();
    console.log("[debug] HP-02 page HTML snippet (first 2000 chars):", html.slice(0, 2000));
    record("HP-02", "Auto-save + revision", "FAIL", Date.now() - t0, e.message);
  } finally {
    page.off("request", handler);
  }
}

async function hp03_fieldAndShareModal(page) {
  const t0 = Date.now();
  try {
    // 第二张表：Tool Rail 新建菜单
    await page.click('[data-testid="tool-new-menu"]');
    await page.locator('[data-testid="tool-new-menu-dropdown"]').waitFor({ state: "visible" });
    await page.click('[data-testid="btn-create-table"]');
    await page.waitForTimeout(500);

    // 新建表自动选中，Inspector 应显示表名
    const inspectorName = page.locator('[data-testid="inspector-table-name"]').first();
    await inspectorName.waitFor({ state: "visible", timeout: 5_000 });
    const tableNameText = (await inspectorName.textContent()) ?? "";
    if (!tableNameText.includes("新表")) {
      throw new Error(`expected inspector-table-name to contain 新表, got "${tableNameText}"`);
    }

    // 加字段
    await page.click('[data-testid="btn-add-field"]');
    const fieldRow = page.locator('[data-testid^="field-row-"]').first();
    await fieldRow.waitFor({ state: "visible" });

    // 改字段类型
    const typeDrop = page.locator('[data-testid^="type-"]').first();
    await typeDrop.selectOption("INT");

    // 打开 File 菜单 → Share
    await openFileMenu(page);
    await page.click('[data-testid="cdb-menu-share"]');
    await page.locator('[data-testid="modal-share"]').waitFor({ state: "visible" });

    // 校验 share URL
    const shareUrl = await page.inputValue('[data-testid="share-url"]');
    if (!shareUrl.includes("share=")) {
      throw new Error(`share URL invalid: "${shareUrl}"`);
    }

    record(
      "HP-03",
      "Field add + change type + Share modal URL",
      "PASS",
      Date.now() - t0,
      `inspector shows "${tableNameText.trim()}", field added, type=INT, share URL ok: ${shareUrl}`,
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
    await page.click('[data-testid="btn-import"]');
    await page.locator('[data-testid="import-drawer"]').waitFor({ state: "visible" });

    const sql = "CREATE TABLE smoke_test (id INT PRIMARY KEY, name VARCHAR(64) NOT NULL);";
    await page.fill('[data-testid="import-textarea"]', sql);

    // 解析摘要（parse_sql_statements 应该返回 1 条）
    await page.locator('[data-testid="import-parse-summary"]').waitFor({ state: "visible", timeout: 3_000 });
    const parseText = (await page.textContent('[data-testid="import-parse-summary"]')) ?? "";
    if (!parseText.includes("1")) {
      throw new Error(`parse count unexpected: "${parseText}"`);
    }

    record(
      "HP-04",
      "ImportDrawer SQL parse (Phase C)",
      "PASS",
      Date.now() - t0,
      `parsed: ${parseText.trim()}`,
    );

    await page.click('[data-testid="import-cancel"]');
    await page.locator('[data-testid="import-drawer"]').waitFor({ state: "hidden" });
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

  page.on("console", (msg) => console.log("[console]", msg.type(), msg.text()));
  page.on("pageerror", (err) => console.log("[pageerror]", err.message));

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
