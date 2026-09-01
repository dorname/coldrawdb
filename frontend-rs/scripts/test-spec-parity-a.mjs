import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { reportOpenLogos } from "../tests/e2e/helpers/openlogos-reporter.mjs";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const FRONTEND_URL = process.env.SPEC_PARITY_FRONTEND_URL || "http://127.0.0.1:4173";
const BACKEND_URL = "http://127.0.0.1:3000";
const playwrightBrowsers = applyPlaywrightBrowserEnv();
const { chromium } = await import("playwright");

const diagram = {
  code: 0,
  request_id: "spec-parity-a",
  data: {
    id: "public-diagram",
    name: "公开数据模型",
    database: "postgresql",
    revision: 3,
    tables: [
      {
        id: "users",
        name: "users",
        x: 120,
        y: 100,
        color: "#175e7a",
        comment: "",
        fields: [
          {
            id: "users-id",
            name: "id",
            type_: "INT",
            primary: true,
            unique: true,
            not_null: true,
            increment: true,
          },
        ],
        indices: [],
      },
    ],
    references: [],
    areas: [],
    notes: [],
  },
};

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
      const result = await fetch(url);
      if (result.ok) return;
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
  return spawn("trunk", ["serve", "--port", "4173"], {
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

async function installApi(page, overrides = {}) {
  const state = {
    requests: [],
    roomsCalls: 0,
    refreshCalls: 0,
    ...overrides.state,
  };
  await page.route(`${BACKEND_URL}/api/v1/**`, async route => {
    const request = route.request();
    const url = new URL(request.url());
    state.requests.push(`${request.method()} ${url.pathname}`);
    if (request.method() === "OPTIONS") return response(route, 204);

    if (url.pathname === "/api/v1/auth/login") {
      return response(route, 200, {
        accessToken: "access-login",
        expiresIn: 900,
        tokenType: "Bearer",
      });
    }
    if (url.pathname === "/api/v1/auth/me") {
      return response(route, 200, {
        id: "user-1",
        email: "reviewer@example.com",
        displayName: "评审者",
      });
    }
    if (url.pathname === "/api/v1/auth/register") {
      return response(route, 409, {
        code: "EMAIL_EXISTS",
        message: "reviewer@example.com 已注册；token=server-secret",
      });
    }
    if (url.pathname === "/api/v1/auth/refresh") {
      state.refreshCalls += 1;
      return response(route, overrides.refreshStatus ?? 200, {
        accessToken: "access-refreshed",
        expiresIn: 900,
        tokenType: "Bearer",
      });
    }
    if (url.pathname === "/api/v1/rooms") {
      state.roomsCalls += 1;
      if (overrides.firstRoomsUnauthorized && state.roomsCalls === 1) {
        return response(route, 401, { code: "token_expired", message: "expired" });
      }
      return response(route, 200, { items: [], total: 0 });
    }
    if (url.pathname === "/api/v1/diagrams/public-diagram") {
      return response(route, overrides.diagramStatus ?? 200, overrides.diagramStatus === 404
        ? { code: "NOT_FOUND", message: "sqlite private-room-name token=secret" }
        : diagram);
    }
    return response(route, 404, { code: "NOT_FOUND" });
  });
  return state;
}

async function fillLogin(page) {
  await page.locator('[data-testid="auth-email"]').fill("reviewer@example.com");
  await page.locator('[data-testid="auth-password"]').fill("Pass1234!");
  await page.locator('[data-testid="login-submit"]').click();
}

let frontend = null;
let browser;
let failed = false;

async function run(ids, title, body) {
  const page = await browser.newPage();
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

  await run(["ST-S03-UI-01", "ST-S02-NO-SHARE", "ST-FE-ALIGN-01"], "默认入口只显示鉴权页", async page => {
    const state = await installApi(page);
    await page.goto(FRONTEND_URL);
    await page.locator('[data-testid="auth-gate"]').waitFor();
    await assert.doesNotReject(page.locator('[data-testid="login-form"]').waitFor());
    await assert.doesNotReject(page.locator('[data-testid="auth-tab-register"]').waitFor());
    assert.equal(await page.locator('[data-testid="rooms-list-page"]:visible').count(), 0);
    assert.equal(state.requests.some(item => item.includes("/rooms")), false);
    const visual = await page.locator('[data-testid="auth-gate"]').evaluate(element => {
      const pageStyle = getComputedStyle(element);
      const storyStyle = getComputedStyle(element.querySelector('[data-testid="auth-story"]'));
      const panelStyle = getComputedStyle(element.querySelector('[data-testid="auth-panel"]'));
      return {
        padding: pageStyle.paddingTop,
        gap: pageStyle.columnGap,
        storyRadius: storyStyle.borderRadius,
        panelRadius: panelStyle.borderRadius,
        panelBackdrop: panelStyle.backdropFilter,
      };
    });
    assert.deepEqual(visual, {
      padding: "18px",
      gap: "18px",
      storyRadius: "28px",
      panelRadius: "28px",
      panelBackdrop: "blur(22px) saturate(1.45)",
    });
  });

  await run(["ST-S03-UI-02"], "登录错误不枚举用户", async page => {
    await installApi(page);
    await page.goto(FRONTEND_URL);
    await page.locator('[data-testid="auth-simulate-error"]').click();
    await fillLogin(page);
    const message = await page.locator('[data-testid="auth-alert"]').textContent();
    assert.match(message, /凭据错误/);
    assert.doesNotMatch(message, /reviewer@example\.com|token|secret/i);
  });

  await run(["ST-S03-UI-03"], "重复邮箱显示脱敏字段错误", async page => {
    await installApi(page);
    await page.goto(FRONTEND_URL);
    await page.locator('[data-testid="auth-tab-register"]').click();
    await page.locator('[data-testid="auth-display-name"]').fill("评审者");
    await page.locator('[data-testid="auth-email"]').fill("reviewer@example.com");
    await page.locator('[data-testid="auth-password"]').fill("Pass1234!");
    await page.locator('[data-testid="auth-confirm-password"]').fill("Pass1234!");
    await page.locator('[data-testid="register-submit"]').click();
    // 同步等待 signal 推送 + DOM 提交完成,避免 race condition 拿到空串。
    await page.locator('[data-testid="auth-email-error"]').evaluate(
      (element) => new Promise((resolve, reject) => {
        const deadline = Date.now() + 5000;
        const tick = () => {
          if (element.textContent && element.textContent.length > 0) resolve(null);
          else if (Date.now() > deadline) reject(new Error("auth-email-error 等待超时(5000ms)"));
          else requestAnimationFrame(tick);
        };
        tick();
      }),
    );
    const message = await page.locator('[data-testid="auth-email-error"]').textContent();
    assert.match(message, /无法创建账户/);
    assert.doesNotMatch(message, /reviewer@example\.com|token|secret/i);
    assert.equal(await page.locator('[data-testid="register-form"]').count(), 1);
  });

  await run(["ST-S03-UI-04", "ST-FE-ALIGN-02"], "登录成功进入房间页", async page => {
    await installApi(page);
    await page.goto(FRONTEND_URL);
    await fillLogin(page);
    await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
    await assert.doesNotReject(page.locator('[data-testid="session-indicator"]:visible').waitFor());
    await assert.doesNotReject(page.locator('[data-testid="user-menu"]:visible').waitFor());
    assert.equal(await page.locator('[data-testid="editor-ready"]:visible').count(), 0);
    const visual = await page.locator('[data-testid="rooms-list-page"]:visible').evaluate(element => {
      const pageStyle = getComputedStyle(element);
      const navStyle = getComputedStyle(element.querySelector(".cdb-rooms-topbar"));
      return {
        padding: pageStyle.paddingTop,
        navHeight: navStyle.height,
        navRadius: navStyle.borderRadius,
      };
    });
    assert.deepEqual(visual, { padding: "18px", navHeight: "64px", navRadius: "18px" });
  });

  await run(["ST-S03-UI-05"], "401 仅续期一次并重放房间请求", async page => {
    const state = await installApi(page, { firstRoomsUnauthorized: true });
    await page.goto(FRONTEND_URL);
    await fillLogin(page);
    await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
    await page.locator('[data-testid="rooms-empty"]:visible').waitFor();
    assert.equal(state.refreshCalls, 1);
    assert.equal(state.roomsCalls, 2);
    await assert.doesNotReject(
      page.locator('[data-testid="rooms-list-page"]:visible [data-testid="session-indicator"]').getByText("会话已续期").waitFor(),
    );

    const expiredPage = await browser.newPage();
    try {
      const expiredState = await installApi(expiredPage, {
        firstRoomsUnauthorized: true,
        refreshStatus: 401,
      });
      await expiredPage.goto(FRONTEND_URL);
      await fillLogin(expiredPage);
      await expiredPage.locator('[data-testid="auth-gate"]:visible').waitFor();
      await expiredPage.locator('[data-testid="auth-session-notice"]').getByText("登录已过期，请重新登录").waitFor();
      assert.equal(expiredState.refreshCalls, 1);
      assert.equal(expiredState.roomsCalls, 1);
    } finally {
      await expiredPage.close();
    }
  });

  await run(["ST-S02-SHARE-RO"], "匿名分享加载且写入口禁用", async page => {
    const state = await installApi(page);
    await page.goto(`${FRONTEND_URL}/editor/private-id?share=public-diagram`);
    await page.locator('[data-testid="share-readonly"]').waitFor();
    assert.equal(await page.locator('[data-testid="auth-gate"]:visible').count(), 0);
    assert.equal(await page.locator('[data-testid="tool-add-table"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="tool-relationship"]').isDisabled(), true);
    assert.equal(await page.locator('[data-testid="diagram-title"]').isEditable(), false);
    assert.equal(state.requests.some(item => /POST|PUT|PATCH|DELETE/.test(item) && item.includes("/diagrams")), false);
  });

  await run(["ST-S02-SHARE-VS-AUTH"], "已完成登录后打开分享仍保持只读", async page => {
    await installApi(page);
    await page.goto(FRONTEND_URL);
    await fillLogin(page);
    await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
    await page.goto(`${FRONTEND_URL}/?share=public-diagram`);
    await page.locator('[data-testid="share-readonly"]').waitFor();
    assert.equal(await page.locator('[data-testid="tool-add-table"]').isDisabled(), true);
  });

  await run(["ST-S02-404"], "失效分享显示公开 404 页面", async page => {
    await installApi(page, { diagramStatus: 404 });
    await page.goto(`${FRONTEND_URL}/?share=public-diagram`);
    const statePage = page.locator('[data-testid="share-not-found"]:visible');
    await statePage.waitFor();
    const message = await statePage.textContent();
    assert.match(message, /分享链接不存在或已失效/);
    assert.doesNotMatch(message, /sqlite|private-room-name|token|secret/i);
    assert.equal(await page.locator('[data-testid="editor-ready"]:visible').count(), 0);
  });
} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
