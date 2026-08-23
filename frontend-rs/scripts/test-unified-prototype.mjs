import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const playwrightBrowsers = applyPlaywrightBrowserEnv();
const { chromium } = await import("playwright");

const currentDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(currentDir, "../..");
const preferredScreenshotDir = resolve(root, "frontend-rs/test-results/unified-prototype");
let screenshotDir = preferredScreenshotDir;
try {
  mkdirSync(screenshotDir, { recursive: true });
} catch {
  screenshotDir = join(tmpdir(), "coldrawdb-unified-prototype");
  mkdirSync(screenshotDir, { recursive: true });
}
const prototypeUrl = pathToFileURL(resolve(root, "logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html")).href;
const browser = await chromium.launch({
  headless: true,
  executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
});
const results = [];

async function withPage(id, title, body, options = {}) {
  const page = await browser.newPage({ viewport: options.viewport ?? { width: 1440, height: 960 }, reducedMotion: options.reducedMotion });
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  const startedAt = Date.now();
  try {
    await page.goto(prototypeUrl, { waitUntil: "domcontentloaded" });
    await body(page);
    assert.deepEqual(errors, []);
    const durationMs = Date.now() - startedAt;
    results.push({ id, title, status: "pass", durationMs });
    reportOpenLogos({ id, status: "pass", durationMs });
  } catch (error) {
    const durationMs = Date.now() - startedAt;
    const message = error instanceof Error ? error.stack ?? error.message : String(error);
    await page.screenshot({ path: resolve(screenshotDir, `${id}.png`), fullPage: true }).catch(() => {});
    results.push({ id, title, status: "fail", durationMs, error: message });
    reportOpenLogos({ id, status: "fail", durationMs, error: message });
  } finally {
    await page.close();
  }
}

async function login(page) {
  await page.locator('[data-testid="login-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]').waitFor();
}

async function enterEditor(page) {
  await login(page);
  await page.locator('[data-action="enter-room"]').first().click();
  await page.locator('[data-testid="room-editor-page"]').waitFor();
}

const snapshot = page => page.evaluate(() => window.__cdbPrototype.snapshot());

await withPage("ST-PU-01", "单文件断网加载", async page => {
  const audit = await page.evaluate(() => ({
    external: [...document.querySelectorAll('link[rel="stylesheet"], script[src]')].length,
    remote: [...document.querySelectorAll('[src], [href]')].filter(node => /^(https?:)?\/\//.test(node.getAttribute("src") || node.getAttribute("href") || "")).length,
    diagnostic: window.__cdbPrototype.diagnose(),
  }));
  assert.equal(audit.external, 0);
  assert.equal(audit.remote, 0);
  assert.equal(audit.diagnostic.pass, true);
});

await withPage("ST-PU-22", "冷启动仅显示鉴权入口", async page => {
  await assert.doesNotReject(page.locator('[data-testid="login-form"]').waitFor());
  assert.equal(await page.locator('[data-testid="rooms-list-page"]:visible').count(), 0);
  assert.equal(await page.locator('[data-testid="room-editor-page"]:visible').count(), 0);
  await page.locator('[data-action="auth-mode"][data-mode="register"]').click();
  await assert.doesNotReject(page.locator('[data-testid="register-form"]').waitFor());
});

await withPage("ST-PU-02", "登录进入空间", async page => {
  await login(page);
  await assert.doesNotReject(page.locator("#toast-region").getByText("欢迎回来").waitFor());
});

await withPage("ST-PU-03", "注册校验与成功", async page => {
  await page.locator('[data-action="auth-mode"][data-mode="register"]').click();
  await page.locator("#auth-confirm").fill("Different123");
  await page.locator('[data-testid="register-submit"]').click();
  await assert.doesNotReject(page.locator('[data-error="confirm"]').getByText("两次密码不一致").waitFor());
  await page.locator("#auth-confirm").fill("Pass1234");
  await page.locator('[data-testid="register-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]').waitFor();
});

await withPage("ST-PU-04", "创建房间并进入", async page => {
  await login(page);
  await page.locator('[data-testid="btn-create-room"]').click();
  await page.locator("#room-name").fill("数据模型评审");
  await page.locator("#create-room-form button[type=submit]").click();
  await page.locator('[data-testid="room-editor-page"]').waitFor();
  await assert.doesNotReject(page.locator('[data-testid="room-badge"]').getByText("数据模型评审").waitFor());
});

await withPage("ST-PU-05", "编辑表字段与拖拽", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  await page.locator('[data-testid="tool-add-table"]').click();
  await page.locator('[data-action="add-field"]').click();
  await page.locator('[data-change="field-type"]').last().selectOption("TEXT");
  const afterEdit = await snapshot(page);
  assert.equal(afterEdit.tables.length, before.tables.length + 1);
  assert.equal(afterEdit.tables.at(-1).fields.length, 2);
  assert.equal(afterEdit.tables.at(-1).fields.at(-1).type, "TEXT");
  const head = page.locator(`[data-drag-table="${afterEdit.tables.at(-1).id}"]`);
  const box = await head.boundingBox();
  assert.ok(box);
  await page.mouse.move(box.x + 30, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + 120, box.y + 70, { steps: 4 });
  await page.mouse.up();
  const afterDrag = await snapshot(page);
  assert.notEqual(afterDrag.tables.at(-1).x, afterEdit.tables.at(-1).x);
});

await withPage("ST-PU-06", "创建字段关系", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  await page.locator('[data-testid="tool-relationship"]').click();
  await page.locator('[data-action="field-click"][data-field="users-email"]').click();
  await page.locator('[data-action="field-click"][data-field="posts-title"]').click();
  const after = await snapshot(page);
  assert.equal(after.relations.length, before.relations.length + 1);
});

await withPage("ST-PU-20", "拖字段出线创建关系", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  await page.locator('[data-testid="tool-relationship"]').click();
  await page.evaluate(() => {
    const app = document.querySelector("#app");
    window.__appIdentity = app;
    const probe = { records: 0 };
    probe.observer = new MutationObserver(records => { probe.records += records.length; });
    probe.observer.observe(app, { childList: true });
    window.__dragProbe = probe;
  });
  const src = page.locator('[data-action="field-click"][data-field="users-email"]');
  const dst = page.locator('[data-action="field-click"][data-field="posts-title"]');
  const s = await src.boundingBox();
  const d = await dst.boundingBox();
  assert.ok(s && d);
  await page.mouse.move(s.x + s.width / 2, s.y + s.height / 2);
  await page.mouse.down();
  await page.mouse.move(s.x + s.width / 2 + 24, s.y + s.height / 2, { steps: 4 });
  const rubber = page.locator('[data-testid="rel-rubber-band"]');
  await assert.doesNotReject(rubber.waitFor({ state: "visible", timeout: 1_000 }));
  await page.mouse.move(d.x + d.width / 2, d.y + d.height / 2, { steps: 8 });
  const during = await page.evaluate(() => ({
    sameApp: window.__appIdentity === document.querySelector("#app"),
    records: window.__dragProbe.records,
  }));
  assert.equal(during.sameApp, true);
  assert.equal(during.records, 0, `拖线过程不应重建 #app，实际 childList ${during.records}`);
  await page.mouse.up();
  const after = await snapshot(page);
  assert.equal(after.relations.length, before.relations.length + 1);
});

await withPage("ST-PU-21", "拖表时关系线每帧跟随并松手对齐网格", async page => {
  await enterEditor(page);
  const path = page.locator('[data-relation="rel-users-posts"]');
  await path.waitFor();
  const d0 = await path.getAttribute("d");
  assert.ok(d0);
  const head = page.locator('[data-drag-table="users"]');
  const box = await head.boundingBox();
  assert.ok(box);
  await page.mouse.move(box.x + 30, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + 90, box.y + 60, { steps: 6 });
  const d1 = await path.getAttribute("d");
  assert.notEqual(d1, d0, "ST-PU-21: pointerup 前 path d 应已变化");
  await page.mouse.up();
  const after = await snapshot(page);
  const users = after.tables.find(table => table.id === "users");
  assert.ok(users);
  assert.equal(users.x % 12, 0, `松手后 users.x 应为 12 的倍数，实际 ${users.x}`);
  assert.equal(users.y % 12, 0, `松手后 users.y 应为 12 的倍数，实际 ${users.y}`);
});

await withPage("ST-PU-07", "撤销重做与自动保存", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  const revBefore = Number((await page.locator('[data-testid="revision-display"]').textContent()).match(/\d+/)[0]);
  await page.locator('[data-testid="tool-add-table"]').click();
  await page.locator('[data-testid="btn-undo"]').click();
  assert.equal((await snapshot(page)).tables.length, before.tables.length);
  await page.locator('[data-testid="btn-redo"]').click();
  assert.equal((await snapshot(page)).tables.length, before.tables.length + 1);
  await page.waitForFunction(rev => document.querySelector('[data-testid="save-state"]')?.dataset.state === "saved" && Number(document.querySelector('[data-testid="revision-display"]')?.textContent.match(/\d+/)?.[0]) > rev, revBefore);
});

await withPage("ST-PU-08", "导入导出闭环", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  await page.locator('[data-testid="btn-more-menu"]').click();
  await page.locator('[data-testid="btn-import"]').click();
  await page.locator('[data-testid="import-textarea"]').fill("CREATE TABLE audit_items (id UUID PRIMARY KEY);");
  await page.locator('[data-testid="import-submit"]').click();
  assert.equal((await snapshot(page)).tables.length, before.tables.length + 1);
  await page.locator('[data-testid="btn-more-menu"]').click();
  await page.locator('[data-testid="btn-export"]').click();
  assert.match(await page.locator('[data-testid="export-preview"]').textContent(), /CREATE TABLE audit_items/);
});

await withPage("ST-PU-09", "代码视图、命令面板与主题", async page => {
  await enterEditor(page);
  await page.locator('[data-testid="btn-code-view"]').click();
  await page.locator('[data-testid="code-view-modal"]').waitFor();
  await page.getByRole("button", { name: /返回画布/ }).click();
  const before = await page.locator("html").getAttribute("data-mode");
  await page.locator('[data-testid="tool-search"]').click();
  await page.getByRole("button", { name: "切换主题" }).click();
  assert.notEqual(await page.locator("html").getAttribute("data-mode"), before);
  assert.equal(await page.locator("[data-command-overlay]").count(), 0);
});

await withPage("ST-PU-10", "Viewer 邀请预览与接受", async page => {
  await enterEditor(page);
  await page.locator('[data-testid="btn-invite"]').click();
  await page.locator("#invite-role").selectOption("viewer");
  await page.locator('[data-action="preview-invite"]').click();
  await page.locator('[data-testid="invite-accept-page"]').waitFor();
  await page.locator('[data-testid="btn-accept-invite"]').click();
  await page.locator('[data-testid="room-editor-page"]').waitFor();
  assert.equal(await page.locator("#demo-role").inputValue(), "viewer");
});

await withPage("ST-PU-11", "成员角色修改与移除", async page => {
  await enterEditor(page);
  await page.locator('[data-action="open-drawer"][data-drawer="members"]').click();
  await page.locator('[data-change="member-role"][data-id="bob"]').selectOption("viewer");
  assert.equal(await page.locator('[data-change="member-role"][data-id="bob"]').inputValue(), "viewer");
  await page.locator('[data-action="confirm-remove-member"][data-id="bob"]').click();
  await page.locator('[data-action="remove-member"]').click();
  assert.equal(await page.locator('[data-testid="room-members-panel"] .member-row').filter({ hasText: "Bob Li" }).count(), 0);
});

await withPage("ST-PU-12", "远端光标与远端建表", async page => {
  await enterEditor(page);
  const before = await snapshot(page);
  await page.locator('[data-action="remote-cursor"]').click();
  await page.locator('[data-action="remote-table"]').click();
  const after = await snapshot(page);
  assert.equal(after.tables.length, before.tables.length + 1);
  assert.ok(after.tables.some(table => table.name === "orders"));
  assert.ok(after.serverRev > before.serverRev);
});

await withPage("ST-PU-13", "断线排队与恢复", async page => {
  await enterEditor(page);
  await page.locator('[data-action="disconnect"]').click();
  await page.locator('[data-testid="tool-add-table"]').click();
  assert.equal((await snapshot(page)).pendingOps.length, 1);
  await page.locator('[data-action="reconnect"]').first().click();
  await page.waitForFunction(() => window.__cdbPrototype.snapshot().connection === "connected");
  assert.equal((await snapshot(page)).pendingOps.length, 0);
});

await withPage("ST-PU-14", "重连失败后本地编辑", async page => {
  await enterEditor(page);
  await page.locator('[data-action="disconnect"]').click();
  await page.locator('[data-action="reconnect-fail"]').click();
  await page.locator('[data-action="offline-edit"]').click();
  await assert.doesNotReject(page.getByText(/仅本地 · 409 风险/).waitFor());
  const before = (await snapshot(page)).tables.length;
  await page.locator('[data-testid="tool-add-table"]').click();
  assert.equal((await snapshot(page)).tables.length, before + 1);
});

await withPage("ST-PU-15", "Viewer 写入拦截与远端可见", async page => {
  await enterEditor(page);
  await page.locator("#demo-role").selectOption("viewer");
  const before = (await snapshot(page)).tables.length;
  assert.equal(await page.locator('[data-testid="btn-invite"]').isDisabled(), true);
  assert.equal(await page.locator('[data-testid="tool-add-table"]').isDisabled(), true);
  assert.equal((await snapshot(page)).tables.length, before);
  await page.locator('[data-action="remote-table"]').click();
  assert.equal((await snapshot(page)).tables.length, before + 1);
});

await withPage("ST-PU-16", "Token 续期保持编辑状态", async page => {
  await enterEditor(page);
  await page.locator('[data-testid="tool-add-table"]').click();
  const before = await snapshot(page);
  await page.locator('[data-testid="user-menu"]').click();
  await page.locator('[data-testid="session-indicator"]').click();
  await page.waitForFunction(() => document.body.textContent.includes("会话有效 · 60 分钟"));
  assert.deepEqual((await snapshot(page)).tables, before.tables);
});

await withPage("ST-PU-17", "720px 浮层可达性", async page => {
  await enterEditor(page);
  await page.locator('[data-action="open-drawer"][data-drawer="members"]').click();
  await page.locator('[data-testid="room-members-panel"]').waitFor();
  await page.getByRole("button", { name: "邀请新成员" }).click();
  const modal = page.locator('[data-testid="modal-invite"]');
  await modal.waitFor();
  const box = await modal.boundingBox();
  assert.ok(box && box.x >= 0 && box.x + box.width <= 720);
  await page.keyboard.press("Escape");
  assert.equal(await modal.count(), 0);
  const dimensions = await page.evaluate(() => ({ scrollWidth: document.documentElement.scrollWidth, width: innerWidth }));
  assert.ok(dimensions.scrollWidth <= dimensions.width + 1, JSON.stringify(dimensions));
}, { viewport: { width: 720, height: 900 } });

await withPage("ST-PU-18", "reduced-motion 降级", async page => {
  await enterEditor(page);
  await page.locator('[data-action="disconnect"]').click();
  const motion = await page.evaluate(() => {
    const nodes = [document.querySelector(".toast"), document.querySelector('[data-testid="reconnect-banner"]'), document.querySelector(".remote-cursor")].filter(Boolean);
    return nodes.map(node => ({ animation: getComputedStyle(node).animationDuration, transition: getComputedStyle(node).transitionDuration }));
  });
  assert.ok(motion.length >= 2);
  assert.ok(motion.every(item => !item.animation.split(",").some(value => parseFloat(value) > 0.01) && !item.transition.split(",").some(value => parseFloat(value) > 0.01)), JSON.stringify(motion));
}, { reducedMotion: "reduce" });

await browser.close();
process.stdout.write(JSON.stringify({ pass: results.filter(r => r.status === "pass").length, fail: results.filter(r => r.status === "fail").length, results }, null, 2));
if (results.some(result => result.status === "fail")) process.exitCode = 1;
