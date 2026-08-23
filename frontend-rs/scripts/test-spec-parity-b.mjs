import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const FRONTEND_URL = process.env.SPEC_PARITY_FRONTEND_URL || "http://127.0.0.1:4174";
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
  return spawn("trunk", ["serve", "--port", "4174"], {
    cwd: new URL("..", import.meta.url),
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

async function installApi(page, options = {}) {
  const state = {
    requests: [],
    diagramCreates: [],
    roomCreates: [],
    inviteCreates: [],
    rolePatches: [],
    memberDeletes: [],
    members: options.members ?? [
      { userId: "owner-1", email: "owner@example.com", displayName: "林默", role: "owner", joinedAt: "2026-08-23T00:00:00Z" },
      { userId: "member-2", email: "chen@example.com", displayName: "陈晨", role: "editor", joinedAt: "2026-08-23T00:01:00Z" },
    ],
    inviteExpired: options.inviteExpired ?? false,
  };

  await page.route(`${BACKEND_URL}/diagrams/queryAll`, route => response(route, 200, {
    code: 0, message: "success", data: [{ id: "diagram-existing", name: "已有订单模型" }],
  }));

  await page.route(`${BACKEND_URL}/api/v1/**`, async route => {
    const request = route.request();
    const url = new URL(request.url());
    state.requests.push(`${request.method()} ${url.pathname}`);
    if (request.method() === "OPTIONS") return response(route, 204);

    const auth = request.headers()["authorization"] ?? "";
    const isGuest = auth.includes("access-guest") || auth.includes("access-viewer");

    if (url.pathname === "/api/v1/auth/login") {
      const body = request.postDataJSON();
      const guest = body?.email === "guest@example.com";
      const viewer = body?.email === "viewer@example.com";
      return response(route, 200, {
        accessToken: guest ? "access-guest" : viewer ? "access-viewer" : "access-owner", expiresIn: 900, tokenType: "Bearer",
      });
    }
    if (url.pathname === "/api/v1/auth/me") {
      if (options.selfUser) return response(route, 200, options.selfUser);
      return response(route, 200, isGuest
        ? { id: "guest-1", email: "guest@example.com", displayName: "沈青" }
        : { id: "owner-1", email: "owner@example.com", displayName: "林默" });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "GET") {
      const items = options.roomsList ?? [];
      return response(route, 200, { items, total: items.length });
    }
    if (url.pathname === "/api/v1/diagrams" && request.method() === "POST") {
      state.diagramCreates.push(request.postDataJSON());
      return response(route, 200, { code: 0, request_id: "create-diagram", data: { id: "diagram-new" } });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "POST") {
      const body = request.postDataJSON();
      state.roomCreates.push(body);
      return response(route, 201, {
        id: "room-new",
        name: body.name,
        diagramId: body.diagramId,
        ownerId: "owner-1",
      });
    }
    // 邀请：创建 / 预览 / 接受
    if (url.pathname === "/api/v1/rooms/room-new/invites" && request.method() === "POST") {
      const body = request.postDataJSON();
      state.inviteCreates.push(body);
      return response(route, 201, {
        inviteUrl: `${FRONTEND_URL}/invite/inv-${body.role}`,
        token: `inv-${body.role}`,
        role: body.role,
        expiresAt: "2026-08-30T00:00:00Z",
      });
    }
    if (url.pathname.startsWith("/api/v1/rooms/invites/") && url.pathname.endsWith("/accept")) {
      const token = url.pathname.split("/")[4];
      return response(route, 200, {
        roomId: "room-new", diagramId: "diagram-new",
        role: token === "inv-viewer" ? "viewer" : "editor", alreadyMember: false,
      });
    }
    if (url.pathname.startsWith("/api/v1/rooms/invites/")) {
      if (state.inviteExpired || url.pathname.endsWith("/inv-expired")) {
        return response(route, 404, { code: "INVITE_NOT_FOUND", message: "invite expired" });
      }
      const viewer = url.pathname.endsWith("/inv-viewer");
      return response(route, 200, {
        roomName: "架构评审室", diagramTitle: "电商核心模型", diagramId: "diagram-new",
        role: viewer ? "viewer" : "editor", invitedBy: "林默", expiresAt: "2026-08-30T00:00:00Z",
      });
    }
    // 成员：列表 / 改角色 / 移除
    if (/^\/api\/v1\/rooms\/[^/]+\/members$/.test(url.pathname) && request.method() === "GET") {
      return response(route, 200, { items: state.members });
    }
    if (/^\/api\/v1\/rooms\/[^/]+\/members\/[^/]+$/.test(url.pathname)) {
      const uid = url.pathname.split("/").pop();
      if (request.method() === "PATCH") {
        const body = request.postDataJSON();
        state.rolePatches.push({ userId: uid, role: body.role });
        const member = state.members.find(m => m.userId === uid);
        if (member) member.role = body.role;
        return response(route, 200, { ...member, role: body.role });
      }
      if (request.method() === "DELETE") {
        state.memberDeletes.push(uid);
        state.members = state.members.filter(m => m.userId !== uid);
        return response(route, 204);
      }
    }
    if (url.pathname === "/api/v1/rooms/room-new") {
      return response(route, 200, {
        id: "room-new",
        name: "架构评审室",
        diagramId: "diagram-new",
        ownerId: "owner-1",
        diagramTitle: "架构评审室",
        myRole: isGuest ? "editor" : "owner",
        memberCount: state.members.length,
      });
    }
    if (options.roomDetail && url.pathname === `/api/v1/rooms/${options.roomDetail.id}`) {
      const detail = options.roomDetail;
      return response(route, 200, {
        ownerId: "owner-1",
        diagramTitle: detail.name,
        memberCount: state.members.length,
        ...detail,
      });
    }
    if (url.pathname.startsWith("/api/v1/diagrams/") && request.method() === "GET") {
      const diagramId = url.pathname.split("/").pop();
      return response(route, 200, {
        code: 0,
        request_id: "load-diagram",
        data: {
          id: diagramId,
          name: "架构评审室",
          database: null,
          revision: 0,
          tables: [],
          references: [],
          areas: [],
          notes: [],
        },
      });
    }
    if (url.pathname.endsWith("/collab/head")) {
      const roomId = url.pathname.split("/")[4];
      const diagramId = roomId === "room-view" ? "diagram-view" : "diagram-new";
      return response(route, 200, { roomId, diagramId, serverRev: 0 });
    }
    return response(route, 404, { code: "NOT_FOUND" });
  });
  return state;
}

async function login(page, email = "owner@example.com") {
  await page.goto(FRONTEND_URL);
  await page.locator('[data-testid="auth-email"]').fill(email);
  await page.locator('[data-testid="auth-password"]').fill("Pass1234!");
  await page.locator('[data-testid="login-submit"]').click();
  await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
}

const frontend = startFrontend();
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
  await waitForServer(FRONTEND_URL);
  browser = await chromium.launch({
    headless: true,
    executablePath: playwrightBrowsers.headless ?? playwrightBrowsers.chrome,
  });

  await run(["ST-S04-UI-01"], "房间首屏与创建入口贴合主原型", async page => {
    await installApi(page);
    await login(page);
    const roomsPage = page.locator('[data-testid="rooms-list-page"]:visible');
    await roomsPage.locator('[data-testid="rooms-empty"]').waitFor();
    await roomsPage.locator('[data-testid="btn-create-room"]').waitFor();
    await roomsPage.locator('[data-testid="btn-refresh-rooms"]').waitFor();
    await roomsPage.locator('[data-testid="user-menu"]').waitFor();
    const cardStyle = await roomsPage.locator(".cdb-room-card--new").evaluate(element => {
      const style = getComputedStyle(element);
      return { minHeight: style.minHeight, radius: style.borderRadius };
    });
    assert.deepEqual(cardStyle, { minHeight: "215px", radius: "20px" });
  });

  await run(["ST-S04-UI-02"], "创建 diagram 后使用真实 ID 创建房间", async page => {
    const state = await installApi(page);
    await login(page);
    await page.locator('[data-testid="btn-create-room"]').click();
    await page.locator('[data-testid="modal-create-room"]').waitFor();
    await page.locator('[data-testid="create-room-name"]').fill("架构评审室");
    await page.locator('[data-testid="create-room-submit"]').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
    await page.locator('[data-testid="room-badge"]').getByText("架构评审室").waitFor();

    assert.deepEqual(state.diagramCreates, [{ name: "架构评审室", database: null }]);
    assert.deepEqual(state.roomCreates, [{ name: "架构评审室", diagramId: "diagram-new" }]);
    assert.notEqual(state.roomCreates[0].diagramId, "default");
    assert.ok(state.requests.indexOf("POST /api/v1/diagrams") < state.requests.indexOf("POST /api/v1/rooms"));
  });

  // ─── ST-S04-UI-03：Owner 生成邀请 → 显示真实 invite URL；角色切换重新生成 ───
  await run(["ST-S04-UI-03"], "邀请模态生成真实邀请链接", async page => {
    const state = await installApi(page);
    await login(page);
    await page.locator('[data-testid="btn-create-room"]').click();
    await page.locator('[data-testid="create-room-name"]').fill("架构评审室");
    await page.locator('[data-testid="create-room-submit"]').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    await page.locator('[data-testid="btn-invite"]').click();
    const modal = page.locator('[data-testid="modal-invite"]:visible');
    await modal.waitFor();
    await modal.getByText("邀请成员加入「架构评审室」").waitFor();
    // 打开即按默认角色 editor 生成真实邀请
    const urlInput = modal.locator('[data-testid="invite-url"]');
    await urlInput.waitFor();
    assert.match(await urlInput.inputValue(), /\/invite\/inv-editor/);
    assert.deepEqual(state.inviteCreates, [{ role: "editor" }]);

    // 切换角色 → 重新生成（preview/accept 链路依赖该 URL）
    await modal.locator('[data-testid="invite-role"]').selectOption("viewer");
    await page.waitForFunction(
      () => document.querySelector('[data-testid="invite-url"]')?.value.includes("inv-viewer"),
      null,
      { timeout: 5_000 },
    );
    assert.deepEqual(state.inviteCreates.map(c => c.role), ["editor", "viewer"]);

    // 复制按钮可用并反馈
    await modal.locator('[data-testid="btn-copy-invite"]').click();
    await modal.locator('[data-testid="btn-copy-invite"]').getByText("已复制").waitFor();
  });

  // ─── ST-S04-UI-04：另一用户接受邀请 → 进入同一 room-editor（含匿名续接） ───
  await run(["ST-S04-UI-04"], "受邀用户经登录续接加入同一房间", async page => {
    // page 作为 Owner 生成邀请（仅取链接）；guestPage 走完整接受链路
    const ownerState = await installApi(page);
    await login(page);
    await page.locator('[data-testid="btn-create-room"]').click();
    await page.locator('[data-testid="create-room-name"]').fill("架构评审室");
    await page.locator('[data-testid="create-room-submit"]').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();
    await page.locator('[data-testid="btn-invite"]').click();
    const inviteUrl = await page.locator('[data-testid="modal-invite"] [data-testid="invite-url"]').inputValue();
    assert.ok(ownerState.inviteCreates.length >= 1);
    const invitePath = new URL(inviteUrl).pathname;

    const guestPage = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    try {
      await installApi(guestPage);
      // 匿名打开邀请 → preview 可见 → 加入提示先登录
      await guestPage.goto(`${FRONTEND_URL}${invitePath}`);
      await guestPage.locator('[data-testid="invite-accept-page"]:visible').waitFor();
      await guestPage.locator('[data-testid="invite-title"]').getByText("加入架构评审室").waitFor();
      await guestPage.locator('[data-testid="invite-meta"]').getByText("邀请人：林默 · 分配角色：editor").waitFor();
      await guestPage.locator('[data-testid="invite-preview"]').getByText("电商核心模型").waitFor();
      await guestPage.locator('[data-testid="btn-accept-invite"]').click();
      await guestPage.locator('[data-testid="invite-login-required"]:visible').waitFor();

      // 切换登录 → 登录成功后续接回邀请页
      await guestPage.locator('[data-testid="btn-invite-goto-login"]').click();
      await guestPage.locator('[data-testid="auth-gate"]:visible').waitFor();
      await guestPage.locator('[data-testid="auth-email"]').fill("guest@example.com");
      await guestPage.locator('[data-testid="auth-password"]').fill("Pass1234!");
      await guestPage.locator('[data-testid="login-submit"]').click();
      await guestPage.locator('[data-testid="invite-accept-page"]:visible').waitFor();

      // 接受 → 进入同一 room-editor（room-badge 同名）
      await guestPage.locator('[data-testid="btn-accept-invite"]').click();
      await guestPage.locator('[data-testid="room-editor-page"]:visible').waitFor();
      await guestPage.locator('[data-testid="room-badge"]').getByText("架构评审室").waitFor();
      await guestPage.locator('[data-testid="status-role"]').getByText("editor").waitFor();

      // room-badge 点击可回 rooms 列表（§7.2 checklist）
      await guestPage.locator('[data-testid="room-badge"]').click();
      await guestPage.locator('[data-testid="rooms-list-page"]:visible').waitFor();
    } finally {
      await guestPage.close();
    }
  });

  // ─── ST-S04-UI-05：成员面板改角色 / 移除，列表即时更新 ───────────────────
  await run(["ST-S04-UI-05"], "Owner 管理成员角色与移除", async page => {
    const state = await installApi(page);
    await login(page);
    await page.locator('[data-testid="btn-create-room"]').click();
    await page.locator('[data-testid="create-room-name"]').fill("架构评审室");
    await page.locator('[data-testid="create-room-submit"]').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    await page.locator('[data-testid="btn-members"]').click();
    const panel = page.locator('[data-testid="room-members-panel"]:visible');
    await panel.waitFor();
    // 自动加载成员（无手动加载按钮）
    await panel.locator('[data-testid="room-member-owner-1"]').waitFor();
    await panel.locator('[data-testid="room-member-member-2"]').waitFor();
    await panel.locator('[data-testid="room-members-count"]').getByText("2 位成员").waitFor();
    // 本人行：角色 tag + （你）；非本人行：角色 select + 移除按钮
    await panel.locator('[data-testid="room-member-owner-1"]').getByText("林默（你）").waitFor();
    assert.equal(await panel.locator('[data-testid="member-role-owner-1"]').count(), 0);

    // 改角色 editor → viewer：PATCH + 列表即时更新
    await panel.locator('[data-testid="member-role-member-2"]').selectOption("viewer");
    await page.waitForFunction(
      () => document.querySelector('[data-testid="member-role-member-2"]')?.value === "viewer",
      null,
      { timeout: 5_000 },
    );
    assert.deepEqual(state.rolePatches, [{ userId: "member-2", role: "viewer" }]);

    // 移除：DELETE + 行消失 + 计数更新
    await panel.locator('[data-testid="btn-remove-member-member-2"]').click();
    await panel.locator('[data-testid="room-members-count"]').getByText("1 位成员").waitFor();
    assert.deepEqual(state.memberDeletes, ["member-2"]);
    assert.equal(await panel.locator('[data-testid="room-member-member-2"]').count(), 0);
  });

  // ─── ST-S04-UI-06：Viewer 写操作禁用 + 只读提示 ─────────────────────────
  await run(["ST-S04-UI-06"], "Viewer 写入口全部禁用并有只读标识", async page => {
    await installApi(page, {
      selfUser: { id: "viewer-1", email: "viewer@example.com", displayName: "实习生" },
      roomsList: [{
        id: "room-view", name: "只读评审室", diagramId: "diagram-view", diagramTitle: "只读评审室",
        myRole: "viewer", memberCount: 2, updatedAt: "2026-08-23T00:02:00Z",
      }],
      members: [
        { userId: "viewer-1", email: "viewer@example.com", displayName: "实习生", role: "viewer", joinedAt: "2026-08-23T00:00:00Z" },
        { userId: "owner-1", email: "owner@example.com", displayName: "林默", role: "owner", joinedAt: "2026-08-22T00:00:00Z" },
      ],
      roomDetail: { id: "room-view", name: "只读评审室", diagramId: "diagram-view", myRole: "viewer" },
    });
    await login(page, "viewer@example.com");
    await page.locator('[data-testid="room-card-room-view"]:visible').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    // 写操作禁用：新建表 / 关系 / 邀请
    assert.equal(await page.locator('[data-testid="tool-add-table"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="tool-relationship"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="btn-invite"]').isDisabled(), true);
    // 只读提示：状态栏角色标识 viewer
    await page.locator('[data-testid="status-role"]').getByText("viewer").waitFor();
    // 成员抽屉：非 owner 时邀请入口与他人角色管理禁用
    await page.locator('[data-testid="btn-members"]').click();
    const panel = page.locator('[data-testid="room-members-panel"]:visible');
    await panel.waitFor();
    await panel.locator('[data-testid="room-member-owner-1"]').waitFor();
    assert.equal(await panel.locator('[data-testid="btn-open-invite"]').isDisabled(), true);
    assert.equal(await panel.locator('[data-testid="member-role-owner-1"]').isDisabled(), true);
    assert.equal(await panel.locator('[data-testid="btn-remove-member-owner-1"]').isDisabled(), true);
  });

  // ─── ST-S04-UI-07 / ST-PU-23：邀请过期 → 失效页，无加入按钮 ─────────────
  await run(["ST-S04-UI-07", "ST-PU-23"], "过期邀请显示失效页且无加入按钮", async page => {
    await installApi(page, { inviteExpired: true });
    await page.goto(`${FRONTEND_URL}/invite/inv-expired`);
    const invitePage = page.locator('[data-testid="invite-accept-page"]:visible');
    await invitePage.waitFor();
    await invitePage.locator('[data-testid="invite-title"]').getByText("邀请已失效").waitFor();
    await invitePage.getByText("为了保护房间安全，此邀请不再可用。").waitFor();
    await invitePage.getByText("邀请链接已超过 7 天，请联系房间 Owner 重新生成。").waitFor();
    assert.equal(await invitePage.locator('[data-testid="btn-accept-invite"]').count(), 0, "失效邀请禁止出现加入按钮");
    await invitePage.locator('[data-testid="btn-invite-back"]').waitFor();
    // 失效页不泄露后端错误原文
    const bodyText = await invitePage.textContent();
    assert.doesNotMatch(bodyText ?? "", /INVITE_NOT_FOUND|404/);
  });
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
