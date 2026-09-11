// implement-unified-prototype-spec-parity C 批 — room-editor 壳层 / 保存态 / 协作可见性 e2e
//
// 覆盖用例（落地后须从 SPEC_PARITY_SKIP_IDS 移除）：
//   ST-S01-SS-01            保存态文案（dirty→op 入队；room 模式无全量 PUT，ack→saved 走真实链路）
//   ST-S01-409-SCOPE        协作已连接态快照 409 禁模态（Activity 反馈 + 采纳服务器 rev）
//   ST-S01-NO-409-OT        两用户连接态近同时编辑：无 409 模态、ot-rev 一致、Activity 有记录
//   ST-S01-409-LOCAL-ONLY   仅本地编辑：op 排队 + 风险文案常驻（方案B 后无全量 PUT/409 模态）
//   ST-S05-UI-01            进房建立协作：ws-status 已连接、ot-rev 显示 serverRev
//   ST-S05-UI-02            连接态编辑：ack（Activity）+ 无 409 模态 + ot-rev 一致
//   ST-S05-UI-03            room-presence 可见 + 不遮挡本地选中
//   ST-S05-UI-04            断连继续编辑：队列计数 + banner；重连 sync 后清零
//   ST-S05-UI-05            仅本地编辑：409 风险文案 + 本地可编辑 + 不误报 OT 同步
//   ST-S05-UI-06            Viewer：写入口禁用、不产生 op、ot-rev 不递增
//   ST-FE-ALIGN-03          ws-status/ot-rev/room-presence 来自真实 REST 或明确降级
//   ST-FE-ALIGN-04          协作已连接态禁止 S01 409 模态
//   ST-PU-24                room-editor 可见 ws-status / ot-rev / room-presence
//
// 说明：生产前端协作历史上为 REST head 明确降级（无 WS 客户端）。
// wire-frontend-collab-ws（2026-09）起生产前端接入真实 WS，以下 5 个依赖降级形态的
// mock 用例转为 skip（SUPERSEDED_BY_REAL_WS），由 ST-FE-S05-07~09 真实双上下文链路
// 与 UT-FE-S05-13~17 单测覆盖：ST-S05-UI-01/02/04/06、ST-PU-24、ST-FE-ALIGN-03/04、
// ST-S01-409-SCOPE、ST-S01-NO-409-OT。
// fix-collab-autosave-race（2026-09-10）起协作持久化改为服务端物化单写者：
// room 模式不再产生全量 PUT（ops 即保存，dirty 由 op ack 对账驱动；PUT room 图 →
// 409 USE_OP_CHANNEL），SS-01 / 409-LOCAL-ONLY+UI-05 / UI-03 三用例断言随之改写
// （不再有 saving 阶段与 409 模态；ack→saved 由 ST-FE-S05-07/10 真实链路覆盖）。
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

// 先同步 trunk build：冷环境首建远超 waitForServer 的 30s 窗口，且 serve 重建期间会提供旧 dist
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
    { userId: "member-2", email: "chen@example.com", displayName: "陈晨", role: "editor", joinedAt: "2026-08-23T00:01:00Z" },
  ],
};
const HEAD_OK = { roomId: "room-new", diagramId: "diagram-new", serverRev: 7 };

// options:
//   putBehavior: "ok" | "slow" | "conflict"   PUT /diagrams/diagram-new 行为
//   headMode:    "ok" | "fail" | "flaky"      collab/head 行为（flaky = 先败后成，由 state.headOk 翻转）
//   roomsList:   [{...RoomSummary}]           预置房间列表（Viewer 场景）
async function installApi(page, options = {}) {
  const state = {
    requests: [],
    putCalls: 0,
    headCalls: 0,
    headOk: false,
    ...options.state,
  };
  const putBehavior = options.putBehavior ?? "ok";
  const headMode = options.headMode ?? "ok";
  let diagramRev = 0;

  // fix-listview-grid-and-room-dbtype: 离线环境兜底——abort Google Fonts 渲染阻塞请求
  await page.route("https://fonts.googleapis.com/**", route => route.abort());
  await page.route("https://fonts.gstatic.com/**", route => route.abort());
  // fix-canvas-zoom-invite-comment-resize（UT-S04-UI-17）：前端 base_url 同源派生，按 path 匹配
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
        diagramTitle: "架构评审室", myRole: options.myRole ?? "owner", memberCount: 2,
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
      if (putBehavior === "conflict") {
        return response(route, 409, { code: 409, message: "revision conflict", details: { current_revision: 5 } });
      }
      if (putBehavior === "slow") await new Promise(resolve => setTimeout(resolve, 600));
      diagramRev += 1;
      return response(route, 200, { code: 0, request_id: "save-diagram", data: { revision: diagramRev } });
    }
    if (url.pathname.endsWith("/collab/head")) {
      state.headCalls += 1;
      const ok = headMode === "ok" || (headMode === "flaky" && state.headOk);
      if (!ok) return response(route, 500, { code: "HEAD_UNAVAILABLE", message: "collab head unavailable" });
      return response(route, 200, HEAD_OK);
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

let frontend = null;
let browser;
let failed = false;

async function run(ids, title, body) {
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
  } finally {
    await page.close();
  }
}

// wire-frontend-collab-ws：被真实 WS 链路取代的 mock 用例统一 skip 上报（保持台账覆盖连续）。
const SUPERSEDED_BY_REAL_WS =
  "wire-frontend-collab-ws：生产前端已接入真实 WS（原 REST head 降级形态已移除），" +
  "本 mock 用例由 tests/e2e/s05-collab.spec.ts 的 ST-FE-S05-07~09 真实双上下文链路 + " +
  "collab_client.rs UT-FE-S05-13~17 覆盖";

async function skip(ids, title, reason) {
  for (const id of ids) reportOpenLogos({ id, status: "skip", durationMs: 0, error: reason });
  process.stdout.write(`SKIP ${ids.join(", ")} ${title}: ${reason}\n`);
}

try {
  await prebuildFrontend();
  frontend = startFrontend();
  await waitForServer(FRONTEND_URL);
  browser = await chromium.launch({
    headless: true,
    executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
  });

  // ─── 协作锚点：ws-status / ot-rev / room-presence ────────────────────────
  await skip(["ST-S05-UI-01", "ST-PU-24", "ST-FE-ALIGN-03"], "进房后协作锚点来自真实 REST head", SUPERSEDED_BY_REAL_WS);

  // ─── 保存态文案阶段 ──────────────────────────────────────────────────────
  await run(["ST-S01-SS-01"], "保存态 dirty→op 入队（room 模式无全量 PUT）", async page => {
    const state = await installApi(page, { putBehavior: "slow" });
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor();

    await page.locator('[data-testid="tool-add-table"]').click();
    // 编辑后立即进入 dirty
    await page.locator('[data-testid="save-state"][data-state="dirty"]').getByText("有未保存更改").waitFor();
    await page.locator('[data-testid="status-pending-ops"]').getByText("1 项待同步").waitFor();
    // fix-collab-autosave-race：room 模式 ops 即保存——越过原 debounce 窗口也不产生全量 PUT
    // （无 saving 阶段）；dirty→saved 由 WS ack 驱动，ack 链路见 ST-FE-S05-07/10 真实双上下文。
    await page.waitForTimeout(1_500);
    assert.ok(!state.requests.some(r => r.startsWith("PUT ")), "room 模式禁止全量 PUT");
  });

  // ─── 协作已连接态快照 409：禁模态 ────────────────────────────────────────
  await skip(["ST-S01-409-SCOPE", "ST-FE-ALIGN-04"], "协作连接态快照 409 不弹 S01 模态", SUPERSEDED_BY_REAL_WS);

  // ─── 两用户连接态编辑：无 409 模态 + ot-rev 一致 + Activity ─────────────
  await skip(["ST-S01-NO-409-OT", "ST-S05-UI-02"], "双端连接态编辑无 409 模态且 ot-rev 一致", SUPERSEDED_BY_REAL_WS);

  // ─── 仅本地编辑：风险文案 + 本地可编辑 + 不产生全量 PUT/409 模态 ─────────
  await run(["ST-S01-409-LOCAL-ONLY", "ST-S05-UI-05"], "仅本地编辑：op 排队 + 风险文案常驻 + 无 PUT/409 模态", async page => {
    const state = await installApi(page, { headMode: "fail", putBehavior: "conflict" });
    await login(page);
    await createRoomAndEnter(page);
    // head 失败 → Reconnecting banner
    const banner = page.locator('[data-testid="reconnect-banner"]:visible');
    await banner.waitFor();
    await banner.getByText("连接已断开，正在重连…").waitFor();

    // 选择仅本地编辑
    await page.locator('[data-testid="btn-local-only"]').click();
    await page.locator('[data-testid="ws-status"]').getByText("仅本地 · 409 风险").waitFor();
    await banner.getByText("仅本地编辑中，更改可能产生 409 冲突").waitFor();

    // 本地可编辑（不误报 OT 已同步）
    await page.locator('[data-testid="tool-add-table"]').click();
    await page.locator('[data-testid="status-pending-ops"]').getByText("1 项待同步").waitFor();
    const wsText = await page.locator('[data-testid="ws-status"]').textContent();
    assert.doesNotMatch(wsText ?? "", /已连接 · OT 同步/, "仅本地态禁止误报 OT 已同步");

    // fix-collab-autosave-race：room 写只走 op 通道（PUT 一律 409 USE_OP_CHANNEL），
    // 前端 room 模式不再发全量 PUT——更改留在 op 队列待重连 flush，不再出现 409 模态。
    await page.waitForTimeout(1_500);
    assert.ok(!state.requests.some(r => r.startsWith("PUT ")), "仅本地编辑不产生全量 PUT");
    assert.equal(await page.locator('[data-testid="modal-conflict"]:visible').count(), 0, "room 模式不再有 409 模态");
    // 风险文案常驻
    await page.locator('[data-testid="ws-status"]').getByText("仅本地 · 409 风险").waitFor();
  });

  // ─── 断连继续编辑 → 重连 sync 队列清零 ──────────────────────────────────
  await skip(["ST-S05-UI-04"], "断连排队编辑，重连后队列清零", SUPERSEDED_BY_REAL_WS);

  // ─── presence 不遮挡本地选中 ─────────────────────────────────────────────
  await run(["ST-S05-UI-03"], "room-presence 可见且不影响本地选中", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="room-presence"]:visible').waitFor();

    // 建表后点击表头区域（diagram 坐标 = 画布 CSS px，默认 pan0 zoom1）→ Inspector 显示该表
    await page.locator('[data-testid="tool-add-table"]').click();
    // fix-collab-autosave-race：room 模式无全量 PUT，编辑落点改等 dirty（op 已入队）而非 saved
    await page.locator('[data-testid="save-state"][data-state="dirty"]').waitFor({ timeout: 8_000 });
    const canvasBox = await page.locator('[data-testid="editor-canvas-container"] canvas').boundingBox();
    assert.ok(canvasBox, "canvas 必须存在");
    // table_1 落位 (180,145)，表头中心 (180+115, 145+20)
    await page.mouse.click(canvasBox.x + 295, canvasBox.y + 165);
    const nameInput = page.locator('[data-testid="inspector-table-name"]');
    await nameInput.waitFor();
    assert.equal(await nameInput.inputValue(), "table_1", "presence 存在时本地选中表必须生效");
  });

  // ─── Viewer 只读：写入口禁用、无 op、ot-rev 不递增 ──────────────────────
  await skip(["ST-S05-UI-06"], "Viewer 写入口禁用且 ot-rev 不递增", SUPERSEDED_BY_REAL_WS);
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
