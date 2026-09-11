// fix-canvas-zoom-invite-comment-resize E 批 — splitter 分隔条 + 表注释 + 邀请链接 e2e
//
// 覆盖用例：
//   ST-PU-27  拖动 splitter-inspector 改变 Inspector 宽度，钳制 240–520，松手落最终值
//   ST-PU-28  窄视口（720px）两条分隔条 display:none；Inspector 覆盖层仍可用
//   ST-PU-29  拖动分隔条期间画布尺寸跟随（data-follow-path 更新）、#app/.cdb-app 不重建
//   ST-PU-30  ListView 树列宽拖动（钳制 160–420），刷新后从 cdb.list-tree.width 恢复
//   ST-PC-08  Inspector 表注释编辑端到端：blur → PUT 落账 → 表树注释行 → 重载仍在
//   ST-S04-02 邀请链接端到端（被邀请人视角）：inviteUrl 格式 → /invite/{token} preview
//             → guest 登录 accept 入房（myRole=editor / memberCount=2）
//
// harness 复用 spec-parity-d 口径：mock 后端（同源 path 路由）+ trunk build/serve + Playwright。

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";
import { installMockCollabWs } from "./mock-collab-ws.mjs";

const FRONTEND_URL = process.env.SPEC_PARITY_FRONTEND_URL || "http://127.0.0.1:4175";
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

const ROOM_SUMMARY = {
  id: "room-new",
  name: "架构评审室",
  diagramId: "diagram-new",
  diagramTitle: "架构评审室",
  myRole: "owner",
  memberCount: 1,
  updatedAt: "2026-09-08 12:00",
};

// GET /diagrams/{id} 返回 state.savedDiagram（PUT 全量体的 diagram 部分 + revision），
// 支撑 ST-PU-30 / ST-PC-08 的「刷新后恢复」断言（落库链路通）。
async function installApi(page, options = {}) {
  const state = { requests: [], putCalls: 0, lastPutBody: null, inviteCreates: [], invitePreviews: [], inviteAccepts: [], roomGets: [] };
  let diagramRev = 0;

  await page.route("https://fonts.googleapis.com/**", route => route.abort());
  await page.route("https://fonts.gstatic.com/**", route => route.abort());
  await page.route("**/diagrams/queryAll", route => response(route, 200, {
    code: 0, message: "success", data: [],
  }));

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
      const items = options.roomsList ?? [ROOM_SUMMARY];
      return response(route, 200, { items, total: items.length });
    }
    if (url.pathname === "/api/v1/diagrams" && request.method() === "POST") {
      return response(route, 200, { code: 0, request_id: "create-diagram", data: { id: "diagram-new" } });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "POST") {
      const body = request.postDataJSON();
      return response(route, 201, { id: "room-new", name: body.name, diagramId: "diagram-new", ownerId: "owner-1" });
    }
    if (url.pathname === "/api/v1/rooms/room-new" && request.method() === "GET") {
      // ST-S04-02：guest 视角接受邀请后拉取的 RoomDetail（myRole/memberCount 断言）
      const detail = options.roomDetail ?? {
        id: "room-new", name: "架构评审室", diagramId: "diagram-new", ownerId: "owner-1",
        diagramTitle: "架构评审室", myRole: "owner", memberCount: 1,
      };
      state.roomGets.push(detail);
      return response(route, 200, detail);
    }
    // ST-S04-02：owner 生成邀请（inviteUrl 用对外 host，验证前端原样消费）
    if (url.pathname === "/api/v1/rooms/room-new/invites" && request.method() === "POST") {
      const body = request.postDataJSON();
      state.inviteCreates.push(body);
      return response(route, 201, {
        id: "invite-1", token: `inv-${body.role}`,
        inviteUrl: `${options.inviteBase ?? "https://coldrawdb.example.com"}/invite/inv-${body.role}`,
        role: body.role, expiresAt: "2026-09-15T00:00:00Z",
      });
    }
    // ST-S04-02：被邀请人 preview / accept（/api/v1/rooms/invites/{token}[/accept]）
    if (url.pathname.startsWith("/api/v1/rooms/invites/") && url.pathname.endsWith("/accept") && request.method() === "POST") {
      state.inviteAccepts.push(url.pathname.split("/")[5]);
      return response(route, 200, { roomId: "room-new", diagramId: "diagram-new", role: "editor", alreadyMember: false });
    }
    if (url.pathname.startsWith("/api/v1/rooms/invites/")) {
      state.invitePreviews.push(url.pathname.split("/")[5]);
      return response(route, 200, {
        roomName: "架构评审室", diagramTitle: "架构评审室", diagramId: "diagram-new",
        role: "editor", invitedBy: "林默", expiresAt: "2026-09-15T00:00:00Z",
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "GET") {
      const base = state.savedDiagram ?? {
        name: "架构评审室", database: null, tables: [], references: [], areas: [], notes: [],
      };
      return response(route, 200, {
        code: 0, request_id: "load-diagram",
        data: { id: "diagram-new", revision: diagramRev, ...base },
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "PUT") {
      state.putCalls += 1;
      state.lastPutBody = request.postDataJSON();
      state.savedDiagram = state.lastPutBody.diagram;
      diagramRev += 1;
      return response(route, 200, { code: 0, request_id: "save-diagram", data: { revision: diagramRev } });
    }
    if (url.pathname.endsWith("/collab/head")) {
      return response(route, 200, { roomId: "room-new", diagramId: "diagram-new", serverRev: 7 });
    }
    if (url.pathname.endsWith("/members")) return response(route, 200, MEMBERS);
    return response(route, 404, { code: "NOT_FOUND" });
  });
  // fix-collab-autosave-race：mock 协作 WS——op 上行即 ack + 服务端物化
  // （savedDiagram 由物化文档驱动，reload 后 GET 返回，与生产「GET 物化读」一致）
  await installMockCollabWs(page, state, { yourRole: options.myRole ?? "owner" });
  return state;
}

async function login(page) {
  await page.goto(FRONTEND_URL);
  await page.locator('[data-testid="auth-email"]').fill("owner@example.com");
  await page.locator('[data-testid="auth-password"]').fill("Pass1234!");
  await page.locator('[data-testid="login-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
}

async function createRoomAndEnter(page, name = "架构评审室") {
  await page.locator('[data-testid="btn-create-room"]').click();
  await page.locator('[data-testid="modal-create-room"]').waitFor();
  await page.locator('[data-testid="create-room-name"]').fill(name);
  await page.locator('[data-testid="create-room-submit"]').click();
  await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
}

// 刷新后重进房间（localStorage 会话恢复 → 房间列表 → 点卡片进房）
async function reenterRoomAfterReload(page) {
  await page.reload();
  await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
  await page.locator('[data-testid="room-card-room-new"]').click();
  await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
}

async function createTwoTables(page) {
  await page.keyboard.press("t");
  await page.locator('[data-testid="inspector-table-name"]').waitFor();
  await page.keyboard.press("t");
  await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
}

async function waitSaved(page, timeoutMs = 8_000) {
  await page.waitForFunction(
    () => document.querySelector('[data-testid="save-state"]')?.getAttribute("data-state") === "saved",
    { timeout: timeoutMs }
  );
}

// 画布坐标（默认 pan0 zoom1；表 i 落位 x=180+i*55, y=145+i*35）
const TABLE1_FIELD = { x: 220, y: 205 };
const TABLE2_FIELD = { x: 295, y: 240 };

async function canvasPoint(page, point) {
  const box = await page.locator('[data-testid="editor-canvas-container"] canvas').boundingBox();
  assert.ok(box, "canvas 必须存在");
  return { x: box.x + point.x, y: box.y + point.y };
}

// 读 Inspector 实际像素宽（grid 第 3 列 = var(--cdb-inspector-w)）
function inspectorWidth(page) {
  return page.evaluate(() => {
    const el = document.querySelector('[data-testid="inspector"]');
    return el ? el.getBoundingClientRect().width : 0;
  });
}

function treeWidth(page) {
  return page.evaluate(() => {
    const el = document.querySelector('[data-testid="list-tree"]');
    return el ? el.getBoundingClientRect().width : 0;
  });
}

// 在分隔条上按下并水平拖动 deltaX（分段 move，模拟真实 pointermove 序列）
async function dragSplitter(page, testid, deltaX) {
  const box = await page.locator(`[data-testid="${testid}"]`).boundingBox();
  assert.ok(box, `${testid} 必须有可命中的包围盒`);
  const cx = box.x + box.width / 2;
  const cy = box.y + Math.min(box.height / 2, 300);
  await page.mouse.move(cx, cy);
  await page.mouse.down();
  const steps = 10;
  for (let i = 1; i <= steps; i += 1) {
    await page.mouse.move(cx + (deltaX * i) / steps, cy);
  }
  return { cx, cy };
}

let frontend = null;
let browser;
let failed = false;

// 调试：SPEC_PARITY_ONLY=ST-PU-27 只跑指定用例（逗号分隔）
const ONLY = process.env.SPEC_PARITY_ONLY ? new Set(process.env.SPEC_PARITY_ONLY.split(",")) : null;

async function run(ids, title, body, options = {}) {
  if (ONLY && !ids.some(id => ONLY.has(id))) return;
  const page = await browser.newPage({ viewport: options.viewport ?? { width: 1440, height: 900 } });
  // 诊断：抓取 wasm panic（pageerror）与 console.error，定位间歇性超时根因
  page.on("pageerror", (e) => process.stderr.write(`[pageerror][${ids.join(",")}] ${String(e).slice(0, 500)}\n`));
  page.on("console", (m) => {
    if (m.type() === "error") process.stderr.write(`[console.error][${ids.join(",")}] ${m.text().slice(0, 500)}\n`);
  });
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

  // ─── ST-PU-27：拖动 splitter-inspector 实时改宽 + 钳制 240–520 ──────────
  await run(["ST-PU-27"], "拖动 splitter-inspector 实时改宽并钳制 240–520", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="splitter-inspector"]').waitFor();

    // 左拖 -200：Inspector 左缘分隔条左拖变宽（§14.2 第 2 条）330 → 530 超界 → 钳制 520；拖动中（is-resizing 无过渡）实时跟随
    await dragSplitter(page, "splitter-inspector", -200);
    await page.waitForTimeout(120); // rAF 重排
    const during = await inspectorWidth(page);
    assert.ok(
      during > 400 && during <= 521,
      `ST-PU-27: 拖动中 Inspector 宽应实时跟随（>400 且 ≤520），实际 ${during}`
    );
    await page.mouse.up();
    await page.waitForTimeout(350); // 恢复过渡 160ms 后稳定
    const clampedMax = await inspectorWidth(page);
    assert.ok(
      Math.abs(clampedMax - 520) <= 1,
      `ST-PU-27: 左拖越过 520 边界应停在 520，实际 ${clampedMax}`
    );

    // 右拖 +400：520 → 120 超界 → 钳制 240（右拖变窄）
    await dragSplitter(page, "splitter-inspector", 400);
    await page.mouse.up();
    await page.waitForTimeout(350);
    const clampedMin = await inspectorWidth(page);
    assert.ok(
      Math.abs(clampedMin - 240) <= 1,
      `ST-PU-27: 右拖越过 240 边界应停在 240，实际 ${clampedMin}`
    );
  });

  // ─── ST-PU-28：720px 窄视口两条分隔条 display:none，Inspector 覆盖层可用 ─
  await run(["ST-PU-28"], "720px 分隔条隐藏 + Inspector 覆盖层可用", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    const hidden = await page.evaluate(() => {
      const a = document.querySelector('[data-testid="splitter-inspector"]');
      const b = document.querySelector('[data-testid="splitter-list-tree"]');
      return {
        inspector: a ? getComputedStyle(a).display === "none" : true,
        // ListView 未打开时树列分隔条不在 DOM（cdb-main 隐藏），视为不可见
        listTree: b ? getComputedStyle(b).display === "none" : true,
      };
    });
    assert.ok(hidden.inspector, "ST-PU-28: 720px 下 splitter-inspector 必须 display:none");
    assert.ok(hidden.listTree, "ST-PU-28: 720px 下 splitter-list-tree 必须不可见");

    // Inspector 覆盖层行为仍可用（720px 紧凑布局）
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]:visible').waitFor();
    await page.locator('[data-testid="btn-inspector-close"]').click();

    // ListView 中树列分隔条同样隐藏
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    const treeHidden = await page.evaluate(() => {
      const b = document.querySelector('[data-testid="splitter-list-tree"]');
      return b ? getComputedStyle(b).display === "none" : true;
    });
    assert.ok(treeHidden, "ST-PU-28: ListView 中 splitter-list-tree 必须 display:none");
  }, { viewport: { width: 720, height: 800 } });

  // ─── ST-PU-29：拖动期间画布尺寸跟随 + 关系 path 更新 + 不重建 .cdb-app ────
  await run(["ST-PU-29"], "拖动分隔条画布跟随，.cdb-app 不重建", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 关系工具点击两点落账
    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    let p = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(p.x, p.y);
    p = await canvasPoint(page, TABLE2_FIELD);
    await page.mouse.click(p.x, p.y);
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await waitSaved(page);

    const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
    const pathBefore = await canvas.getAttribute("data-follow-path");
    assert.ok(pathBefore && pathBefore.startsWith("M"), "ST-PU-29: 关系线 data-follow-path 必须存在");
    const canvasWidthBefore = await page.evaluate(() => {
      return document.querySelector('[data-testid="editor-canvas-container"]').clientWidth;
    });

    // 挂 body childList 观察器 + 记录 .cdb-app 节点身份（ST-PU-19 不变量）
    await page.evaluate(() => {
      window.__cdbMutations = 0;
      window.__cdbAppEl = document.querySelector(".cdb-app");
      new MutationObserver(recs => { window.__cdbMutations += recs.length; })
        .observe(document.body, { childList: true });
    });

    // 右拖 +200 收窄 Inspector（左缘分隔条右拖变窄，§14.2 第 2 条）→ 画布变宽；拖动中（is-resizing 无过渡）即时跟随
    await dragSplitter(page, "splitter-inspector", 200);
    await page.waitForTimeout(150); // rAF 重排
    const canvasDuring = await canvas.boundingBox();
    assert.ok(
      canvasDuring && canvasDuring.width > canvasWidthBefore + 60,
      `ST-PU-29: 拖动中画布宽应即时跟随网格重排（${canvasWidthBefore} → ${canvasDuring?.width}）`
    );
    await page.mouse.up();
    await page.waitForTimeout(350);

    const canvasWidthAfter = await page.evaluate(() => {
      return document.querySelector('[data-testid="editor-canvas-container"]').clientWidth;
    });
    // Inspector 330 → 240（钳制下限）：画布净增 ~90px（含 gap 变化），阈值取 60 容忍边框取整
    assert.ok(
      canvasWidthAfter > canvasWidthBefore + 60,
      `ST-PU-29: 画布 clientWidth 应随 Inspector 收窄即时增大（${canvasWidthBefore} → ${canvasWidthAfter}）`
    );
    const identity = await page.evaluate(() => ({
      same: window.__cdbAppEl === document.querySelector(".cdb-app"),
      mutations: window.__cdbMutations,
    }));
    assert.ok(identity.same, "ST-PU-29: .cdb-app 节点身份必须保持（不重建）");
    assert.equal(identity.mutations, 0, "ST-PU-29: 拖动全程 body childList 零变更");
    assert.ok(state.putCalls >= 2, "ST-PU-29: 建表 + 关系落账 PUT 应已发生");
  });

  // ─── ST-PU-30：ListView 树列拖动钳制 + 刷新恢复 ─────────────────────────
  await run(["ST-PU-30"], "树列宽拖动钳制 160–420 + 刷新恢复", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();
    await waitSaved(page);

    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    await page.locator('[data-testid="splitter-list-tree"]').waitFor();

    // 右拖 +300：230 → 530 超界 → 钳制 420
    await dragSplitter(page, "splitter-list-tree", 300);
    await page.mouse.up();
    await page.waitForTimeout(300);
    const clamped = await treeWidth(page);
    assert.ok(
      Math.abs(clamped - 420) <= 1,
      `ST-PU-30: 树列拖过 420 边界应停在 420，实际 ${clamped}`
    );
    // localStorage 已持久化
    const stored = await page.evaluate(() => localStorage.getItem("cdb.list-tree.width"));
    assert.equal(stored, "420", "ST-PU-30: pointerup 后应写入 cdb.list-tree.width=420");

    // 刷新 → 重进房间 → ListView：宽度从 localStorage 恢复
    await reenterRoomAfterReload(page);
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    const restoredVar = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--cdb-list-tree-w").trim()
    );
    assert.equal(restoredVar, "420px", `ST-PU-30: 刷新后 --cdb-list-tree-w 应恢复 420px，实际 ${restoredVar}`);
    const restored = await treeWidth(page);
    assert.ok(
      Math.abs(restored - 420) <= 1,
      `ST-PU-30: 刷新后树列实际宽应恢复 420，实际 ${restored}`
    );
  });

  // ─── ST-PC-08：Inspector 表注释编辑端到端 ────────────────────────────────
  await run(["ST-PC-08"], "Inspector 表注释 blur 落账 + 表树展示 + 重载仍在", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();
    await page.locator('[data-testid="inspector-table-comment"]').fill("用户表");
    // blur：点 Inspector 标题移开焦点
    await page.locator('[data-testid="inspector-title"]').click();
    await waitSaved(page);

    // 1. PUT 请求体 table.comment 一致
    const table = state.lastPutBody?.diagram?.tables?.[0];
    assert.ok(table, "ST-PC-08: PUT 请求体必须包含表");
    assert.equal(table.comment, "用户表", "ST-PC-08: PUT 请求体 table.comment 必须为「用户表」");
    const tableId = table.id;
    assert.ok(tableId, "ST-PC-08: PUT 表必须有 id");

    // 2. 表树渲染 list-table-comment-{id} 且文本一致
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    const commentLine = page.locator(`[data-testid="list-table-comment-${tableId}"]`);
    await commentLine.waitFor();
    assert.equal(
      (await commentLine.innerText()).trim(),
      "用户表",
      "ST-PC-08: 表树注释行文本必须为「用户表」"
    );

    // 3. 重载后注释仍在（GET 返回 PUT 落库的 diagram）
    await reenterRoomAfterReload(page);
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    const reloaded = page.locator(`[data-testid="list-table-comment-${tableId}"]`);
    await reloaded.waitFor();
    assert.equal(
      (await reloaded.innerText()).trim(),
      "用户表",
      "ST-PC-08: 重载后表树注释行仍应展示「用户表」（落库链路通）"
    );
  });

  // ─── ST-S04-02：邀请链接端到端（被邀请人视角）────────────────────────────
  await run(["ST-S04-02"], "inviteUrl 格式 → /invite/{token} preview → guest accept 入房", async page => {
    // ── Owner 侧：进房并生成邀请链接 ──
    const ownerState = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="btn-invite"]').click();
    const modal = page.locator('[data-testid="modal-invite"]:visible');
    await modal.waitFor();
    const urlInput = modal.locator('[data-testid="invite-url"]');
    await urlInput.waitFor();
    const inviteUrl = await urlInput.inputValue();

    // 断言 1：^{scheme}://{host}(:port)?/invite/{token}$，不含 localhost / 127.0.0.1
    assert.match(
      inviteUrl,
      /^https:\/\/[a-z0-9.-]+(:[0-9]{1,5})?\/invite\/[A-Za-z0-9_-]+$/,
      `ST-S04-02: inviteUrl 必须符合对外地址格式，实际 ${inviteUrl}`
    );
    assert.ok(
      !/localhost|127\.0\.0\.1/.test(inviteUrl),
      `ST-S04-02: inviteUrl 不得含 localhost/127.0.0.1，实际 ${inviteUrl}`
    );
    const token = inviteUrl.split("/invite/")[1];
    assert.equal(token, "inv-editor");
    assert.deepEqual(ownerState.inviteCreates, [{ role: "editor" }]);

    // ── Guest 侧：独立上下文页面走 /invite/{token} 链路 ──
    const guest = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    try {
      const guestState = await installApi(guest, {
        // accept 后拉取的 RoomDetail：被邀请人以 editor 入房，成员 2 人
        roomDetail: {
          id: "room-new", name: "架构评审室", diagramId: "diagram-new", ownerId: "owner-1",
          diagramTitle: "架构评审室", myRole: "editor", memberCount: 2,
        },
      });
      await guest.goto(`${FRONTEND_URL}/invite/${token}`);
      await guest.locator('[data-testid="invite-accept-page"]:visible').waitFor();
      await guest.locator('[data-testid="invite-preview"]:visible').waitFor();

      // 断言 2：preview 200，roomName / diagramId / role=editor 透传到 UI
      assert.deepEqual(guestState.invitePreviews, [token], `ST-S04-02: guest 视角必须 GET invites/{token} preview，实际 ${JSON.stringify(guestState.invitePreviews)} token=${token}`);
      const meta = await guest.locator('[data-testid="invite-meta"]').innerText();
      assert.match(meta, /邀请人：林默/, `ST-S04-02: preview invitedBy 应展示，实际 ${meta}`);
      assert.match(meta, /角色：editor/, `ST-S04-02: preview role 必须为 editor，实际 ${meta}`);
      const previewBody = await guest.locator('[data-testid="invite-preview"]').innerText();
      assert.match(previewBody, /架构评审室/, "ST-S04-02: preview 应展示 roomName/diagramTitle");

      // 未登录点接受 → 登录提示 → 「切换登录」→ guest 登录 → 自动回邀请页续接
      await guest.locator('[data-testid="btn-accept-invite"]').click();
      await guest.locator('[data-testid="invite-login-required"]:visible').waitFor();
      await guest.locator('[data-testid="btn-invite-goto-login"]').click();
      await guest.locator('[data-testid="auth-email"]:visible').waitFor();
      await guest.locator('[data-testid="auth-email"]').fill("guest@example.com");
      await guest.locator('[data-testid="auth-password"]').fill("Guest1234!");
      await guest.locator('[data-testid="login-submit"]').click();
      // 登录成功自动回到邀请页（invite_token 内存续接，§7.3）
      await guest.locator('[data-testid="invite-accept-page"]:visible').waitFor();
      await guest.locator('[data-testid="invite-preview"]:visible').waitFor();
      await guest.locator('[data-testid="btn-accept-invite"]').click();
      try {
        await guest.locator('[data-testid="room-editor-page"]:visible').waitFor({ timeout: 10_000 });
      } catch (err) {
        const inviteErr = await guest.locator('[data-testid="invite-error"]').count()
          ? await guest.locator('[data-testid="invite-error"]').innerText() : "(无 invite-error)";
        const bodyText = await guest.evaluate(() => document.body.innerText.replace(/\s+/g, " ").slice(0, 300));
        throw new Error(`ST-S04-02: accept 后未进入 room-editor；invite-error=${inviteErr}；页面=${bodyText}；请求=${guestState.requests.join(",")}`);
      }

      // 断言 3：accept 200；GET /rooms/{roomId} 的 myRole==editor、memberCount==2
      assert.deepEqual(guestState.inviteAccepts, [token], "ST-S04-02: accept 必须 POST invites/{token}/accept");
      const detail = guestState.roomGets.at(-1);
      assert.equal(detail.myRole, "editor", "ST-S04-02: guest 入房 myRole 必须为 editor");
      assert.equal(detail.memberCount, 2, "ST-S04-02: accept 后 memberCount 必须为 2");
      await guest.locator('[data-testid="room-badge"]:visible').waitFor();
    } finally {
      await guest.close();
    }
  });
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
