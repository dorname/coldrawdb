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

async function installApi(page) {
  const state = { requests: [], diagramCreates: [], roomCreates: [] };

  await page.route(`${BACKEND_URL}/diagrams/queryAll`, route => response(route, 200, {
    code: 0,
    message: "success",
    data: [{ id: "diagram-existing", name: "已有订单模型" }],
  }));

  await page.route(`${BACKEND_URL}/api/v1/**`, async route => {
    const request = route.request();
    const url = new URL(request.url());
    state.requests.push(`${request.method()} ${url.pathname}`);
    if (request.method() === "OPTIONS") return response(route, 204);

    if (url.pathname === "/api/v1/auth/login") {
      return response(route, 200, { accessToken: "access-owner", expiresIn: 900, tokenType: "Bearer" });
    }
    if (url.pathname === "/api/v1/auth/me") {
      return response(route, 200, { id: "owner-1", email: "owner@example.com", displayName: "林默" });
    }
    if (url.pathname === "/api/v1/rooms" && request.method() === "GET") {
      return response(route, 200, { items: [], total: 0 });
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
    if (url.pathname === "/api/v1/rooms/room-new") {
      return response(route, 200, {
        id: "room-new",
        name: "架构评审室",
        diagramId: "diagram-new",
        ownerId: "owner-1",
        diagramTitle: "架构评审室",
        myRole: "owner",
        memberCount: 1,
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new") {
      return response(route, 200, {
        code: 0,
        request_id: "load-diagram",
        data: {
          id: "diagram-new",
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
      return response(route, 200, { serverRev: 0, snapshotRev: 0 });
    }
    if (url.pathname.endsWith("/members")) {
      return response(route, 200, { items: [] });
    }
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
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
