/**
 * ST-FE-S05-07~09（wire-frontend-collab-ws）双上下文真实 WS 协作测试的公共辅助。
 *
 * 走真实后端 REST 完成 setup（注册/登录/建房/邀请/接受），再通过 localStorage 注入
 * AuthSession 进入生产前端，保证 WS 链路完全真实（禁止 mock WS）。
 *
 * 调试钩子（仅 debug 构建暴露，见 frontend-rs/src/lib.rs 与 collab_client.rs）：
 * - `__cdb_debug_add_table(name)`：注入表（真实 store mutation → diff watcher → op 上行）
 * - `__cdb_debug_state()`：画布实体状态 JSON（canvas 绘点无法 DOM 断言）
 * - `__cdb_collab_state()`：协作状态 JSON（connection/serverRev/queueLen/…）
 * - `__cdb_remote_presence_map`：远端 presence userId→坐标 JSON
 */

import type { APIRequestContext, Browser, BrowserContext, Page } from "@playwright/test";

export const API_BASE = process.env.E2E_API_BASE ?? "http://127.0.0.1:3000";

export interface CollabUser {
  email: string;
  accessToken: string;
  expiresIn: number;
  userId: string;
  displayName: string;
}

function uniq(label: string): string {
  return `${label}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export async function registerAndLogin(
  request: APIRequestContext,
  label: string,
): Promise<CollabUser> {
  const email = `e2e-${uniq(label)}@example.com`;
  const password = "E2e-passw0rd!";
  const reg = await request.post(`${API_BASE}/api/v1/auth/register`, {
    data: { email, password, displayName: label },
  });
  if (reg.status() !== 201) {
    throw new Error(`register failed: ${reg.status()} ${await reg.text()}`);
  }
  const login = await request.post(`${API_BASE}/api/v1/auth/login`, {
    data: { email, password, rememberDevice: true },
  });
  if (!login.ok()) {
    throw new Error(`login failed: ${login.status()} ${await login.text()}`);
  }
  // 后端 TokenResponse 为 camelCase（accessToken/expiresIn/tokenType）。
  const token = (await login.json()) as { accessToken: string; expiresIn: number };
  const me = await request.get(`${API_BASE}/api/v1/auth/me`, {
    headers: { Authorization: `Bearer ${token.accessToken}` },
  });
  if (!me.ok()) {
    throw new Error(`me failed: ${me.status()} ${await me.text()}`);
  }
  const profile = (await me.json()) as { id: string; displayName?: string };
  return {
    email,
    accessToken: token.accessToken,
    expiresIn: token.expiresIn,
    userId: profile.id,
    displayName: profile.displayName ?? label,
  };
}

export interface RoomFixture {
  roomId: string;
  diagramId: string;
}

export async function createRoomWithDiagram(
  request: APIRequestContext,
  owner: CollabUser,
  name: string,
): Promise<RoomFixture> {
  const d = await request.post(`${API_BASE}/api/v1/diagrams`, { data: { name } });
  if (!d.ok()) {
    throw new Error(`create diagram failed: ${d.status()} ${await d.text()}`);
  }
  const dj = (await d.json()) as { data: { id: string } };
  const diagramId = dj.data.id;
  const r = await request.post(`${API_BASE}/api/v1/rooms`, {
    headers: { Authorization: `Bearer ${owner.accessToken}` },
    data: { name, diagramId },
  });
  if (r.status() !== 201) {
    throw new Error(`create room failed: ${r.status()} ${await r.text()}`);
  }
  const room = (await r.json()) as { id: string };
  return { roomId: room.id, diagramId };
}

export async function inviteAndAccept(
  request: APIRequestContext,
  owner: CollabUser,
  member: CollabUser,
  roomId: string,
  role: "editor" | "viewer" = "editor",
): Promise<void> {
  const inv = await request.post(`${API_BASE}/api/v1/rooms/${roomId}/invites`, {
    headers: { Authorization: `Bearer ${owner.accessToken}` },
    data: { role },
  });
  if (inv.status() !== 201) {
    throw new Error(`create invite failed: ${inv.status()} ${await inv.text()}`);
  }
  const { token } = (await inv.json()) as { token: string };
  const acc = await request.post(`${API_BASE}/api/v1/rooms/invites/${token}/accept`, {
    headers: { Authorization: `Bearer ${member.accessToken}` },
  });
  if (!acc.ok()) {
    throw new Error(`accept invite failed: ${acc.status()} ${await acc.text()}`);
  }
}

export interface RoomPage {
  context: BrowserContext;
  page: Page;
}

/** 注入 AuthSession → 进入房间列表 → 点击进入房间编辑器。 */
export async function newRoomPage(
  browser: Browser,
  user: CollabUser,
  roomId: string,
): Promise<RoomPage> {
  const context = await browser.newContext();
  const session = JSON.stringify({
    access_token: user.accessToken,
    expires_in: user.expiresIn,
    token_type: "Bearer",
    user: null,
  });
  await context.addInitScript(([s]) => {
    window.localStorage.setItem("coldrawdb.auth_session.v1", s as string);
  }, [session]);
  const page = await context.newPage();
  await page.goto("/");
  await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
  const item = page.locator(`[data-testid="room-list-item-${roomId}"]`);
  if (!(await item.isVisible().catch(() => false))) {
    await page.click('[data-testid="btn-refresh-rooms"]');
  }
  await item.waitFor({ state: "visible", timeout: 10_000 });
  await item.click();
  await page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
  return { context, page };
}

/** 等待 WS 握手完成（connected 帧到达 → connection=Connected）。 */
export async function waitCollabConnected(page: Page, timeout = 20_000): Promise<void> {
  await page.waitForFunction(
    () => {
      const fn = (window as unknown as Record<string, unknown>).__cdb_collab_state;
      if (typeof fn !== "function") return false;
      try {
        return JSON.parse((fn as () => string)()).connection === "Connected";
      } catch {
        return false;
      }
    },
    undefined,
    { timeout },
  );
}

export interface DebugState {
  tables: string[];
  revision: number;
  dirty: boolean;
}

export interface CollabState {
  connection: string;
  serverRev: number;
  queueLen: number;
  roomId: string;
  userId: string | null;
  attempts: number;
}

export async function debugState(page: Page): Promise<DebugState> {
  const raw = await page.evaluate(() =>
    (window as unknown as Record<string, () => string>).__cdb_debug_state(),
  );
  return JSON.parse(raw) as DebugState;
}

export async function collabState(page: Page): Promise<CollabState> {
  const raw = await page.evaluate(() =>
    (window as unknown as Record<string, () => string>).__cdb_collab_state(),
  );
  return JSON.parse(raw) as CollabState;
}

export async function remotePresenceMap(page: Page): Promise<Record<string, { x: number; y: number }>> {
  const raw = await page.evaluate(
    () =>
      ((window as unknown as Record<string, unknown>).__cdb_remote_presence_map as string) ?? "{}",
  );
  return JSON.parse(raw) as Record<string, { x: number; y: number }>;
}

export async function addTable(page: Page, name: string): Promise<string> {
  return page.evaluate(
    (n) => (window as unknown as Record<string, (n: string) => string>).__cdb_debug_add_table(n),
    name,
  );
}

/** 双上下文房间 setup：注册 owner/member、建房、邀请接受、两端进入房间并等待 WS Connected。 */
export async function setupCollabPair(
  browser: Browser,
  request: APIRequestContext,
  label: string,
  memberRole: "editor" | "viewer" = "editor",
): Promise<{ a: RoomPage; b: RoomPage; owner: CollabUser; member: CollabUser; roomId: string }> {
  const owner = await registerAndLogin(request, `${label}-a`);
  const member = await registerAndLogin(request, `${label}-b`);
  const { roomId } = await createRoomWithDiagram(request, owner, `${label}-room`);
  await inviteAndAccept(request, owner, member, roomId, memberRole);
  const a = await newRoomPage(browser, owner, roomId);
  const b = await newRoomPage(browser, member, roomId);
  await waitCollabConnected(a.page);
  await waitCollabConnected(b.page);
  return { a, b, owner, member, roomId };
}
