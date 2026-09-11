// fix-canvas-zoom-perf e2e — 滚轮缩放 rAF 合并 + 锚定 + Inspector 输入不触发画布重绘
//
// 覆盖用例：
//   ST-CR-PAN-01   滚轮缩放/平移连续 ≥10 次事件：window.__cdb_paint_count 增长 ≤ 事件数
//                  （rAF 合并，一帧至多一次 draw_canvas）+ 缩放后光标下对象仍在光标附近（锚定）
//   ST-CR-INSP-01  Inspector `inspector-field-tag` 连续键入 10 字符期间 paint_count 不增长
//                  （重绘仅由画布自身信号驱动）；blur 落账后正常保存
//   ST-CR-GHOST-01 无关系线表拖拽走幽灵层（R-PERF-11）：drag-ghost 节点出现/松手移除、
//                  move 阶段 paint_count 零增长（主画布零重绘）、落账坐标随拖动更新
//
// harness 参照 test-spec-parity-d.mjs（route stub，无需真实后端；端口 4176 避免与 D 批并行冲突）

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";
import { installMockCollabWs } from "./mock-collab-ws.mjs";

const FRONTEND_URL = process.env.CANVAS_PERF_FRONTEND_URL || "http://127.0.0.1:4176";
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
  if (process.env.CANVAS_PERF_FRONTEND_URL) return null;
  const env = { ...process.env };
  delete env.NO_COLOR;
  delete env.FORCE_COLOR;
  return spawn("trunk", ["serve", "--port", "4176"], {
    cwd: new URL("..", import.meta.url),
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

// trunk serve 在重建期间会继续提供旧 dist；先同步 trunk build 保证测试跑到当前源码
async function prebuildFrontend() {
  if (process.env.CANVAS_PERF_FRONTEND_URL) return;
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

// 与 test-spec-parity-d.mjs 同款的 API route stub（仅保留画布用例需要的分支）
async function installApi(page) {
  const state = { requests: [], putCalls: 0, lastPutBody: null };
  let diagramRev = 0;

  await page.route("https://fonts.googleapis.com/**", route => route.abort());
  await page.route("https://fonts.gstatic.com/**", route => route.abort());
  await page.route(`${BACKEND_URL}/diagrams/queryAll`, route => response(route, 200, {
    code: 0, message: "success", data: [],
  }));

  // fix-canvas-zoom-invite-comment-resize 期间 base_url 同源派生改造（UT-S04-UI-17）：
  // 生产默认取 window.location.origin，dev 可经 COLDRAWDB_API_BASE 覆盖——
  // 两种形态都 stub：`**/api/v1/**` 同时命中 127.0.0.1:3000 与前端同源 origin。
  await page.route("**/api/v1/**", async route => {
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
      return response(route, 200, { items: [], total: 0 });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "POST") {
      const body = request.postDataJSON();
      return response(route, 201, { id: "room-new", name: body.name, diagramId: "diagram-new", ownerId: "owner-1" });
    }
    if (url.pathname === "/api/v1/diagrams" && request.method() === "POST") {
      return response(route, 200, { code: 0, request_id: "create-diagram", data: { id: "diagram-new" } });
    }
    if (url.pathname === "/api/v1/rooms/room-new" && request.method() === "GET") {
      return response(route, 200, {
        id: "room-new", name: "画布性能验收室", diagramId: "diagram-new", ownerId: "owner-1",
        diagramTitle: "画布性能验收室", myRole: "owner", memberCount: 1,
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "GET") {
      return response(route, 200, {
        code: 0, request_id: "load-diagram",
        data: { id: "diagram-new", name: "画布性能验收室", database: null, revision: diagramRev, tables: [], references: [], areas: [], notes: [] },
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
    process.stderr.write(`[stub-404] ${request.method()} ${url.pathname}\n`);
    return response(route, 404, { code: 404, message: "not stubbed" });
  });

  // fix-collab-autosave-race：mock 协作 WS——op 上行即 ack + 服务端物化（落账断言事实源）
  await installMockCollabWs(page, state, { yourRole: "owner" });
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
  await page.locator('[data-testid="create-room-name"]').fill("画布性能验收室");
  await page.locator('[data-testid="create-room-submit"]').click();
  await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
}

async function waitSaved(page, timeoutMs = 8_000) {
  await page.waitForFunction(
    () => document.querySelector('[data-testid="save-state"]')?.getAttribute("data-state") === "saved",
    { timeout: timeoutMs }
  );
}

// 建 1 张表并等保存落账；表 1 缺省落位 x=180, y=145（表头中心 ≈ (295, 166.5)）
// 偶发 flake：房间页刚挂载时焦点/键盘监听未就绪，"t" 可能丢失——重试至 3 次
async function createOneTable(page) {
  for (let attempt = 1; ; attempt++) {
    await page.keyboard.press("t");
    try {
      await page.locator('[data-testid="inspector-table-name"]').waitFor({ timeout: 5_000 });
      break;
    } catch (e) {
      if (attempt >= 3) throw e;
      await page.waitForTimeout(300);
    }
  }
  await waitSaved(page);
}

async function canvasBox(page) {
  const box = await page.locator('[data-testid="editor-canvas-container"] canvas').boundingBox();
  assert.ok(box, "canvas 必须存在");
  return box;
}

// 表 1 表头中心（pan 0 / zoom 1 下的画布坐标，与 D 批 harness 常量一致）
const TABLE1_HEADER = { x: 295, y: 166.5 };

async function paintCount(page) {
  return page.evaluate(() => window.__cdb_paint_count ?? 0);
}

let frontend = null;
let browser;
let failed = false;

const ONLY = process.env.CANVAS_PERF_ONLY ? new Set(process.env.CANVAS_PERF_ONLY.split(",")) : null;

async function run(ids, title, body) {
  if (ONLY && !ids.some(id => ONLY.has(id))) return;
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
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
    // fix-collab-autosave-race：失败时 dump 协作事件环（与 D 批同款），定位 op 通道断点
    try {
      const collab = await page.evaluate(() => {
        const fn = window.__cdb_collab_state;
        return typeof fn === "function" ? fn() : null;
      });
      if (collab) process.stderr.write(`[collab-dump][${ids.join(",")}] ${collab}\n`);
    } catch {}
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

  // ─── ST-CR-PAN-01：wheel/平移 rAF 合并 + 缩放光标锚定 ─────────────────────
  await run(["ST-CR-PAN-01"], "滚轮缩放/平移 rAF 合并（paint_count 有界）+ 光标锚定", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createOneTable(page);
    const box = await canvasBox(page);
    const anchor = { x: box.x + TABLE1_HEADER.x, y: box.y + TABLE1_HEADER.y };

    // 缩放前：anchor 处点击应选中表 1（证明表头恰在 anchor 下）
    await page.mouse.click(anchor.x, anchor.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.waitForTimeout(200); // 等选中态重绘落账

    const before = await paintCount(page);

    // 10 次滚上（1.1^10 ≈ 2.59x，不触及 ZOOM_MAX）——事件连续触发，transform 经
    // pending_transform 意图通道 rAF 合并落账
    await page.mouse.move(anchor.x, anchor.y);
    for (let i = 0; i < 10; i++) await page.mouse.wheel(0, -100);
    await page.waitForTimeout(400); // 等 rAF 落账 + 重绘

    const afterWheel = await paintCount(page);
    const wheelPaints = afterWheel - before;
    assert.ok(wheelPaints >= 1, `ST-CR-PAN-01: 缩放必须触发重绘（paint_count +${wheelPaints}）`);
    assert.ok(
      wheelPaints <= 10,
      `ST-CR-PAN-01: paint_count 增长（${wheelPaints}）不得超过事件数（10）——rAF 合并失效`
    );

    // 锚定 e2e：同一 client 点再次 pointerdown，表头仍应在光标下（±2px 内——
    // 命中测试本身精确，容差消耗于 device pixel 取整），即仍选中表 1
    await page.mouse.click(anchor.x, anchor.y);
    await page
      .locator('[data-testid="inspector-table-form"]:visible')
      .waitFor({ timeout: 3_000 })
      .catch(() => {
        throw new Error("ST-CR-PAN-01: 缩放后光标下对象漂移——锚定不变量被破坏");
      });

    // 平移合并：空白处按下拖拽 10 步——paint_count 增长 ≤ 事件数
    // （计量窗口从 mouse.down 之后开始：pointerdown 的选中态信号重绘不计入本断言）
    const blank = { x: box.x + 900, y: box.y + 700 };
    await page.mouse.move(blank.x, blank.y);
    await page.mouse.down();
    await page.waitForTimeout(200);
    const beforePan = await paintCount(page);
    for (let i = 1; i <= 10; i++) {
      await page.mouse.move(blank.x + i * 12, blank.y + i * 6);
    }
    await page.mouse.up();
    await page.waitForTimeout(400);
    const panPaints = (await paintCount(page)) - beforePan;
    // perf-canvas-drag-smoothness 口径修订：+1 冗余给 R-PERF-10 松手帧（恢复全分辨率
    // backing store 的一次性重绘）；rAF 合并不变量仍由「≤ 事件数 + 1」保证
    assert.ok(panPaints <= 11, `ST-CR-PAN-01: 平移 paint_count 增长（${panPaints}）≤ 事件数+1（11）`);
  });

  // ─── ST-CR-INSP-01：Inspector 键入不触发画布重绘 ─────────────────────────
  await run(["ST-CR-INSP-01"], "Inspector tag 键入期间 paint_count 不增长", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createOneTable(page);

    // 与 ST-CR-TAG-01 同通路：选中表 → 点字段卡（DOM field-row，canvas 命中是表而非字段）
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.locator('[data-testid^="field-row-"]').first().dispatchEvent("click");
    const tag = page.locator('[data-testid="inspector-field-tag"]');
    await tag.waitFor();
    await page.waitForTimeout(200); // 等表单切换重绘稳定

    const before = await paintCount(page);
    await tag.click();
    for (const ch of "finance-core") await tag.press(ch);
    await page.waitForTimeout(150);
    const during = await paintCount(page);
    assert.equal(
      during, before,
      `ST-CR-INSP-01: 键入 10 字符期间 paint_count 增长 ${during - before}——Inspector 输入不得驱动 draw_canvas`
    );
    assert.equal(await tag.inputValue(), "finance-core");

    // blur 落账：PUT 触发保存（与 ST-CR-TAG-01 不丢字结论共存）
    await page.locator('[data-testid="inspector-title"]').click();
    await waitSaved(page);
    const put = state.lastPutBody;
    const table1 = put?.diagram?.tables?.find(t => t.name === "table_1");
    assert.ok(
      table1,
      `ST-CR-INSP-01: blur 后必须有保存 PUT（含 table_1）putCalls=${state.putCalls} ` +
        `revs=${JSON.stringify(state.sentClientRevs ?? null)} ` +
        `tables=${JSON.stringify(put?.diagram?.tables?.map(t => t.name) ?? null)}`
    );
    assert.equal(
      table1.fields?.[0]?.tag, "finance-core",
      `ST-CR-INSP-01: 落账 fields[0].tag 应为 finance-core，实际 ${table1.fields?.[0]?.tag}`
    );
  });

  // ─── ST-CR-GHOST-01：无关系线表拖拽幽灵层（R-PERF-11，主画布零重绘）─────────
  await run(["ST-CR-GHOST-01"], "无关系线表拖拽幽灵层（move 阶段 paint_count 零增长）", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createOneTable(page);
    const box = await canvasBox(page);
    const header = { x: box.x + TABLE1_HEADER.x, y: box.y + TABLE1_HEADER.y };

    await page.mouse.move(header.x, header.y);
    await page.mouse.down();
    // 首次移动（位移 > 4px 阈值）：幽灵层创建 + 主画布「抠出」重绘一次
    await page.mouse.move(header.x + 12, header.y + 8, { steps: 1 });
    const ghost = page.locator('[data-testid="drag-ghost"]');
    await ghost.waitFor({ timeout: 3_000 });

    const paintAtGhost = await paintCount(page);
    for (let i = 2; i <= 22; i++) {
      await page.mouse.move(header.x + 12 + i * 6, header.y + 8 + i * 2, { steps: 1 });
    }
    const paintBeforeUp = await paintCount(page);
    assert.equal(
      paintBeforeUp, paintAtGhost,
      `ST-CR-GHOST-01: 幽灵层拖拽 move 阶段 paint_count 增长 ${paintBeforeUp - paintAtGhost}——应为 0（主画布零重绘）`
    );

    await page.mouse.up();
    await page.waitForTimeout(300);
    assert.equal(await ghost.count(), 0, "ST-CR-GHOST-01: 松手后幽灵层必须被移除");
    await waitSaved(page);
    const table1 = state.lastPutBody?.diagram?.tables?.find(t => t.name === "table_1");
    assert.ok(table1, "ST-CR-GHOST-01: 松手后必须有保存 PUT（含 table_1）");
    assert.notEqual(table1.x, 180, `ST-CR-GHOST-01: 落账 x 应离开初始 180（拖动已生效），实际 ${table1.x}`);
  });
} finally {
  if (browser) await browser.close();
  if (frontend) frontend.kill();
}

if (failed) process.exitCode = 1;
