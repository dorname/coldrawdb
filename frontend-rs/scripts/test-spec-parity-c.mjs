// implement-unified-prototype-spec-parity C 批 — room-editor 壳层 / 保存态 / 协作可见性 e2e
//
// 覆盖用例（落地后须从 SPEC_PARITY_SKIP_IDS 移除）：
//   ST-S01-SS-01            保存态文案阶段（dirty→saving→saved + rev 推进）
//   ST-S01-409-SCOPE        协作已连接态快照 409 禁模态（Activity 反馈 + 采纳服务器 rev）
//   ST-S01-NO-409-OT        两用户连接态近同时编辑：无 409 模态、ot-rev 一致、Activity 有记录
//   ST-S01-409-LOCAL-ONLY   仅本地编辑后 PUT 409 允许 S01 模态 + 风险文案常驻
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
// 说明：生产前端协作当前为 REST head 明确降级（无 WS 客户端），凡涉及「远端实时推送」
// 的断言均以降级形态核对（双端各自 REST 取 head，ot-rev 一致；presence 仅自身在线）。
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

try {
  await prebuildFrontend();
  frontend = startFrontend();
  await waitForServer(FRONTEND_URL);
  browser = await chromium.launch({
    headless: true,
    executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
  });

  // ─── 协作锚点：ws-status / ot-rev / room-presence ────────────────────────
  await run(["ST-S05-UI-01", "ST-PU-24", "ST-FE-ALIGN-03"], "进房后协作锚点来自真实 REST head", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();
    await page.locator('[data-testid="ot-rev"]').getByText("server_rev 7").waitFor();
    const presence = page.locator('[data-testid="room-presence"]:visible');
    await presence.waitFor();
    // 降级事实：REST 无远端在线信号，仅自身显示在线光斑（成员 member-2 离线被过滤）
    assert.equal(await presence.locator(".cdb-presence-person").count(), 1);
    assert.equal(await presence.locator('[data-testid="presence-online"]').count(), 1);
    const title = await presence.locator(".cdb-presence-person").first().getAttribute("title");
    assert.match(title ?? "", /林默 · owner/);
    await page.locator('[data-testid="status-role"]').getByText("owner").waitFor();
  });

  // ─── 保存态文案阶段 ──────────────────────────────────────────────────────
  await run(["ST-S01-SS-01"], "保存态 dirty→saving→saved 与 rev 推进", async page => {
    await installApi(page, { putBehavior: "slow" });
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor();

    await page.locator('[data-testid="tool-add-table"]').click();
    // 编辑后立即进入 dirty（debounce 静默期内）
    await page.locator('[data-testid="save-state"][data-state="dirty"]').getByText("有未保存更改").waitFor();
    // PUT 进行中（mock 延迟 600ms）→ saving
    await page.locator('[data-testid="save-state"][data-state="saving"]').getByText("保存中…").waitFor({ timeout: 8_000 });
    // 成功 → saved + revision +1
    await page.locator('[data-testid="save-state"][data-state="saved"]').getByText("已保存").waitFor({ timeout: 8_000 });
    await page.locator('[data-testid="revision-display"]').getByText(" · rev 1").waitFor();
  });

  // ─── 协作已连接态快照 409：禁模态 ────────────────────────────────────────
  await run(["ST-S01-409-SCOPE", "ST-FE-ALIGN-04"], "协作连接态快照 409 不弹 S01 模态", async page => {
    await installApi(page, { putBehavior: "conflict" });
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();

    await page.locator('[data-testid="tool-add-table"]').click();
    // PUT 409 后：禁止 modal-conflict；采纳服务器 rev；Activity 反馈
    await page.locator('[data-testid="revision-display"]').getByText(" · rev 5").waitFor({ timeout: 8_000 });
    await page.waitForTimeout(400);
    assert.equal(await page.locator('[data-testid="modal-conflict"]').count(), 0, "协作连接态禁止弹 S01 409 模态");
    await page.locator('[data-testid="activity-feed"]').getByText("快照 409 已由协作合并 · 推进至 rev 5").waitFor();
    await page.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();
  });

  // ─── 两用户连接态编辑：无 409 模态 + ot-rev 一致 + Activity ─────────────
  await run(["ST-S01-NO-409-OT", "ST-S05-UI-02"], "双端连接态编辑无 409 模态且 ot-rev 一致", async page => {
    // page = 用户 A；另开用户 B
    const pageB = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    try {
      await installApi(page);
      await installApi(pageB);
      await login(page);
      await login(pageB);
      await createRoomAndEnter(page);
      await createRoomAndEnter(pageB);
      await pageB.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();

      // A 创建表（op 入队 + Activity 记录 + 快照 PUT 成功）
      await page.locator('[data-testid="tool-add-table"]').click();
      await page.locator('[data-testid="activity-feed"]').getByText(/本地创建表 table_1，等待 OT ack/).waitFor();
      await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });

      // 两端：无 S01 409 模态；ot-rev 一致（REST head 降级形态）
      await page.waitForTimeout(400);
      assert.equal(await page.locator('[data-testid="modal-conflict"]').count(), 0);
      assert.equal(await pageB.locator('[data-testid="modal-conflict"]').count(), 0);
      const revA = await page.locator('[data-testid="ot-rev"]').textContent();
      const revB = await pageB.locator('[data-testid="ot-rev"]').textContent();
      assert.equal(revA?.trim(), "server_rev 7");
      assert.equal(revB?.trim(), revA?.trim(), "两端 ot-rev 必须一致");
    } finally {
      await pageB.close();
    }
  });

  // ─── 仅本地编辑：风险文案 + 本地可编辑 + 409 模态允许 ───────────────────
  await run(["ST-S01-409-LOCAL-ONLY", "ST-S05-UI-05"], "仅本地编辑后 409 允许模态且风险文案常驻", async page => {
    await installApi(page, { headMode: "fail", putBehavior: "conflict" });
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

    // PUT 409 → S01 模态允许出现
    await page.locator('[data-testid="modal-conflict"]:visible').waitFor({ timeout: 8_000 });
    // 风险文案在模态出现后仍常驻
    await page.locator('[data-testid="ws-status"]').getByText("仅本地 · 409 风险").waitFor();
  });

  // ─── 断连继续编辑 → 重连 sync 队列清零 ──────────────────────────────────
  await run(["ST-S05-UI-04"], "断连排队编辑，重连后队列清零", async page => {
    const state = await installApi(page, { headMode: "flaky" });
    await login(page);
    await createRoomAndEnter(page);
    const banner = page.locator('[data-testid="reconnect-banner"]:visible');
    await banner.waitFor();

    // 断连期间继续编辑 → 队列计数可见
    await page.locator('[data-testid="tool-add-table"]').click();
    await page.locator('[data-testid="status-pending-ops"]').getByText("1 项待同步").waitFor();
    await banner.getByText(/1 项更改已排队/).waitFor();

    // 恢复 head → 手动重连 → sync 后队列清零、banner 消失
    state.headOk = true;
    await page.locator('[data-testid="btn-reconnect-now"]').click();
    await page.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();
    await page.locator('[data-testid="reconnect-banner"]').waitFor({ state: "hidden" });
    assert.equal(await page.locator('[data-testid="status-pending-ops"]').count(), 0, "重连 sync 后队列必须清零");
    await page.locator('[data-testid="ot-rev"]').getByText("server_rev 7").waitFor();
  });

  // ─── presence 不遮挡本地选中 ─────────────────────────────────────────────
  await run(["ST-S05-UI-03"], "room-presence 可见且不影响本地选中", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.locator('[data-testid="room-presence"]:visible').waitFor();

    // 建表后点击表头区域（diagram 坐标 = 画布 CSS px，默认 pan0 zoom1）→ Inspector 显示该表
    await page.locator('[data-testid="tool-add-table"]').click();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    const canvasBox = await page.locator('[data-testid="editor-canvas-container"] canvas').boundingBox();
    assert.ok(canvasBox, "canvas 必须存在");
    // table_1 落位 (180,145)，表头中心 (180+115, 145+20)
    await page.mouse.click(canvasBox.x + 295, canvasBox.y + 165);
    const nameInput = page.locator('[data-testid="inspector-table-name"]');
    await nameInput.waitFor();
    assert.equal(await nameInput.inputValue(), "table_1", "presence 存在时本地选中表必须生效");
  });

  // ─── Viewer 只读：写入口禁用、无 op、ot-rev 不递增 ──────────────────────
  await run(["ST-S05-UI-06"], "Viewer 写入口禁用且 ot-rev 不递增", async page => {
    const state = await installApi(page, {
      roomsList: [{
        id: "room-view", name: "只读评审室", diagramId: "diagram-new", diagramTitle: "只读评审室",
        myRole: "viewer", memberCount: 2, updatedAt: "2026-08-23T00:02:00Z",
      }],
    });
    await login(page);
    await page.locator('[data-testid="room-card-room-view"]:visible').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
    await page.locator('[data-testid="ws-status"]').getByText("已连接 · OT 同步").waitFor();

    assert.equal(await page.locator('[data-testid="tool-add-table"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="tool-relationship"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="btn-invite"]').isDisabled(), true);

    // 强制触发不会入队 op，也不触发快照 PUT
    await page.locator('[data-testid="tool-add-table"]').click({ force: true }).catch(() => {});
    await page.waitForTimeout(1_600);
    assert.equal(await page.locator('[data-testid="status-pending-ops"]').count(), 0, "Viewer 禁止产生待同步 op");
    assert.equal(state.putCalls, 0, "Viewer 禁止触发快照 PUT");
    await page.locator('[data-testid="ot-rev"]').getByText("server_rev 7").waitFor();
  });
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
