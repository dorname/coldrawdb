import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const playwrightBrowsers = applyPlaywrightBrowserEnv();
const { chromium } = await import("playwright");

const startedAt = Date.now();
const currentDir = dirname(fileURLToPath(import.meta.url));
const prototypePath = resolve(
  currentDir,
  "../../logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html",
);

function revisionFrom(text) {
  const match = text.match(/rev\s+(\d+)/);
  assert.ok(match, `无法从保存状态中解析 revision：${text}`);
  return Number(match[1]);
}

async function installRenderProbe(page) {
  await page.evaluate(() => {
    const app = document.querySelector("#app");
    if (!app) throw new Error("缺少 #app 根节点");
    window.__renderProbe?.observer.disconnect();
    const probe = { records: 0, batches: 0, canvasAfterRender: null, observer: null };
    probe.observer = new MutationObserver(records => {
      probe.records += records.length;
      probe.batches += 1;
    });
    probe.observer.observe(app, { childList: true });
    window.__renderProbe = probe;
  });
}

async function resetRenderProbe(page) {
  await page.evaluate(() => {
    window.__renderProbe.records = 0;
    window.__renderProbe.batches = 0;
    window.__renderProbe.canvasAfterRender = null;
  });
}

async function captureCanvasAfterRender(page) {
  await page.waitForTimeout(50);
  await page.evaluate(() => {
    window.__renderProbe.canvasAfterRender = document.querySelector("#canvas");
  });
}

async function waitForAutoSave(page, previousRevision) {
  await page.waitForFunction(
    revision => {
      const chip = document.querySelector('[data-testid="save-state"]');
      const match = chip?.textContent?.match(/rev\s+(\d+)/);
      return chip?.dataset.state === "saved" && match && Number(match[1]) > revision;
    },
    previousRevision,
    { timeout: 3_000 },
  );
}

async function probeResult(page) {
  return page.evaluate(() => ({
    records: window.__renderProbe.records,
    batches: window.__renderProbe.batches,
    canvasPreserved: window.__renderProbe.canvasAfterRender === document.querySelector("#canvas"),
    stableView: document.querySelector("#app > .view")?.classList.contains("view--stable"),
    saveState: document.querySelector('[data-testid="save-state"]')?.dataset.state,
    revisionText: document.querySelector('[data-testid="revision-display"]')?.textContent ?? "",
  }));
}

async function assertSingleRender(page, previousRevision, operation) {
  await captureCanvasAfterRender(page);
  await waitForAutoSave(page, previousRevision);
  const result = await probeResult(page);
  assert.equal(result.records, 1, `${operation}应仅重建一次 #app，实际 ${result.records} 次`);
  assert.equal(result.batches, 1, `${operation}应仅产生一个主渲染批次，实际 ${result.batches} 个`);
  assert.equal(result.canvasPreserved, true, `${operation}的保存阶段替换了 Canvas DOM`);
  assert.equal(result.stableView, true, `${operation}后同一视图仍会重复播放入场动画`);
  assert.equal(result.saveState, "saved", `${operation}完成后未恢复已保存状态`);
  assert.ok(revisionFrom(result.revisionText) > previousRevision, `${operation}未递增 revision`);
  return result;
}

let browser;
try {
  browser = await chromium.launch({
    headless: true,
    executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
  });
  const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
  await page.goto(pathToFileURL(prototypePath).href, { waitUntil: "domcontentloaded" });

  await page.locator('[data-testid="login-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]').waitFor();
  await page.locator('[data-action="enter-room"][data-room="r-001"]').click();
  await page.locator('[data-testid="room-editor-page"]').waitFor();
  await installRenderProbe(page);

  const addRevision = revisionFrom(await page.locator('[data-testid="revision-display"]').textContent());
  await resetRenderProbe(page);
  await page.locator('[data-testid="tool-add-table"]').click();
  const addTable = await assertSingleRender(page, addRevision, "新增表");

  await page.locator('[data-testid="tool-relationship"]').click();
  await page.locator('[data-action="field-click"][data-table="users"][data-field="users-email"]').click();
  await page.waitForTimeout(0);
  const relationRevision = revisionFrom(await page.locator('[data-testid="revision-display"]').textContent());
  await resetRenderProbe(page);
  await page.locator('[data-action="field-click"][data-table="posts"][data-field="posts-title"]').click();
  const completeRelation = await assertSingleRender(page, relationRevision, "创建关系");

  await page.locator('[data-testid="btn-more-menu"]').click();
  await page.locator('[data-testid="btn-import"]').click();
  await page.locator('[data-testid="import-textarea"]').fill(
    "CREATE TABLE products (id UUID PRIMARY KEY);\nCREATE TABLE categories (id UUID PRIMARY KEY);",
  );
  const importRevision = revisionFrom(await page.locator('[data-testid="revision-display"]').textContent());
  const tableCountBeforeImport = (await page.locator(".db-table").all()).length;
  await resetRenderProbe(page);
  await page.locator('[data-testid="import-submit"]').click();
  const batchImport = await assertSingleRender(page, importRevision, "批量导入");
  assert.equal(
    (await page.locator(".db-table").all()).length,
    tableCountBeforeImport + 2,
    "批量导入未在一次提交中创建全部数据表",
  );

  const diagnostics = await page.evaluate(() => window.__cdbPrototype.diagnose());
  assert.equal(diagnostics.pass, true, "原型内置诊断未全部通过");
  assert.equal(diagnostics.checks.length, 7, "原型内置诊断项数量异常");

  reportOpenLogos({ id: "ST-PU-19", status: "pass", durationMs: Date.now() - startedAt });
  process.stdout.write(`${JSON.stringify({ addTable, completeRelation, batchImport, diagnostics: "7/7 PASS" }, null, 2)}\n`);
} catch (error) {
  reportOpenLogos({
    id: "ST-PU-19",
    status: "fail",
    durationMs: Date.now() - startedAt,
    error: error instanceof Error ? error.stack ?? error.message : String(error),
  });
  throw error;
} finally {
  await browser?.close();
}
