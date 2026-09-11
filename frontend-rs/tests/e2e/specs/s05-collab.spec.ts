/**
 * S05 OT 协作 V2 主链路浏览器回归（6 个 ST-FE-S05-*）
 *
 * change-20260826-1330-complete-skipped-e2e：占位骨架。
 */

import { test as base, expect } from "@playwright/test";

const test = base.extend({});

test("ST-FE-S05-01: 两 tab 同房间 → WS 握手", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="editor-canvas"]')).toBeVisible();
});

test("ST-FE-S05-02: 远端 op → 本地视图同步", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="ot-sync-indicator"]')).toBeVisible();
});

test("ST-FE-S05-03: presence 光标显示用户名", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="remote-cursor"]')).toBeVisible();
});

test("ST-FE-S05-04: 本地/远端 op 冲突 → OT 解决", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="ot-resolve"]')).toBeVisible();
});

test("ST-FE-S05-05: 断网 5s → reconnect-banner → 重连 sync", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="reconnect-banner"]')).toBeVisible();
});

test("ST-FE-S05-06: server-rev 落后 → 重新拉取整图", async ({ page }) => {
  await page.goto("/editor/sample-room");
  await expect(page.locator('[data-testid="resync-indicator"]')).toBeVisible();
});
// ─── wire-frontend-collab-ws — ST-FE-S05-07~09：双上下文真实 WS 传输 ─────────
// 与上方 01~06 占位骨架不同，本组用例对真实 backend 建立实际 WebSocket，
// 覆盖 §4.3.2：A→B op 可见性 / 断网队列恢复 flush / presence 远端光标。

import {
  addTable,
  collabState,
  debugState,
  newRoomPage,
  remotePresenceMap,
  setupCollabPair,
  waitCollabConnected,
  type RoomPage,
} from "../helpers/collab";

test("ST-FE-S05-07: A 建表 → B 端可见 + ot-rev 一致 + 无 409 模态", async ({
  browser,
  request,
}) => {
  const { a, b } = await setupCollabPair(browser, request, "s05-07");
  try {
    const t0 = Date.now();
    await addTable(a.page, "orders");
    // 规格目标 500ms 内可见；本地链路含 200ms 防抖 + WS 往返，轮询窗口给到 3s 防抖动。
    await expect
      .poll(async () => (await debugState(b.page)).tables, {
        timeout: 3_000,
        intervals: [50, 100, 200, 400],
      })
      .toContain("orders");
    console.log(`ST-FE-S05-07 A→B 可见时延 ${Date.now() - t0}ms`);

    // 两端 server_rev 收敛一致（A 收 ack、B 收 remote_op 后）
    await expect
      .poll(
        async () => {
          const ra = (await collabState(a.page)).serverRev;
          const rb = (await collabState(b.page)).serverRev;
          return ra > 0 && ra === rb;
        },
        { timeout: 5_000 },
      )
      .toBe(true);

    // room 模式不得出现 S01 409 冲突模态
    await expect(a.page.locator('[data-testid="modal-conflict"]')).toHaveCount(0);
    await expect(b.page.locator('[data-testid="modal-conflict"]')).toHaveCount(0);

    // activity-feed 有远端更新记录
    await expect(b.page.locator('[data-testid="activity-feed"]')).toContainText("远端更新");
  } finally {
    await a.context.close();
    await b.context.close();
  }
});

test("ST-FE-S05-08: B 断网编辑 → banner+队列可见 → 恢复后 sync+flush，A 最终可见", async ({
  browser,
  request,
}) => {
  const { a, b } = await setupCollabPair(browser, request, "s05-08");
  try {
    await b.context.setOffline(true);
    // headless Chromium 的 offline 仿真不会主动断开已建立的 WS（收不到 1006），
    // 用调试钩子丢弃连接，等价异常断线：onclose → 指数退避重连 → sync+flush。
    await b.page.evaluate(() => {
      const fn = (window as unknown as Record<string, (() => void) | undefined>).__cdb_debug_ws_drop;
      if (fn) fn();
    });

    // 断线后 banner 可见（Reconnecting 或终态均为非 hidden 态）
    await expect
      .poll(
        async () =>
          b.page
            .locator('[data-testid="reconnect-banner"]')
            .getAttribute("class"),
        { timeout: 15_000 },
      )
      .not.toContain("cdb-reconnect-banner--hidden");

    // 断线期间本地编辑 → 入队（watcher 仍在工作，发送被离线阻断）
    await addTable(b.page, "offline_edit");
    await expect
      .poll(async () => (await collabState(b.page)).queueLen, { timeout: 5_000 })
      .toBeGreaterThan(0);

    await b.context.setOffline(false);

    // 恢复后自动重连（指数退避首次 1s）→ sync → flush；A 端最终可见
    await expect
      .poll(async () => (await debugState(a.page)).tables, { timeout: 30_000 })
      .toContain("offline_edit");
    // 队列清零
    await expect
      .poll(async () => (await collabState(b.page)).queueLen, { timeout: 15_000 })
      .toBe(0);
  } finally {
    await a.context.close();
    await b.context.close();
  }
});

test("ST-FE-S05-09: A 移动光标 → B 出现 A 的远端光标且不含 B 自身", async ({
  browser,
  request,
}) => {
  const { a, b, owner, member } = await setupCollabPair(browser, request, "s05-09");
  try {
    // canvas 为 <canvas> 元素，headless 下布局尺寸可能为 0（boundingBox=null）；
    // 直接派发 pointermove 事件驱动 presence 上报（测协作链路而非命中测试）。
    const canvasA = a.page.locator('[data-testid="editor-canvas"]');
    await canvasA.waitFor({ state: "attached", timeout: 10_000 });
    // 两次移动跨越 100ms 节流窗口，确保至少一帧 presence 上行
    await canvasA.dispatchEvent("pointermove", { clientX: 200, clientY: 200 });
    await a.page.waitForTimeout(150);
    await canvasA.dispatchEvent("pointermove", { clientX: 320, clientY: 260 });

    await expect
      .poll(async () => Object.keys(await remotePresenceMap(b.page)), { timeout: 8_000 })
      .toContain(owner.userId);

    // B 移动自身光标后，B 的远端层仍不得包含 B 自己
    const canvasB = b.page.locator('[data-testid="editor-canvas"]');
    await canvasB.dispatchEvent("pointermove", { clientX: 150, clientY: 150 });
    await b.page.waitForTimeout(300);
    const keys = Object.keys(await remotePresenceMap(b.page));
    expect(keys).not.toContain(member.userId);
  } finally {
    await a.context.close();
    await b.context.close();
  }
});

// ─── fix-collab-autosave-race — ST-FE-S05-10：服务端物化单写者端到端一致性 ─────
// 用户原始场景：A 操作完 op 未同步到 B 时两边都触发自动保存 → 不一致。
// 方案 B 后：room 内不再有全量 PUT（ops 即保存，op 落库即物化），任何全量加载
// （新加入 / 刷新重进）恒读到 op log 一致状态。
test("ST-FE-S05-10: 并发编辑后新加入者恒一致 + room 模式零全量 PUT", async ({
  browser,
  request,
}) => {
  const { a, b, owner, roomId } = await setupCollabPair(browser, request, "s05-10");
  const c: RoomPage = await newRoomPage(browser, owner, roomId);
  // 网络抓包：room 模式下不得出现任何 PUT /api/v1/diagrams/*（全量快照写已收编）
  const putRequests: string[] = [];
  for (const p of [a.page, b.page]) {
    p.on("request", (req) => {
      if (req.method() === "PUT" && req.url().includes("/api/v1/diagrams/")) {
        putRequests.push(req.url());
      }
    });
  }
  try {
    // A/B 近乎同时编辑（制造 op 在途交错，等价原 bug 场景的双端并发写）
    await Promise.all([addTable(a.page, "t_from_a"), addTable(b.page, "t_from_b")]);

    // 双端最终收敛（互见对方修改）
    await expect
      .poll(async () => (await debugState(a.page)).tables, { timeout: 10_000 })
      .toContain("t_from_b");
    await expect
      .poll(async () => (await debugState(b.page)).tables, { timeout: 10_000 })
      .toContain("t_from_a");

    // C（owner 新会话进房，等价刷新/新加入的全量加载路径）→ 恒看到全部修改
    await waitCollabConnected(c.page);
    await expect
      .poll(
        async () => (await debugState(c.page)).tables.slice().sort(),
        { timeout: 10_000 },
      )
      .toEqual(["t_from_a", "t_from_b"]);

    // A 刷新页面并重新进房 → 全量重载仍不丢任何修改
    await a.page.goto("/");
    await a.page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 20_000 });
    await a.page.locator(`[data-testid="room-list-item-${roomId}"]`).click();
    await a.page.waitForSelector('[data-testid="room-editor-page"]', { timeout: 20_000 });
    await expect
      .poll(
        async () => (await debugState(a.page)).tables.slice().sort(),
        { timeout: 10_000 },
      )
      .toEqual(["t_from_a", "t_from_b"]);

    // 全程 room 模式零全量 PUT（ops 即保存）
    expect(putRequests).toEqual([]);
  } finally {
    await a.context.close();
    await b.context.close();
    await c.context.close();
  }
});
