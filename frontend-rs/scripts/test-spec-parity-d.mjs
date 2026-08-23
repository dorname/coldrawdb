// implement-unified-prototype-spec-parity D 批 — IO / 快捷键 / 主题 / 响应式 / 画布拖拽 e2e
//
// 覆盖用例（落地后须从 SPEC_PARITY_SKIP_IDS 移除）：
//   ST-KB-CMD-01     ⌘K 打开 command-palette；Esc 关闭无残留
//   ST-KB-ESC-01     Esc 按层级关闭最上层；不误关编辑器页
//   ST-KB-T-01       按 T（无输入焦点）新建表；输入焦点不抢键
//   ST-KB-R-01       按 R 进入关系工具
//   ST-KB-VIEWER     Viewer 按 T/R 不创建、不进工具（只读）
//   ST-PC-MENU-01    更多菜单 → 导入/导出 → IO 抽屉（非历史独立 Import 模态）
//   ST-PC-FMT-01     导出抽屉 SQL/DBML/JSON 切换预览随模型更新；可复制/下载
//   ST-PC-INSPECTOR  IO 打开 Inspector 让位；关闭后恢复
//   ST-PU-25         主题切换 data-mode；画布随主题重绘；无半透明残留层
//   ST-PU-26         720px：Inspector/IO 抽屉化、可关闭、无动态背景滚动锁定
//
// 同批落地的既有 harness-skip（验收 §7.5 证据，落地后从 ST_SKIP_IDS 移除）：
//   ST-CR-02         拖表过程连线路径跟手；松手后表坐标为 GRID_SIZE=20 的倍数
//   ST-PB-01         关系工具点击两点 + 确认条创建关系
//   ST-PB-02         关系工具拖线（≥4px + rubber-band）+ 确认条创建关系
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const FRONTEND_URL = process.env.SPEC_PARITY_FRONTEND_URL || "http://127.0.0.1:4175";
const BACKEND_URL = "http://127.0.0.1:3000";
const playwrightBrowsers = applyPlaywrightBrowserEnv();
const { chromium } = await import("playwright");

const corsHeaders = {
  "access-control-allow-origin": FRONTEND_URL,
  "access-control-allow-credentials": "true",
  "access-control-allow-headers": "authorization,content-type",
  "access-control-allow-methods": "GET,POST,PUT,PATCH,DELETE,OPTIONS",
  "content-type": "application/json",
};

function response(route, status, body = {}) {
  return route.fulfill({ status, headers: corsHeaders, body: JSON.stringify(body) });
}

async function waitForServer(url, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {}
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  throw new Error(`前端服务未在 ${timeoutMs}ms 内启动：${url}`);
}

function startFrontend() {
  if (process.env.SPEC_PARITY_FRONTEND_URL) return null;
  const env = { ...process.env };
  delete env.NO_COLOR;
  delete env.FORCE_COLOR;
  return spawn("trunk", ["serve", "--port", "4175"], {
    cwd: new URL("..", import.meta.url),
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

// trunk serve 在重建期间会继续提供旧 dist；先同步 trunk build 保证测试跑到当前源码
async function prebuildFrontend() {
  if (process.env.SPEC_PARITY_FRONTEND_URL) return;
  const env = { ...process.env };
  delete env.NO_COLOR;
  delete env.FORCE_COLOR;
  const build = spawn("trunk", ["build"], {
    cwd: new URL("..", import.meta.url),
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stderr = "";
  build.stderr.on("data", chunk => { stderr += chunk; });
  const code = await new Promise(resolve => build.on("close", resolve));
  if (code !== 0) throw new Error(`trunk build 失败（exit ${code}）：${stderr.slice(-500)}`);
}

const OWNER = { id: "owner-1", email: "owner@example.com", displayName: "林默" };
const MEMBERS = {
  items: [
    { userId: "owner-1", email: "owner@example.com", displayName: "林默", role: "owner", joinedAt: "2026-08-23T00:00:00Z" },
  ],
};

// options.roomsList: Viewer 场景预置房间；options.myRole: 房间角色
async function installApi(page, options = {}) {
  const state = { requests: [], putCalls: 0, lastPutBody: null };
  let diagramRev = 0;

  await page.route(`${BACKEND_URL}/diagrams/queryAll`, route => response(route, 200, {
    code: 0, message: "success", data: [],
  }));

  await page.route(`${BACKEND_URL}/api/v1/**`, async route => {
    const request = route.request();
    const url = new URL(request.url());
    state.requests.push(`${request.method()} ${url.pathname}`);
    if (request.method() === "OPTIONS") return response(route, 204);

    if (url.pathname === "/api/v1/auth/login") {
      return response(route, 200, { accessToken: "access-owner", expiresIn: 900, tokenType: "Bearer" });
    }
    if (url.pathname === "/api/v1/auth/me") return response(route, 200, OWNER);
    if (url.pathname === "/api/v1/auth/refresh") {
      return response(route, 200, { accessToken: "access-refreshed", expiresIn: 900, tokenType: "Bearer" });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "GET") {
      return response(route, 200, { items: options.roomsList ?? [], total: (options.roomsList ?? []).length });
    }
    if (url.pathname === "/api/v1/diagrams" && request.method() === "POST") {
      return response(route, 200, { code: 0, request_id: "create-diagram", data: { id: "diagram-new" } });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "POST") {
      const body = request.postDataJSON();
      return response(route, 201, { id: "room-new", name: body.name, diagramId: body.diagramId, ownerId: "owner-1" });
    }
    if (url.pathname === "/api/v1/rooms/room-new" && request.method() === "GET") {
      return response(route, 200, {
        id: "room-new", name: "架构评审室", diagramId: "diagram-new", ownerId: "owner-1",
        diagramTitle: "架构评审室", myRole: options.myRole ?? "owner", memberCount: 1,
      });
    }
    if (url.pathname === "/api/v1/rooms/room-view" && request.method() === "GET") {
      return response(route, 200, {
        id: "room-view", name: "只读评审室", diagramId: "diagram-new", ownerId: "owner-1",
        diagramTitle: "只读评审室", myRole: "viewer", memberCount: 2,
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "GET") {
      return response(route, 200, {
        code: 0, request_id: "load-diagram",
        data: { id: "diagram-new", name: "架构评审室", database: null, revision: diagramRev, tables: [], references: [], areas: [], notes: [] },
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "PUT") {
      state.putCalls += 1;
      state.lastPutBody = request.postDataJSON();
      diagramRev += 1;
      return response(route, 200, { code: 0, request_id: "save-diagram", data: { revision: diagramRev } });
    }
    if (url.pathname.endsWith("/collab/head")) {
      return response(route, 200, { roomId: "room-new", diagramId: "diagram-new", serverRev: 7 });
    }
    if (url.pathname.endsWith("/members")) return response(route, 200, MEMBERS);
    return response(route, 404, { code: "NOT_FOUND" });
  });
  return state;
}

async function login(page) {
  await page.goto(FRONTEND_URL);
  await page.locator('[data-testid="auth-email"]').fill("owner@example.com");
  await page.locator('[data-testid="auth-password"]').fill("Pass1234!");
  await page.locator('[data-testid="login-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
}

async function createRoomAndEnter(page) {
  await page.locator('[data-testid="btn-create-room"]').click();
  await page.locator('[data-testid="modal-create-room"]').waitFor();
  await page.locator('[data-testid="create-room-name"]').fill("架构评审室");
  await page.locator('[data-testid="create-room-submit"]').click();
  await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
}

// 建两张表并等保存落账；返回 state.lastPutBody 可用的时机点
async function createTwoTables(page) {
  await page.keyboard.press("t");
  await page.locator('[data-testid="inspector-table-name"]').waitFor();
  await page.keyboard.press("t");
  await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
}

// 画布坐标（默认 pan0 zoom1；表 i 落位 x=180+i*55, y=145+i*35；表头 43 / 字段行 35）
const TABLE1_FIELD = { x: 220, y: 205 }; // table_1 首字段行（避开 table_2 x 重叠区）
const TABLE2_FIELD = { x: 295, y: 240 }; // table_2 首字段行
const TABLE2_HEADER = { x: 350, y: 201.5 }; // table_2 表头中心（rev 命中优先 table_2）

async function canvasPoint(page, point) {
  const box = await page.locator('[data-testid="editor-canvas-container"] canvas').boundingBox();
  assert.ok(box, "canvas 必须存在");
  return { x: box.x + point.x, y: box.y + point.y };
}

// 等画布布局稳定（Inspector 开合/保存态切换会触发容器过渡，动画中点击会落空）
async function waitForCanvasStable(page, timeoutMs = 3_000) {
  const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
  const deadline = Date.now() + timeoutMs;
  let prev = await canvas.boundingBox();
  while (Date.now() < deadline) {
    await page.waitForTimeout(120);
    const next = await canvas.boundingBox();
    if (prev && next && prev.x === next.x && prev.y === next.y
        && prev.width === next.width && prev.height === next.height) {
      return;
    }
    prev = next;
  }
  throw new Error("画布布局在 3s 内未稳定");
}

let frontend = null;
let browser;
let failed = false;

async function run(ids, title, body, options = {}) {
  const page = await browser.newPage({ viewport: options.viewport ?? { width: 1440, height: 900 } });
  const startedAt = Date.now();
  try {
    await body(page);
    const durationMs = Date.now() - startedAt;
    for (const id of ids) reportOpenLogos({ id, status: "pass", durationMs });
    process.stdout.write(`PASS ${ids.join(", ")} ${title}\n`);
  } catch (error) {
    failed = true;
    const durationMs = Date.now() - startedAt;
    const message = error instanceof Error ? error.message : String(error);
    for (const id of ids) reportOpenLogos({ id, status: "fail", durationMs, error: message });
    process.stderr.write(`FAIL ${ids.join(", ")} ${title}: ${message}\n`);
  } finally {
    await page.close();
  }
}

try {
  await prebuildFrontend();
  frontend = startFrontend();
  await waitForServer(FRONTEND_URL);
  browser = await chromium.launch({
    headless: true,
    executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
  });

  // ─── ST-KB-CMD-01：⌘K 打开命令面板；Esc 关闭无残留 ──────────────────────
  await run(["ST-KB-CMD-01"], "⌘K 打开命令面板，Esc 关闭无残留", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.keyboard.press("Control+k");
    await page.locator('[data-testid="command-palette"]:visible').waitFor();
    await page.locator('[data-testid="command-palette-input"]:visible').waitFor();

    await page.keyboard.press("Escape");
    await page.waitForTimeout(200);
    assert.equal(await page.locator('[data-testid="command-palette"]').count(), 0, "Esc 后命令面板 DOM 必须无残留");

    // 再次唤起（toggle 在 Esc 关闭后仍可用）
    await page.keyboard.press("Control+k");
    await page.locator('[data-testid="command-palette"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await page.waitForTimeout(200);
    assert.equal(await page.locator('[data-testid="command-palette"]').count(), 0);
  });

  // ─── ST-KB-ESC-01：Esc 按层级关闭最上层；不误关编辑器页 ──────────────────
  await run(["ST-KB-ESC-01"], "Esc 按层级关闭浮层且不误关编辑器页", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();

    // L4 主模态在最上层：Esc 只关模态，不关 Inspector / 编辑器页
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-share"]').click();
    await page.locator('[data-testid="modal-share"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="modal-root"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="inspector"]:visible').waitFor();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    // 无浮层时再 Esc：编辑器页不被误关（Inspector 非 Esc 浮层，保持原状）
    await page.keyboard.press("Escape");
    await page.waitForTimeout(200);
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
    await page.locator('[data-testid="inspector"]:visible').waitFor();

    // L6 IO 抽屉层级：Esc 关闭抽屉（Inspector 按缓存恢复）
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    await page.locator('[data-testid="export-drawer"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    // L8 关系工具模式：Esc 退出工具
    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="rel-tool-hint"]').waitFor({ state: "hidden" });
    assert.equal(
      await page.locator('[data-testid="tool-relationship"].cdb-is-active').count(), 0,
      "Esc 必须退出关系工具",
    );
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
  });

  // ─── ST-KB-T-01：按 T 新建表；输入焦点不抢键 ─────────────────────────────
  await run(["ST-KB-T-01"], "按 T 建表；输入框焦点时不触发", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.keyboard.press("t");
    const nameInput = page.locator('[data-testid="inspector-table-name"]');
    await nameInput.waitFor();
    assert.equal(await nameInput.inputValue(), "table_1", "按 T 必须新建 table_1");
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });

    // 输入焦点：在表名输入框按 t 只输入字符，不再建表
    await nameInput.click();
    await page.keyboard.press("t");
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 1, "输入焦点下按 t 不得新建表");
  });

  // ─── ST-KB-R-01：按 R 进入关系工具 ───────────────────────────────────────
  await run(["ST-KB-R-01"], "按 R 进入关系工具", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.keyboard.press("r");
    await page.locator('[data-testid="tool-relationship"].cdb-is-active').waitFor();
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
  });

  // ─── ST-KB-VIEWER：Viewer 按 T/R 不创建、不进工具 ────────────────────────
  await run(["ST-KB-VIEWER"], "Viewer 只读下 T/R 快捷键不生效", async page => {
    const state = await installApi(page, {
      roomsList: [{
        id: "room-view", name: "只读评审室", diagramId: "diagram-new", diagramTitle: "只读评审室",
        myRole: "viewer", memberCount: 2, updatedAt: "2026-08-23T00:02:00Z",
      }],
    });
    await login(page);
    await page.locator('[data-testid="room-card-room-view"]:visible').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    await page.keyboard.press("t");
    await page.keyboard.press("r");
    await page.waitForTimeout(1_600);
    assert.equal(state.putCalls, 0, "Viewer 按 T 不得触发保存");
    assert.equal(await page.locator('[data-testid="inspector-table-name"]').count(), 0, "Viewer 按 T 不得建表");
    assert.equal(
      await page.locator('[data-testid="tool-relationship"].cdb-is-active').count(), 0,
      "Viewer 按 R 不得进入关系工具",
    );
    assert.equal(await page.locator('[data-testid="rel-tool-hint"]').count(), 0);
  });

  // ─── ST-PC-MENU-01：更多菜单 → 导入/导出 → IO 抽屉 ───────────────────────
  await run(["ST-PC-MENU-01"], "更多菜单进出导入/导出 IO 抽屉", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="app-bar-overflow-menu"]:visible').waitFor();
    await page.locator('[data-testid="btn-import"]').click();
    await page.locator('[data-testid="io-drawer"]:visible').waitFor();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    // 主路径是 IO 抽屉而非历史独立 Import 模态
    await page.locator('[data-testid="modal-root"]').waitFor({ state: "hidden" });
    assert.equal(await page.locator('[data-testid="modal-import"]').count(), 0);

    await page.locator('[data-testid="import-cancel"]').click();
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    await page.locator('[data-testid="export-drawer"]:visible').waitFor();
  });

  // ─── ST-PC-FMT-01：格式切换预览随模型更新；可复制/下载 ────────────────────
  await run(["ST-PC-FMT-01"], "导出抽屉 SQL/DBML/JSON 预览切换与复制下载", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    const preview = page.locator('[data-testid="export-preview"]');
    await preview.waitFor();

    // 默认 SQL：预览含当前模型
    assert.match((await preview.textContent()) ?? "", /CREATE TABLE table_1 \(/);
    // DBML
    await page.locator('[data-testid="io-format-tabs"] button', { hasText: "DBML" }).click();
    assert.match((await preview.textContent()) ?? "", /Table table_1 \{/);
    // JSON
    await page.locator('[data-testid="io-format-tabs"] button', { hasText: "JSON" }).click();
    const jsonText = (await preview.textContent()) ?? "";
    assert.match(jsonText, /"tables": \[/);
    assert.match(jsonText, /"name": "table_1"/);
    // 切回 SQL（预览随格式切换更新）
    await page.locator('[data-testid="io-format-tabs"] button', { hasText: "SQL" }).click();
    assert.match((await preview.textContent()) ?? "", /CREATE TABLE table_1 \(/);

    // 复制：成功后按钮反馈「已复制」
    await page.locator('[data-testid="export-copy"]').click();
    await page.locator('[data-testid="export-copy"]').getByText("已复制").waitFor({ timeout: 3_000 });
    // 下载：触发浏览器下载事件
    const downloadPromise = page.waitForEvent("download", { timeout: 5_000 });
    await page.locator('[data-testid="export-download"]').click();
    const download = await downloadPromise;
    assert.match(download.suggestedFilename(), /\.sql$/, "SQL 格式下载文件名必须以 .sql 结尾");
  });

  // ─── ST-PC-INSPECTOR：IO 打开 Inspector 让位；关闭后恢复 ──────────────────
  await run(["ST-PC-INSPECTOR"], "IO 抽屉与 Inspector 互斥让位与恢复", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector"]:visible').waitFor();

    // 打开导出抽屉 → Inspector 让位
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    await page.locator('[data-testid="export-drawer"]:visible').waitFor();
    await page.locator('[data-testid="inspector"]').waitFor({ state: "hidden" });

    // Esc 关闭抽屉 → Inspector 恢复
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="inspector"]:visible').waitFor();

    // 再开导入抽屉 → 同样让位；取消按钮关闭 → 恢复
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-import"]').click();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    await page.locator('[data-testid="inspector"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="import-cancel"]').click();
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="inspector"]:visible').waitFor();
  });

  // ─── ST-PU-25：主题切换 data-mode；画布重绘；无残留层 ─────────────────────
  await run(["ST-PU-25"], "主题切换 data-mode 且画布随主题重绘", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });

    const html = page.locator("html");
    assert.equal(await html.getAttribute("data-mode"), "dark", "默认暗色（index.html 基线）");
    const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
    const darkShot = await canvas.screenshot();

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-theme-toggle"]').click();
    await page.waitForTimeout(200); // 等 theme effect 重绘一帧
    assert.equal(await html.getAttribute("data-mode"), "light", "切换后 data-mode 必须为 light");
    const lightShot = await canvas.screenshot();
    assert.ok(!darkShot.equals(lightShot), "画布必须随主题重绘（亮暗调色板不同）");

    // 壳层仍可读可操作；无半透明残留层
    await page.locator('[data-testid="app-bar"]:visible').waitFor();
    await page.locator('[data-testid="tool-rail"]:visible').waitFor();
    assert.equal(await page.locator(".cdb-command-palette-overlay").count(), 0);
    assert.equal(await page.locator('[data-testid="modal-conflict"]').count(), 0);
    await page.locator('[data-testid="modal-root"]').waitFor({ state: "hidden" });

    // 切回暗色
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-theme-toggle"]').click();
    await page.waitForTimeout(200);
    assert.equal(await html.getAttribute("data-mode"), "dark");
  });

  // ─── ST-PU-26：720px 下 Inspector/IO 抽屉化、可关闭、无背景滚动锁定 ────────
  await run(["ST-PU-26"], "720px 视口 Inspector/IO 抽屉化与可达性", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    const scrollWidth = () => page.evaluate(() => document.documentElement.scrollWidth);
    assert.ok((await scrollWidth()) <= 720, "720px 视口不得横向溢出");

    // Inspector 抽屉化：全宽浮层、可关闭
    await page.keyboard.press("t");
    const inspector = page.locator('[data-testid="inspector"]:visible');
    await inspector.waitFor();
    const inspectorBox = await inspector.boundingBox();
    assert.ok(inspectorBox && inspectorBox.width <= 720, "Inspector 抽屉宽度不得超出视口");
    assert.ok((await scrollWidth()) <= 720, "Inspector 打开不得引入横向溢出");
    const bodyOverflowBefore = await page.evaluate(() => document.body.style.overflow);

    // IO 抽屉：与 Inspector 互斥，全宽
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    const drawer = page.locator('[data-testid="io-drawer"]:visible');
    await drawer.waitFor();
    await page.locator('[data-testid="inspector"]').waitFor({ state: "hidden" });
    const drawerBox = await drawer.boundingBox();
    assert.ok(drawerBox && drawerBox.width <= 720, "IO 抽屉宽度不得超出视口");
    assert.ok((await scrollWidth()) <= 720, "IO 抽屉打开不得引入横向溢出");
    // 打开抽屉不得引入 JS 动态背景滚动锁定
    assert.equal(await page.evaluate(() => document.body.style.overflow), bodyOverflowBefore);

    // 抽屉可关闭；关闭后 Inspector 恢复、无滚动锁定残留
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="inspector"]:visible').waitFor();
    assert.equal(await page.evaluate(() => document.body.style.overflow), bodyOverflowBefore);

    // Inspector 可关闭；关键操作仍可达（在视口内）
    await page.locator('[data-testid="btn-inspector-close"]').click();
    await page.locator('[data-testid="inspector"]').waitFor({ state: "hidden" });
    for (const testid of ["btn-more-menu", "tool-add-table", "tool-relationship"]) {
      const box = await page.locator(`[data-testid="${testid}"]`).boundingBox();
      assert.ok(box && box.x >= 0 && box.x + box.width <= 720, `${testid} 必须在 720px 视口内可达`);
    }
  }, { viewport: { width: 720, height: 900 } });

  // ─── ST-PB-01：关系工具点击两点 + 确认条创建 ──────────────────────────────
  await run(["ST-PB-01"], "点击两点 + 确认条创建关系", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    await waitForCanvasStable(page);
    const p1 = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(p1.x, p1.y);
    const p2 = await canvasPoint(page, TABLE2_FIELD);
    await page.mouse.click(p2.x, p2.y);

    // 落点后必须出现确认条；确认后才 references+1
    await page.locator('[data-testid="rel-confirm-bar"]:visible').waitFor();
    await page.locator('[data-testid="rel-confirm-bar"]').getByText(/table_1\.id → table_2\.id/).waitFor();
    await page.locator('[data-testid="rel-confirm-create"]').click();

    // Inspector 可编辑该关系
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "确认后必须落账 1 条关系");
  });

  // ─── ST-PB-02：拖线（≥4px + rubber-band）+ 确认条创建 ─────────────────────
  await run(["ST-PB-02"], "字段拖线 + 确认条创建关系", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    await waitForCanvasStable(page);
    const from = await canvasPoint(page, TABLE1_FIELD);
    const to = await canvasPoint(page, TABLE2_FIELD);
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move((from.x + to.x) / 2, (from.y + to.y) / 2, { steps: 4 });
    // 位移 ≥4px → rubber-band 预览线可见
    const rubber = page.locator('[data-testid="rel-rubber-band"]');
    await page.waitForTimeout(150);
    assert.equal(await rubber.getAttribute("hidden"), null, "拖动中 rubber-band 必须可见");
    assert.notEqual((await rubber.getAttribute("d")) ?? "", "", "rubber-band 必须有路径");
    await page.mouse.move(to.x, to.y, { steps: 2 });
    await page.mouse.up();

    // 落到目标字段 → 确认条可见 → 确认创建
    await page.locator('[data-testid="rel-confirm-bar"]:visible').waitFor();
    await page.locator('[data-testid="rel-confirm-create"]').click();
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "拖线确认后必须落账 1 条关系");
  });

  // ─── ST-CR-02：拖表跟手 + 松手 GRID_SIZE=20 吸附 ──────────────────────────
  await run(["ST-CR-02"], "拖表过程连线跟手；松手吸附 20 网格", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 先建一条关系（点击两点 + 确认），再 Esc 退出关系工具
    await page.keyboard.press("r");
    const p1 = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(p1.x, p1.y);
    const p2 = await canvasPoint(page, TABLE2_FIELD);
    await page.mouse.click(p2.x, p2.y);
    await page.locator('[data-testid="rel-confirm-bar"]:visible').waitFor();
    await page.locator('[data-testid="rel-confirm-create"]').click();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="rel-tool-hint"]').waitFor({ state: "hidden" });
    await waitForCanvasStable(page);

    const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
    const pathBefore = await canvas.getAttribute("data-follow-path");
    assert.ok(pathBefore && pathBefore !== "", "关系线几何必须暴露在 data-follow-path");

    // 拖动 table_2 表头：pointerup 前采样 —— 连线必须跟手（非松手跳变）
    const start = await canvasPoint(page, TABLE2_HEADER);
    await page.mouse.move(start.x, start.y);
    await page.mouse.down();
    await page.mouse.move(start.x + 97, start.y + 63, { steps: 5 });
    await page.waitForTimeout(150); // rAF 重绘
    const pathDuring = await canvas.getAttribute("data-follow-path");
    assert.notEqual(pathDuring, pathBefore, "拖动中连线路径必须随表位置更新（跟手）");
    await page.mouse.up();

    // 松手吸附 GRID_SIZE=20：raw (332,243) → (340,240)，随保存 PUT 落账
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    const table2 = state.lastPutBody?.diagram?.tables?.find(t => t.name === "table_2");
    assert.ok(table2, "PUT 请求体必须包含 table_2");
    assert.equal(table2.x, 340, "松手后 x 必须吸附为 20 的倍数（332→340）");
    assert.equal(table2.y, 240, "松手后 y 必须吸附为 20 的倍数（243→240）");
  });
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
