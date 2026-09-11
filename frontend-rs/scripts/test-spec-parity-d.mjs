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
//   ST-PB-01         关系工具点击两点直接落账（p0-fix 定点 3：确认条已删除）
//   ST-PB-02         关系工具拖线（≥4px + rubber-band）直接落账（p0-fix 定点 3）
// p0-fix 定点 3 新增：
//   ST-PB-03         点击连线选中 + Inspector 详情/删除（不弹详情模态）
//   ST-PB-04         选中连线 Delete / Backspace 双键删除落账 0 条
//   ST-AN-01         拖框创建区域 → Inspector 编辑 → Delete 键删除落账 0 个
//   ST-AN-02         点击放置便签 → Inspector 编辑内容 → 按钮删除落账 0 个
//   ST-CR-NOTE-01    便签拖动落账（mousemove 不 PUT）+ 内容连续键入不丢字
//   ST-CR-AREA-01    区域拖动落账（宽高不变）
//   ST-CR-TAG-01     Inspector 字段 tag 受控输入连续键入不丢字
//   ST-LV-01         ListView 表树+单表网格（树节点选中表；行内编辑控件；组头锚点移除）
//   ST-SP-LIST-01    列表视图全屏渲染（网格定位 + 层叠遮挡 + 树+单表 10 列编辑网格）
//   ST-SP-LIST-02    列表视图行内编辑落账（名称/类型长度/勾选/增删移动/撤销）
//   ST-SP-LIST-03    列表视图表树导航（搜索过滤 + 切表渲染 + 空态 + 删表回落）
//   ST-DB-02         引擎创建后锁定（AppBar 禁用）；PG 基类型清单；UUID 无占位重复
// fix-appbar-roomname-back-and-import-merge 新增：
//   ST-PC-01         导入 DDL 本地合并进当前画布（2→4 表 + FK 落账 + Toast + 不跳页面不调 bridge）
//   ST-S04-UI-12     AppBar「← 空间」返回入口 + 长房间名缩略 title 悬停全文
// fix-overflow-menu-room-delete-listview-io 新增：
//   ST-PC-02         ListView 工具条导入/导出（本地合并落账 + 表树含新表 + 导出预览含模型）
//   ST-PC-03         720×480 小视口更多菜单边界保护（bounding box 不越界 + 滚动可达末项）
//   ST-S04-UI-13     房间列表卡片删除入口（owner 可见 + 确认模态 + DELETE 落账 + 卡片移除）
// feat-db-connect-import-and-ddl-realdb-verify 新增：
//   ST-PC-04         数据库连接导入：数据库 Tab → 连接并解析 → 摘要 → 合并落账（PUT 2→3，走 bridge/import/connect）
// feat-room-recycle-bin-dropdown-and-db-export 新增：
//   ST-S04-UI-14     回收站恢复：btn-room-trash → 恢复 → POST /rooms/{id}/restore + Toast + 列表重现
//   ST-S04-UI-15     回收站彻底删除：二次确认模态 → DELETE /rooms/{id}/permanent + Toast + 空态
//   ST-PC-05         导出到数据库：SQL Tab 填连接信息 → 在数据库中执行（ddl 与预览一致）→ 结果反馈 + Toast
// fix-dbimport-save-and-pg-schema 新增：
//   ST-PC-06         二次导入 + 保存成功 + schema 透传：两次连接导入（sqlite/postgres）→ 保存 ID 无交集 → schema 仅 postgres 携带
// fix-select-bg-diagram-delete-and-log-format 新增：
//   ST-S04-UI-16     创建空间弹窗删除游离图表：选中 → 删除入口 → 二次确认 → DELETE 落账 + Toast + 候选移除 + 回落新建
// fix-canvas-zoom-invite-comment-resize 新增：
//   ST-PC-04         MODIFIED：import/connect 响应改结构化 {table_count, tables[]}（core-03 §13.1），
//                    断言迁移为 tables 数组 + 表/列 comment 透传（mock 含 comment）
//   UT-S04-UI-17     前端 base_url 同源派生（core-01-deployment-plan §12.2）：API 路由按 path 匹配，
//                    不再锚定 BACKEND_URL host
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
import { installMockCollabWs } from "./mock-collab-ws.mjs";

async function installApi(page, options = {}) {
  // fix-collab-autosave-race：方案B 后 room 模式无全量 PUT——putCalls/lastPutBody
  // 由 mock WS op 落账驱动（op 上行次数 / 物化文档快照），断言语义「落账」不变。
  const state = { requests: [], putCalls: 0, lastPutBody: null };
  let diagramRev = 0;

  // fix-listview-grid-and-room-dbtype: 离线环境兜底——index.html 的 Google Fonts 为渲染阻塞资源，
  // 外网不可达时 page.goto(waitUntil:"load") 必超时；直接 abort 字体请求让 load 立即触发
  await page.route("https://fonts.googleapis.com/**", route => route.abort());
  await page.route("https://fonts.gstatic.com/**", route => route.abort());
  // fix-canvas-zoom-invite-comment-resize（UT-S04-UI-17）：前端 base_url 同源派生
  // window.location.origin，路由按 path 匹配（不再锚定 BACKEND_URL host）
  await page.route("**/diagrams/queryAll", route => response(route, 200, {
    // fix-select-bg-diagram-delete-and-log-format：可通过 options.diagramsList 预置游离图表（ST-S04-UI-16）
    code: 0, message: "success", data: options.diagramsList ?? [],
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
      // feat-room-recycle-bin-dropdown-and-db-export：archived=true → 回收站列表（ST-S04-UI-14/15）
      if (url.searchParams.get("archived") === "true") {
        const archived = options.archivedRooms ?? [];
        return response(route, 200, { items: archived, total: archived.length });
      }
      return response(route, 200, { items: options.roomsList ?? [], total: (options.roomsList ?? []).length });
    }
    // feat-room-recycle-bin-dropdown-and-db-export：恢复归档房间（ST-S04-UI-14）
    if (url.pathname.startsWith("/api/v1/rooms/") && url.pathname.endsWith("/restore") && request.method() === "POST") {
      const roomId = url.pathname.split("/")[4];
      state.restoredRooms = state.restoredRooms ?? [];
      state.restoredRooms.push(roomId);
      return response(route, 200, {
        id: roomId, name: "归档评审室", diagramId: "diagram-arch", ownerId: "owner-1",
        diagramTitle: "归档评审室", myRole: "owner", memberCount: 1,
      });
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
        id: "room-new", name: options.roomName ?? "架构评审室", diagramId: "diagram-new", ownerId: "owner-1",
        diagramTitle: options.roomName ?? "架构评审室", myRole: options.myRole ?? "owner", memberCount: 1,
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
        // fix-pg-types-listview-zindex-lock-engine: diagramDatabase 透传（缺省 null = Generic 缺省，回归不破）
        data: { id: "diagram-new", name: "架构评审室", database: options.diagramDatabase ?? null, revision: diagramRev, tables: [], references: [], areas: [], notes: [] },
      });
    }
    if (url.pathname === "/api/v1/diagrams/diagram-new" && request.method() === "PUT") {
      state.putCalls += 1;
      state.lastPutBody = request.postDataJSON();
      // fix-dbimport-save-and-pg-schema：全量保存体序列（ST-PC-06 跨次 ID 无交集断言）
      state.putBodies = state.putBodies ?? [];
      state.putBodies.push(state.lastPutBody);
      diagramRev += 1;
      return response(route, 200, { code: 0, request_id: "save-diagram", data: { revision: diagramRev } });
    }
    if (url.pathname.endsWith("/collab/head")) {
      return response(route, 200, { roomId: "room-new", diagramId: "diagram-new", serverRev: 7 });
    }
    if (url.pathname.endsWith("/members")) return response(route, 200, MEMBERS);
    // fix-select-bg-diagram-delete-and-log-format：DELETE /diagrams/{id}（ST-S04-UI-16）
    if (url.pathname.startsWith("/api/v1/diagrams/") && request.method() === "DELETE") {
      state.deletedDiagrams = state.deletedDiagrams ?? [];
      state.deletedDiagrams.push(url.pathname.split("/").pop());
      return response(route, 200, { code: 0, request_id: "delete-diagram", data: { id: url.pathname.split("/").pop() } });
    }
    // fix-overflow-menu-room-delete-listview-io：DELETE /rooms/{id}（ST-S04-UI-13）
    if (url.pathname.startsWith("/api/v1/rooms/") && request.method() === "DELETE") {
      // feat-room-recycle-bin-dropdown-and-db-export：/permanent 硬删除（ST-S04-UI-15）
      if (url.pathname.endsWith("/permanent")) {
        state.purgedRooms = state.purgedRooms ?? [];
        state.purgedRooms.push(url.pathname.split("/")[4]);
        return response(route, 204);
      }
      state.deletedRooms = state.deletedRooms ?? [];
      state.deletedRooms.push(url.pathname.split("/").pop());
      return response(route, 204);
    }
    // feat-db-connect-import-and-ddl-realdb-verify：POST /bridge/import/connect（ST-PC-04）
    // fix-canvas-zoom-invite-comment-resize：响应改结构化 tables JSON（core-03 §13.1，
    // {table_count, tables[]} 取代 {tables: int, ddl}），含表/列 comment 透传断言
    if (url.pathname === "/api/v1/bridge/import/connect" && request.method() === "POST") {
      state.connectCalls = (state.connectCalls ?? 0) + 1;
      state.lastConnectBody = request.postDataJSON();
      return response(route, 200, {
        code: 0,
        request_id: "import-connect",
        data: {
          engine: "sqlite",
          table_count: 1,
          tables: [
            {
              name: "posts",
              comment: "文章表",
              fields: [
                { name: "id", type: "UUID", primary: true, not_null: true, unique: false, increment: false, default: "", comment: "" },
                { name: "title", type: "VARCHAR(64)", primary: false, not_null: true, unique: false, increment: false, default: "", comment: "标题" },
              ],
            },
          ],
        },
      });
    }
    // feat-room-recycle-bin-dropdown-and-db-export：POST /bridge/export/execute（ST-PC-05）
    if (url.pathname === "/api/v1/bridge/export/execute" && request.method() === "POST") {
      state.exportExecuteCalls = (state.exportExecuteCalls ?? 0) + 1;
      state.lastExportExecuteBody = request.postDataJSON();
      return response(route, 200, {
        code: 0,
        request_id: "export-execute",
        data: { engine: "sqlite", statements: 2, tables: 2 },
      });
    }
    return response(route, 404, { code: "NOT_FOUND" });
  });
  // fix-collab-autosave-race：mock 协作 WS——op 上行即 ack + 服务端物化（见 mock-collab-ws.mjs）
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

// p0-fix 定点 3：等保存完成（data-state=saved）。
// 不用 locator visible wait：保存完成瞬间 dirty/is_saving/revision 多信号连变，
// Leptos 会快速重建 chip 节点，locator 轮询可能稳定命中到刚被替换的隐藏旧节点（偶发 flake）。
async function waitSaved(page, timeoutMs = 8_000) {
  await page.waitForFunction(
    () => document.querySelector('[data-testid="save-state"]')?.getAttribute("data-state") === "saved",
    { timeout: timeoutMs }
  );
}

// p0-fix 定点 3：从 canvas data-follow-path 解析连线中点
// （贝塞尔控制点在两端中线上，曲线中点恰为端点中点，见 editor_render bezier_controls）
async function relationLineMidpoint(page) {
  const d = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");
  assert.ok(d && d.startsWith("M"), "关系线 data-follow-path 必须存在");
  const nums = d.match(/-?\d+\.?\d*/g).map(Number);
  // M x1 y1 C cx1 cy1,cx2 cy2,x2 y2 → 共 8 个数
  const [x1, y1, , , , , x2, y2] = nums;
  return { x: (x1 + x2) / 2, y: (y1 + y2) / 2 };
}

// p0-fix 定点 3：建一条「可见」关系——先把 table_2 拖开（默认布局两表重叠，
// 连线被表遮住时命中顺序表 > 连线），再点击两点直接落账，最后 Esc 退出关系工具
async function createVisibleRelation(page) {
  await createTwoTables(page);
  const header = await canvasPoint(page, TABLE2_HEADER);
  await page.mouse.move(header.x, header.y);
  await page.mouse.down();
  await page.mouse.move(header.x + 300, header.y + 100, { steps: 5 });
  await page.mouse.up();
  await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
  await waitForCanvasStable(page);

  await page.keyboard.press("r");
  await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
  let p = await canvasPoint(page, TABLE1_FIELD);
  await page.mouse.click(p.x, p.y);
  p = await canvasPoint(page, { x: TABLE2_FIELD.x + 300, y: TABLE2_FIELD.y + 100 });
  await page.mouse.click(p.x, p.y);
  await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
  await page.keyboard.press("Escape");
  await page.locator('[data-testid="rel-tool-hint"]').waitFor({ state: "hidden" });
  await waitForCanvasStable(page);
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

// 调试：SPEC_PARITY_ONLY=ST-PC-04 只跑指定用例（逗号分隔）
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
    // fix-collab-autosave-race：失败时转储协作状态机事件环（op 收发/ack/sync 时序），
    // 供间歇性丢 op 类故障定位
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

  // ─── ST-FE-S05-11：远端帧到达时 debounce 窗口内的本地编辑不丢（fix-collab-autosave-race）─
  // ST-CR-INSP-01 间歇失败根因回归：旧实现 apply_remote/sync 处理器无条件重采 baseline，
  // 建表后 200ms debounce 窗口内到达的远端帧会把未上行的 table.create 吞进 baseline
  // （op 永远不发，后续依赖它的 field.update 上行时服务端文档里根本没有该表）。
  await run(["ST-FE-S05-11"], "remote_op 到达不吞 debounce 窗口内的本地建表", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    // 等 WS 连接 + 首连 sync 全部完成（flush 事件出现）——否则注入的 remote_op 会因
    // rev 缺口/sync 在途被缓冲（Buffer），apply_remote 不执行，race 窗口根本不被触发
    // （阴性验证：无此等待时用例在旧实现上也 PASS，属空转）。
    await page.waitForFunction(() => {
      const fn = window.__cdb_collab_state;
      if (typeof fn !== "function") return false;
      try {
        const s = JSON.parse(fn());
        return s.connection === "Connected" && (s.events ?? []).some(e => e.startsWith("flush"));
      } catch { return false; }
    }, { timeout: 10_000 });
    assert.ok(state.__mockWs, "ST-FE-S05-11: mock WS 必须已连接");

    // 建表后立即注入远端 op（<200ms debounce 窗口）：修复版须先把本地编辑结算成 op
    // 再应用远端帧；旧版会在 apply_remote 重采 baseline 时吞掉 table.create。
    await page.keyboard.press("t");
    const pushed = state.pushRemoteOp({
      type: "note.create", targetId: "note-remote-1",
      changes: { id: "note-remote-1", x: 400, y: 300, content: "远端便签", color: "#ff0" },
    });
    assert.ok(pushed, "ST-FE-S05-11: 远端 op 必须推送成功");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();

    // 保存收敛后：服务端物化文档必须同时含 table_1（本地）与 note-remote-1（远端）
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    const tables = state.lastPutBody?.diagram?.tables?.map(t => t.name) ?? [];
    assert.ok(
      tables.includes("table_1"),
      `ST-FE-S05-11: debounce 窗口内的本地建表不得被远端帧吞掉（tables=${JSON.stringify(tables)}）`,
    );
    assert.ok(
      state.lastPutBody?.diagram?.notes?.some(n => n.id === "note-remote-1"),
      "ST-FE-S05-11: 远端 note 须已物化进服务端文档",
    );
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

  // ─── ST-PC-01：导入 DDL 本地合并进当前画布（fix-appbar-roomname-back-and-import-merge） ──
  await run(["ST-PC-01"], "导入 DDL 合并进当前画布并落账", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page); // 已有 2 表
    const urlBefore = page.url();

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-import"]').click();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    await page.locator('[data-testid="import-textarea"]').fill(
      "CREATE TABLE posts (id UUID PRIMARY KEY, title VARCHAR(64) NOT NULL);\n" +
      "CREATE TABLE comments (id UUID PRIMARY KEY, post_id UUID REFERENCES posts(id));",
    );
    // 解析摘要可见
    await page.locator('[data-testid="import-parse-summary"]:visible').waitFor();
    await page.locator('[data-testid="import-submit"]').click();

    // Toast「已导入」+ 抽屉关闭
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已导入/, "合并成功后必须有「已导入」Toast");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });

    // 合并进当前画布：落账 PUT 含 4 表 + 1 条关系
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 4, "导入合并后 PUT 必须含 4 张表");
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "导入 FK 必须落账 1 条关系");
    // 无页面跳转；不调 bridge import/local（本地合并）
    assert.equal(page.url(), urlBefore, "导入合并不得跳转页面");
    assert.ok(
      !state.requests.some(r => r === "POST /api/v1/bridge/import/local"),
      "本地合并不得调用 bridge/import/local",
    );
  });

  // ─── ST-S04-UI-12：AppBar「← 空间」返回入口 + 长房间名缩略 title（fix-appbar-roomname-back-and-import-merge） ──
  await run(["ST-S04-UI-12"], "AppBar 返回空间按钮与房间名缩略悬停全文", async page => {
    const longName = "架构评审室-国际化中台-数据模型协作空间-超长名称";
    await installApi(page, { roomName: longName });
    await login(page);
    await createRoomAndEnter(page, longName);

    const back = page.locator('[data-testid="btn-back-to-rooms"]:visible');
    await back.waitFor();
    const badge = page.locator('[data-testid="room-badge"]:visible');
    await badge.waitFor();

    // 返回按钮位于 room-badge 左侧
    const backBox = await back.boundingBox();
    const badgeBox = await badge.boundingBox();
    assert.ok(backBox && badgeBox, "返回按钮与 room-badge 都必须有布局盒");
    assert.ok(backBox.x + backBox.width <= badgeBox.x + 1, "「← 空间」必须在 room-badge 左侧");

    // 长房间名缩略：title 属性为全文
    assert.equal(await badge.getAttribute("title"), `${longName}（点击返回房间列表）`, "room-badge title 必须为房间全名");
    const badgeWidth = badgeBox.width;
    assert.ok(badgeWidth <= 190, `room-badge 应缩略限宽（实际 ${badgeWidth}px）`);

    // 点击「← 空间」返回房间列表页（SPA 内部路由 PageState::Rooms，与既有 room-badge 返回行为一致）
    await back.click();
    await page.locator('[data-testid="rooms-list-page"]:visible').waitFor();
    await page.locator('[data-testid="room-editor-page"]').waitFor({ state: "hidden" });
  });

  // ─── ST-PC-02：ListView 工具条导入/导出（fix-overflow-menu-room-delete-listview-io） ──
  await run(["ST-PC-02"], "ListView 导入合并落账 + 导出预览含模型", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page); // 已有 2 表

    // 进入全屏 ListView
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();

    // 工具条「导入」→ IoDrawer
    await page.locator('[data-testid="list-btn-import"]').click();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    await page.locator('[data-testid="import-textarea"]').fill(
      "CREATE TABLE imported_posts (id UUID PRIMARY KEY, title VARCHAR(64) NOT NULL);",
    );
    await page.locator('[data-testid="import-parse-summary"]:visible').waitFor();
    await page.locator('[data-testid="import-submit"]').click();

    // Toast「已导入」+ 抽屉自动关闭 + 落账 3 表（本地合并不调 bridge）
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已导入/);
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, "ListView 导入合并后 PUT 必须含 3 张表");
    assert.ok(
      !state.requests.some(r => r === "POST /api/v1/bridge/import/local"),
      "ListView 导入不得调用 bridge/import/local",
    );

    // 回到 ListView 且表树含新表
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    await page.locator('[data-testid="list-tree-node-imported_posts"]:visible').waitFor();

    // 工具条「导出」→ 预览含当前模型
    await page.locator('[data-testid="list-btn-export"]').click();
    await page.locator('[data-testid="export-drawer"]:visible').waitFor();
    const preview = page.locator('[data-testid="export-preview"]');
    await preview.waitFor();
    assert.match((await preview.textContent()) ?? "", /CREATE TABLE imported_posts \(/, "导出预览必须含新导入的表");
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
  });

  // ─── ST-PC-03：小视口更多菜单边界保护（fix-overflow-menu-room-delete-listview-io） ──
  await run(["ST-PC-03"], "720×480 视口更多菜单不越界且可滚动到达末项", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    await page.locator('[data-testid="btn-more-menu"]').click();
    const menu = page.locator('[data-testid="app-bar-overflow-menu"]:visible');
    await menu.waitFor();

    const box = await menu.boundingBox();
    assert.ok(box, "溢出菜单必须有布局盒");
    assert.ok(box.x >= 0 && box.y >= 0, `菜单左上角不得越出视口（实际 ${box.x},${box.y}）`);
    assert.ok(box.x + box.width <= 720, `菜单右缘不得越出 720px 视口（实际 ${box.x + box.width}）`);
    assert.ok(box.y + box.height <= 480, `菜单下缘不得越出 480px 视口（实际 ${box.y + box.height}）`);

    // 菜单可滚动到达末项「命令面板」
    const last = page.locator('[data-testid="btn-command-palette"]');
    await last.scrollIntoViewIfNeeded();
    const lastBox = await last.boundingBox();
    assert.ok(lastBox, "末项「命令面板」必须可见");
    assert.ok(lastBox.y + lastBox.height <= 480, "滚动后末项必须完整落在视口内");
  }, { viewport: { width: 720, height: 480 } });

  // ─── ST-PC-04：数据库连接导入（feat-db-connect-import-and-ddl-realdb-verify，core-01d §4.4） ──
  await run(["ST-PC-04"], "数据库 Tab 连接并解析 → 合并落账（bridge/import/connect）", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page); // 已有 2 表
    const urlBefore = page.url();

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-import"]').click();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    await page.locator('[data-testid="io-format-tabs"] button', { hasText: "数据库" }).click();
    await page.locator('[data-testid="import-db-engine"]').selectOption("sqlite");
    await page.locator('[data-testid="import-db-source"]').fill("/tmp/fixture.db");
    await page.locator('[data-testid="import-db-connect"]').click();

    // 连接成功摘要：已连接 · 检测到 1 张表（mock 回传 tables=1）
    const summary = page.locator('[data-testid="import-parse-summary"]:visible');
    try {
      await summary.waitFor({ timeout: 3_000 });
    } catch {
      const drawerText = ((await page.locator('[data-testid="import-drawer"]').textContent()) ?? "").replace(/\s+/g, " ");
      throw new Error(
        `连接后摘要未出现；drawer="${drawerText.slice(0, 300)}"；requests=${state.requests.join(" | ")}；` +
        `lastConnectBody=${JSON.stringify(state.lastConnectBody ?? null)}`,
      );
    }
    assert.match((await summary.textContent()) ?? "", /已连接 · 检测到 1 张表/, "连接成功后摘要须显示已连接与表数");

    await page.locator('[data-testid="import-submit"]').click();

    // Toast「已导入」+ 抽屉关闭
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已导入/, "合并成功后必须有「已导入」Toast");
    await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });

    // 合并进当前画布：落账 PUT 含 3 表（2 + 1）；请求走 bridge/import/connect
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, "连接导入合并后 PUT 必须含 3 张表");
    // fix-canvas-zoom-invite-comment-resize：结构化 tables 表/列 comment 透传进模型（UT-PC-26 端到端）
    const importedTable = state.lastPutBody?.diagram?.tables?.[2];
    assert.equal(importedTable?.name, "posts", "连接导入表名须透传");
    assert.equal(importedTable?.comment, "文章表", "结构化 tables 表 comment 须透传进模型");
    assert.equal(importedTable?.fields?.[1]?.comment, "标题", "结构化 tables 列 comment 须透传进模型");
    assert.ok(
      state.requests.some(r => r === "POST /api/v1/bridge/import/connect"),
      "数据库来源必须调用 bridge/import/connect",
    );
    assert.deepEqual(
      { engine: state.lastConnectBody?.engine, source: state.lastConnectBody?.source },
      { engine: "sqlite", source: "/tmp/fixture.db" },
      "connect 请求体须携带所选引擎与连接信息（Bearer 头由拦截器处理）",
    );
    assert.ok(
      !state.requests.some(r => r === "POST /api/v1/bridge/import/local"),
      "连接导入不得调用 bridge/import/local",
    );
    assert.equal(page.url(), urlBefore, "导入合并不得跳转页面");
  });

  // ─── ST-S04-UI-13：房间列表卡片删除入口（fix-overflow-menu-room-delete-listview-io） ──
  await run(["ST-S04-UI-13"], "owner 卡片删除入口 + 确认模态 + DELETE 落账", async page => {
    const state = await installApi(page, {
      roomsList: [
        { id: "room-own", name: "我的评审室", diagramId: "diagram-own", diagramTitle: "我的评审室",
          myRole: "owner", memberCount: 1, updatedAt: "2026-09-07T00:00:00Z" },
        { id: "room-edit", name: "协作评审室", diagramId: "diagram-edit", diagramTitle: "协作评审室",
          myRole: "editor", memberCount: 3, updatedAt: "2026-09-06T00:00:00Z" },
      ],
    });
    await login(page);

    // 删除入口仅 owner 可见
    await page.locator('[data-testid="room-card-delete-room-own"]').waitFor();
    assert.equal(
      await page.locator('[data-testid="room-card-delete-room-edit"]').count(), 0,
      "editor 角色不得渲染删除按钮",
    );

    // 点击删除：不进房间，弹确认模态
    await page.locator('[data-testid="room-card-delete-room-own"]').click();
    await page.locator('[data-testid="modal-delete-room-list"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid="room-editor-page"]:visible').count(), 0, "点击删除不得进入房间");
    assert.equal(await page.locator('[data-testid="room-card-room-own"]').count(), 1, "确认前卡片不得移除");

    // 确认删除：DELETE 落账 + 卡片移除 + Toast
    await page.locator('[data-testid="btn-confirm-delete-room-list"]').click();
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="room-list-item-room-own"]'),
      { timeout: 5_000 },
    );
    assert.deepEqual(state.deletedRooms, ["room-own"], "必须发起 DELETE /rooms/room-own");
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已删除/);
    // editor 卡片保留
    await page.locator('[data-testid="room-list-item-room-edit"]:visible').waitFor();
  });

  // ─── ST-S04-UI-14：回收站恢复（feat-room-recycle-bin-dropdown-and-db-export，core-S04 §2.4） ──
  await run(["ST-S04-UI-14"], "回收站 → 恢复归档房间 → Toast + 列表重现", async page => {
    const state = await installApi(page, {
      roomsList: [
        { id: "room-arch", name: "归档评审室", diagramId: "diagram-arch", diagramTitle: "归档评审室",
          myRole: "owner", memberCount: 1, updatedAt: "2026-09-07T00:00:00Z" },
      ],
      archivedRooms: [
        { id: "room-arch", name: "归档评审室", diagramId: "diagram-arch", diagramTitle: "归档评审室",
          myRole: "owner", memberCount: 1, updatedAt: "2026-09-06T00:00:00Z", archivedAt: "2026-09-07T12:00:00Z" },
      ],
    });
    await login(page);

    // 打开回收站：展示归档房间名与归档时间
    await page.locator('[data-testid="btn-room-trash"]').click();
    await page.locator('[data-testid="room-trash-view"]:visible').waitFor();
    const item = page.locator('[data-testid="room-trash-item-room-arch"]');
    await item.waitFor();
    const itemText = (await item.textContent()) ?? "";
    assert.match(itemText, /归档评审室/, "回收站须展示归档房间名");
    assert.match(itemText, /归档于 2026-09-07T12:00:00Z/, "回收站须展示归档时间");

    // 恢复：POST /rooms/{id}/restore + Toast「已恢复房间」+ 条目移出回收站
    await page.locator('[data-testid="btn-restore-room-room-arch"]').click();
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="room-trash-item-room-arch"]'),
      { timeout: 5_000 },
    );
    assert.deepEqual(state.restoredRooms, ["room-arch"], "必须发起 POST /rooms/room-arch/restore");
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已恢复房间/, "恢复成功必须有「已恢复房间」Toast");

    // 返回列表：房间重现
    await page.locator('[data-testid="btn-trash-back"]').click();
    await page.locator('[data-testid="room-list-item-room-arch"]:visible').waitFor();
  });

  // ─── ST-S04-UI-15：回收站彻底删除（feat-room-recycle-bin-dropdown-and-db-export，core-S04 §2.4） ──
  await run(["ST-S04-UI-15"], "回收站 → 彻底删除二次确认 → DELETE /permanent 落账", async page => {
    const state = await installApi(page, {
      archivedRooms: [
        { id: "room-arch", name: "归档评审室", diagramId: "diagram-arch", diagramTitle: "归档评审室",
          myRole: "owner", memberCount: 1, updatedAt: "2026-09-06T00:00:00Z", archivedAt: "2026-09-07T12:00:00Z" },
      ],
    });
    await login(page);

    await page.locator('[data-testid="btn-room-trash"]').click();
    await page.locator('[data-testid="room-trash-item-room-arch"]').waitFor();

    // 点彻底删除 → 二次确认模态（正文含「不可恢复」与「Diagram 保留」），确认前条目不移除
    await page.locator('[data-testid="btn-purge-room-room-arch"]').click();
    const modal = page.locator('[data-testid="modal-purge-room"]:visible');
    await modal.waitFor();
    const modalText = (await modal.textContent()) ?? "";
    assert.match(modalText, /不可恢复/, "模态须含「不可恢复」警示");
    assert.match(modalText, /Diagram 保留/, "模态须含「Diagram 保留」说明");
    await page.locator('[data-testid="room-trash-item-room-arch"]:visible').waitFor();

    // 确认：DELETE /rooms/{id}/permanent + Toast「已彻底删除」+ 列表移除回到空态
    await page.locator('[data-testid="btn-confirm-purge-room"]').click();
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="room-trash-item-room-arch"]'),
      { timeout: 5_000 },
    );
    assert.deepEqual(state.purgedRooms, ["room-arch"], "必须发起 DELETE /rooms/room-arch/permanent");
    assert.ok(!state.deletedRooms?.length, "彻底删除不得走 DELETE /rooms/{id} 归档路径");
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已彻底删除/, "彻底删除成功必须有「已彻底删除」Toast");
    await page.locator('[data-testid="trash-empty"]:visible').waitFor();
  });

  // ─── ST-S04-UI-16：创建空间弹窗删除游离图表（fix-select-bg-diagram-delete-and-log-format，core-S04 图表行） ──
  await run(["ST-S04-UI-16"], "关联图表选中游离图表 → 删除入口 → 二次确认 → DELETE 落账 + Toast + 候选移除", async page => {
    const state = await installApi(page, {
      diagramsList: [{ id: "diagram-orphan", name: "游离图表" }],
    });
    await login(page);

    // 打开创建空间弹窗：候选含游离图表；未选中时无删除入口
    await page.locator('[data-testid="btn-create-room"]').click();
    await page.locator('[data-testid="modal-create-room"]:visible').waitFor();
    const select = page.locator('[data-testid="create-room-diagram"]');
    await select.locator('option[value="diagram-orphan"]').waitFor({ state: "attached" });
    assert.equal(
      await page.locator('[data-testid="create-room-diagram-delete"]').count(), 0,
      "选中「新建空白模型」时不得渲染删除按钮",
    );

    // 选中游离图表 → 删除入口出现 → 点击弹二次确认（确认前候选不移除）
    await select.selectOption("diagram-orphan");
    await page.locator('[data-testid="create-room-diagram-delete"]:visible').waitFor();
    await page.locator('[data-testid="create-room-diagram-delete"]').click();
    const modal = page.locator('[data-testid="modal-delete-diagram"]:visible');
    await modal.waitFor();
    const modalText = (await modal.textContent()) ?? "";
    assert.match(modalText, /游离图表/, "确认模态须含图表名");
    assert.match(modalText, /不可恢复/, "确认模态须含「不可恢复」警示");
    await select.locator('option[value="diagram-orphan"]').waitFor({ state: "attached" });

    // 确认删除：DELETE 落账 + Toast「已删除图表」+ 候选移除 + 选择回落新建空白模型
    await page.locator('[data-testid="btn-confirm-delete-diagram"]').click();
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="modal-delete-diagram"]'),
      { timeout: 5_000 },
    );
    assert.deepEqual(state.deletedDiagrams, ["diagram-orphan"], "必须发起 DELETE /api/v1/diagrams/diagram-orphan");
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已删除图表/, "删除成功必须有「已删除图表」Toast");
    assert.equal(
      await select.locator('option[value="diagram-orphan"]').count(), 0,
      "删除后候选下拉不得再含该图表",
    );
    assert.equal(await select.inputValue(), "__new__", "删除选中图表后应回落「新建空白模型」");
  });

  // ─── ST-PC-05：导出到数据库（feat-room-recycle-bin-dropdown-and-db-export，core-01d §5.4） ──
  await run(["ST-PC-05"], "SQL Tab 填连接信息 → 在数据库中执行 → 结果反馈 + Toast", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page); // 已有 2 表

    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-export"]').click();
    await page.locator('[data-testid="export-drawer"]:visible').waitFor();

    // 默认 SQL Tab：导出到数据库区块可见；空连接信息 → 内联提示不发请求
    await page.locator('[data-testid="export-db-source"]:visible').waitFor();
    await page.locator('[data-testid="export-db-execute"]').click();
    await page.waitForFunction(
      () => (document.querySelector('[data-testid="export-db-result"]')?.textContent ?? "").includes("请填写连接信息"),
      { timeout: 3_000 },
    );
    assert.equal(state.exportExecuteCalls ?? 0, 0, "空连接信息不得发请求");

    // 填连接信息执行：请求体 ddl 与预览区 SQL 一致
    const previewSql = (await page.locator('[data-testid="export-preview"]').textContent()) ?? "";
    await page.locator('[data-testid="export-db-source"]').fill("/tmp/export-fixture.db");
    await page.locator('[data-testid="export-db-execute"]').click();
    const result = page.locator('[data-testid="export-db-result"]');
    await page.waitForFunction(
      () => (document.querySelector('[data-testid="export-db-result"]')?.textContent ?? "").includes("已执行"),
      { timeout: 5_000 },
    );
    assert.match((await result.textContent()) ?? "", /已执行 2 条语句 · 建表 2 张/, "结果区须显示语句数与建表数");
    assert.equal(state.exportExecuteCalls, 1, "必须调用一次 bridge/export/execute");
    assert.equal(state.lastExportExecuteBody?.engine, "sqlite", "请求体 engine 须为目标库引擎");
    assert.equal(state.lastExportExecuteBody?.source, "/tmp/export-fixture.db", "请求体 source 须为连接信息");
    assert.equal(state.lastExportExecuteBody?.ddl, previewSql, "请求体 ddl 须与预览区 SQL 一致");

    // Toast「已导出到数据库」
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已导出到数据库/, "成功必须有「已导出到数据库」Toast");

    // 连接信息不入持久化存储
    const persisted = await page.evaluate(() => JSON.stringify(window.localStorage));
    assert.ok(!persisted.includes("/tmp/export-fixture.db"), "连接信息不得写入 localStorage");
  });

  // ─── ST-PC-06：二次导入 + 保存成功 + schema 透传（fix-dbimport-save-and-pg-schema，core-01d §4.4） ──
  await run(["ST-PC-06"], "数据库 Tab 连续两次导入 → 两次保存 ID 无交集 → schema 参数透传", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page); // 已有 2 表

    const connectImport = async (engine, source, schema) => {
      await page.locator('[data-testid="btn-more-menu"]').click();
      await page.locator('[data-testid="btn-import"]').click();
      await page.locator('[data-testid="import-drawer"]:visible').waitFor();
      await page.locator('[data-testid="io-format-tabs"] button', { hasText: "数据库" }).click();
      await page.locator('[data-testid="import-db-engine"]').selectOption(engine);
      await page.locator('[data-testid="import-db-source"]').fill(source);
      const schemaInput = page.locator('[data-testid="import-db-schema"]');
      if (engine === "postgres") {
        await schemaInput.waitFor({ state: "visible", timeout: 2_000 });
        await schemaInput.fill(schema);
      } else {
        assert.equal(await schemaInput.count(), 0, "sqlite 引擎不得渲染 schema 输入");
      }
      await page.locator('[data-testid="import-db-connect"]').click();
      const summary = page.locator('[data-testid="import-parse-summary"]:visible');
      await summary.waitFor({ timeout: 3_000 });
      await page.locator('[data-testid="import-submit"]').click();
      const toast = page.locator('[data-testid="notice-toast"]:visible');
      await toast.waitFor({ timeout: 3_000 });
      await page.locator('[data-testid="io-drawer"]').waitFor({ state: "hidden" });
      await waitSaved(page);
    };

    // 第一次导入：sqlite，无 schema 字段
    await connectImport("sqlite", "/tmp/fixture-a.db");
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, `第一次导入合并后 PUT 须含 3 表（actual=${state.lastPutBody?.diagram?.tables?.length}, tables=${JSON.stringify(state.lastPutBody?.diagram?.tables?.map(t => t.name))}, putCalls=${state.putCalls}, ops=${JSON.stringify(state.sentOps?.map(o => o?.type))}`);
    assert.ok(
      state.lastConnectBody && !("schema" in state.lastConnectBody),
      "sqlite 请求体不得携带 schema 字段",
    );
    const save1 = state.lastPutBody;

    // 第二次导入：postgres + schema 透传
    await connectImport("postgres", "postgres://localhost/db", "app");
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 4, `第二次导入合并后 PUT 须含 4 表（actual=${state.lastPutBody?.diagram?.tables?.length}, tables=${JSON.stringify(state.lastPutBody?.diagram?.tables?.map(t => t.name))}, putCalls=${state.putCalls}, ops=${JSON.stringify(state.sentOps?.map(o => o?.type))}`);
    assert.equal(state.lastConnectBody?.schema, "app", "postgres 请求体须携带输入的 schema");
    const save2 = state.lastPutBody;

    // 重键生效：单次保存体内实体 ID 无重复（原 bug：第二次导入再生成 import-t-0，
    // 与第一次导入的同 ID 在同一 PUT 内 INSERT 撞全局主键 → 500）
    const idsListOf = body => {
      const d = body?.diagram ?? {};
      const ids = [];
      for (const t of d.tables ?? []) {
        ids.push(t.id);
        for (const f of t.fields ?? []) ids.push(f.id);
      }
      for (const r of d.references ?? []) ids.push(r.id);
      return ids;
    };
    for (const [label, body] of [["save1", save1], ["save2", save2]]) {
      const ids = idsListOf(body);
      const dup = ids.filter((id, i) => ids.indexOf(id) !== i);
      assert.deepEqual([...new Set(dup)], [], `${label} 保存体内不得有重复实体 ID（主键冲突根源）`);
    }
    // 两次导入的同名表 posts 须持有不同 ID（重键全局唯一）
    const posts1 = save1.diagram.tables.filter(t => t.name === "posts").map(t => t.id);
    const posts2 = save2.diagram.tables.filter(t => t.name === "posts").map(t => t.id);
    assert.equal(posts1.length, 1, "save1 应含 1 张 posts");
    assert.equal(posts2.length, 2, "save2 应含 2 张 posts（二次导入不覆盖）");
    assert.notEqual(posts2[0], posts2[1], "save2 两张 posts ID 必须不同");

    // saveText 为「已保存」（两次 waitSaved 已保证 data-state=saved）
    const saveState = await page.locator('[data-testid="save-state"]').getAttribute("data-state");
    assert.equal(saveState, "saved", "两次导入后保存必须为已保存态（无 500 主键冲突）");
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

  // ─── ST-PB-01：关系工具点击两点直接落账（p0-fix 定点 3：确认条已删除） ────
  await run(["ST-PB-01"], "点击两点直接创建关系", async page => {
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

    // p0-fix 定点 3：第二击落点直接落账（无确认条）；确认条组件已删除
    assert.equal(await page.locator('[data-testid="rel-confirm-bar"]').count(), 0, "p0-fix 后不应存在确认条");
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "点击两点后必须直接落账 1 条关系");
  });

  // ─── ST-PB-02：拖线（≥4px + rubber-band）直接落账（p0-fix 定点 3） ────────
  await run(["ST-PB-02"], "字段拖线直接创建关系", async page => {
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

    // p0-fix 定点 3：落到目标字段松开直接落账（无确认条）
    assert.equal(await page.locator('[data-testid="rel-confirm-bar"]').count(), 0, "p0-fix 后不应存在确认条");
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "拖线松开后必须直接落账 1 条关系");
  });

  // ─── ST-PB-03：点击连线选中 + Inspector 详情/删除（relation-inspector-and-ddl-io：不再弹模态） ──
  await run(["ST-PB-03"], "点击连线选中并走 Inspector 删除", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createVisibleRelation(page);

    // 点击连线中点 → 选中 + Inspector 关系面板（不弹详情模态）
    const mid = await canvasPoint(page, await relationLineMidpoint(page));
    await page.mouse.click(mid.x, mid.y);
    assert.equal(await page.locator('[data-testid="modal-reference-detail"]').count(), 0, "点击连线不得弹详情模态");
    const panel = page.locator('[data-testid="inspector-reference-form"]:visible');
    await panel.waitFor();
    await panel.locator(".cdb-rel-confirm-bar__label").getByText(/table_1\.id → table_2\.id/).waitFor();

    // Inspector「删除关系」→ 落账 0 条关系
    await page.locator('[data-testid="inspector-ref-delete"]').click();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 0, "Inspector 删除后必须落账 0 条关系");
  });

  // ─── ST-PB-04：选中连线 Delete 与 Backspace 双键删除（relation-inspector-and-ddl-io） ──
  await run(["ST-PB-04"], "点击连线后 Delete 与 Backspace 键删除", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createVisibleRelation(page);

    // 点击连线选中（不弹模态）→ Delete 键删除
    const mid = await canvasPoint(page, await relationLineMidpoint(page));
    await page.mouse.click(mid.x, mid.y);
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.keyboard.press("Delete");
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 0, "Delete 键删除后必须落账 0 条关系");

    // 重建关系：既有 table_1/table_2 之间直接点击两点（table_2 已拖开 +300,+100；
    // 不能再走 createVisibleRelation——它会新建 table_3/4 且 TABLE2_HEADER 常量已失效）
    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    let rp = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(rp.x, rp.y);
    rp = await canvasPoint(page, { x: TABLE2_FIELD.x + 300, y: TABLE2_FIELD.y + 100 });
    await page.mouse.click(rp.x, rp.y);
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="rel-tool-hint"]').waitFor({ state: "hidden" });
    await waitForCanvasStable(page);
    const mid2 = await canvasPoint(page, await relationLineMidpoint(page));
    await page.mouse.click(mid2.x, mid2.y);
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await page.keyboard.press("Backspace");
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 0, "Backspace 键删除后必须落账 0 条关系");
  });

  // ─── ST-AN-01：拖框创建区域 → Inspector 编辑 → Delete 键删除（p0-fix 定点 2） ──
  await run(["ST-AN-01"], "拖框创建区域并可编辑删除", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 进入区域创建工具 → 十字光标拖框（避开两表区域）
    await page.locator('[data-testid="tool-new-area"]').click();
    const from = await canvasPoint(page, { x: 600, y: 400 });
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move(from.x + 300, from.y + 200, { steps: 5 });
    await page.mouse.up();

    // 松开落账：Inspector 区域面板 + PUT areas==1 + 工具复位
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 1, "拖框松开后必须落账 1 个区域");
    const area = state.lastPutBody.diagram.areas[0];
    assert.equal(area.name, "未命名区域", "区域默认名必须为 未命名区域");
    assert.equal(area.width, 300, "区域宽度必须等于拖框宽度");
    await page.locator('[data-testid="tool-new-area"]:not(.cdb-is-active)').waitFor();

    // Inspector 改名 → blur 落账新名称（受控输入：blur 才写回 store）
    await page.locator('[data-testid="inspector-area-name"]').fill("核心域");
    await page.locator('[data-testid="inspector-title"]').click(); // blur
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.[0]?.name, "核心域", "改名后必须落账");

    // 选中态按 Delete 键删除 → 落账 0 个
    await page.keyboard.press("Delete");
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 0, "Delete 键删除后必须落账 0 个区域");
  });

  // ─── ST-AN-02：点击放置便签 → Inspector 编辑内容 → 按钮删除（p0-fix 定点 2） ──
  await run(["ST-AN-02"], "点击放置便签并可编辑删除", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 进入便签创建工具 → 点击画布放置（避开两表与 ST-AN-01 区域坐标）
    await page.locator('[data-testid="tool-new-note"]').click();
    const p = await canvasPoint(page, { x: 1000, y: 500 });
    await page.mouse.click(p.x, p.y);

    // 松开落账：Inspector 便签面板 + PUT notes==1
    await page.locator('[data-testid="inspector-note-form"]:visible').waitFor();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.notes?.length, 1, "点击后必须落账 1 个便签");

    // Inspector 编辑内容 → blur 落账（受控输入：blur 才写回 store）
    await page.locator('[data-testid="inspector-note-content"]').fill("这是一张便签");
    await page.locator('[data-testid="inspector-title"]').click(); // blur
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.notes?.[0]?.content, "这是一张便签", "编辑后必须落账内容");

    // 按钮删除 → 落账 0 个
    await page.locator('[data-testid="btn-delete-note"]').click();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.notes?.length, 0, "按钮删除后必须落账 0 个便签");
  });

  // ─── ST-CR-NOTE-01：便签拖动落账 + 编辑不丢字（redesign-listview-type-length-canvas-fix） ──
  await run(["ST-CR-NOTE-01"], "便签拖动落账与编辑不丢字", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 点击放置便签（避开两表与右侧 Inspector 覆盖区——拖动时 Inspector 已打开，
    // 落点 x≥950 会被面板盖住导致 mousedown 不到画布）
    await page.locator('[data-testid="tool-new-note"]').click();
    const p = await canvasPoint(page, { x: 700, y: 500 });
    await page.mouse.click(p.x, p.y);
    await page.locator('[data-testid="inspector-note-form"]:visible').waitFor();
    await waitSaved(page);
    const placed = state.lastPutBody?.diagram?.notes?.[0];
    assert.equal(state.lastPutBody?.diagram?.notes?.length, 1, "放置后必须落账 1 个便签");

    // mousedown 便签中心（180×100 命中矩形）→ 拖动 ≥60px → mouseup；mousemove 期间不得产生 PUT
    const putsBeforeDrag = state.putCalls;
    await page.mouse.move(p.x + 90, p.y + 50);
    await page.mouse.down();
    await page.mouse.move(p.x + 90 + 80, p.y + 50 + 60, { steps: 6 });
    assert.equal(state.putCalls, putsBeforeDrag, "拖动 mousemove 期间不得触发 PUT（视觉层跟随）");
    await page.mouse.up();
    await waitSaved(page);
    const dragged = state.lastPutBody?.diagram?.notes?.[0];
    assert.ok(Math.abs(dragged.x - placed.x) >= 50, `便签 x 必须随拖动更新（${placed.x} → ${dragged.x}）`);
    assert.ok(Math.abs(dragged.y - placed.y) >= 40, `便签 y 必须随拖动更新（${placed.y} → ${dragged.y}）`);

    // Inspector 内容连续键入（受控输入：击键不落账、不丢字）→ blur 落账
    const content = page.locator('[data-testid="inspector-note-content"]');
    await content.click();
    await content.pressSequentially("release-note-v1", { delay: 20 });
    assert.equal(await content.inputValue(), "release-note-v1", "连续键入后输入框值必须完整（不丢字）");
    await page.locator('[data-testid="inspector-title"]').click(); // blur
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.notes?.[0]?.content, "release-note-v1", "blur 后必须落账便签内容");
  });

  // ─── ST-CR-AREA-01：区域拖动落账（redesign-listview-type-length-canvas-fix） ──
  await run(["ST-CR-AREA-01"], "区域拖动落账", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 拖框创建区域（避开两表）
    await page.locator('[data-testid="tool-new-area"]').click();
    const from = await canvasPoint(page, { x: 600, y: 400 });
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move(from.x + 300, from.y + 200, { steps: 5 });
    await page.mouse.up();
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    await waitSaved(page);
    const created = state.lastPutBody?.diagram?.areas?.[0];
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 1, "拖框松开后必须落账 1 个区域");

    // mousedown 区域中心 → 拖动 ≥60px → mouseup 落账 x/y（宽高不变）
    const center = await canvasPoint(page, { x: created.x + created.width / 2, y: created.y + created.height / 2 });
    const putsBeforeDrag = state.putCalls;
    await page.mouse.move(center.x, center.y);
    await page.mouse.down();
    await page.mouse.move(center.x + 90, center.y + 70, { steps: 6 });
    assert.equal(state.putCalls, putsBeforeDrag, "拖动 mousemove 期间不得触发 PUT");
    await page.mouse.up();
    await waitSaved(page);
    const dragged = state.lastPutBody?.diagram?.areas?.[0];
    assert.ok(Math.abs(dragged.x - created.x) >= 50, `区域 x 必须随拖动更新（${created.x} → ${dragged.x}）`);
    assert.ok(Math.abs(dragged.y - created.y) >= 40, `区域 y 必须随拖动更新（${created.y} → ${dragged.y}）`);
    assert.equal(dragged.width, created.width, "拖动后区域宽度必须不变");
    assert.equal(dragged.height, created.height, "拖动后区域高度必须不变");
    // Inspector 区域表单仍选中
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
  });

  // ─── ST-CR-TAG-01：Inspector 字段 tag 受控输入（redesign-listview-type-length-canvas-fix） ──
  await run(["ST-CR-TAG-01"], "字段 tag 连续键入不丢字", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 选中表 → 点字段卡 → Inspector 字段表单（tag 输入通路）
    // （dispatchEvent：字段卡中心是名称输入框，其 click stopPropagation，直接在 article 上派发）
    const f = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(f.x, f.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.locator('[data-testid^="field-row-"]').first().dispatchEvent("click");
    const tag = page.locator('[data-testid="inspector-field-tag"]');
    await tag.waitFor();

    // 逐字键入（每击键后值单调增长，不被重建打断）
    await tag.click();
    let prev = 0;
    for (const ch of "finance-core") {
      await tag.press(ch);
      const len = (await tag.inputValue()).length;
      assert.ok(len === prev + 1, `击键后 tag 值必须单调增长（${prev} → ${len}）`);
      prev = len;
    }
    assert.equal(await tag.inputValue(), "finance-core", "连续键入后 tag 必须完整（不丢字）");
    // blur 落账
    await page.locator('[data-testid="inspector-title"]').click();
    await waitSaved(page);
    const table1 = state.lastPutBody?.diagram?.tables?.find(t => t.name === "table_1");
    assert.equal(table1?.fields?.[0]?.tag, "finance-core", "blur 后必须落账 fields[0].tag");
    // 再次聚焦时值保持
    await tag.click();
    assert.equal(await tag.inputValue(), "finance-core", "落账后 tag 值必须保持");
  });

  // ─── ST-LV-01：ListView 表树+单表网格（listview-tree-master-detail） ──
  await run(["ST-LV-01"], "ListView 表树+单表网格", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 打开 ListView → 左树出两表节点（表名 + 字段数），首表默认选中；右网格仅 table_1 字段行
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();
    const n1 = page.locator('[data-testid="list-tree-node-table_1"]');
    const n2 = page.locator('[data-testid="list-tree-node-table_2"]');
    await n1.waitFor();
    await n2.waitFor();
    assert.match(await n1.textContent(), /table_1/);
    assert.match(await n1.textContent(), /1 字段/, "树节点必须显示表名 + 字段数");
    assert.match(await n2.textContent(), /table_2/);
    assert.equal(await page.locator('[data-testid="list-tree-node-table_1"].is-active').count(), 1, "首表必须默认选中高亮");
    // 组头行锚点已移除（list-view-group-* 契约废弃）
    assert.equal(await page.locator('[data-testid^="list-view-group-"]').count(), 0, "组头行锚点必须已移除");
    // 10 列编辑网格表头 + 右网格仅当前表字段行
    assert.equal(await page.locator('[data-testid="list-view-table"] thead th').count(), 10, "编辑网格必须为 10 列");
    assert.equal(await page.locator('tr[data-testid^="list-view-row-"]').count(), 1, "右网格只渲染当前选中表的字段行");
    const row1 = page.locator('tr[data-testid^="list-view-row-table_1-"]');
    await row1.waitFor();
    await row1.locator('[data-testid^="list-field-name-"]').waitFor();
    await row1.locator('select[data-testid^="list-field-type-"]').waitFor();
    await row1.locator('[data-testid^="list-field-pk-"]').waitFor();

    // 点树节点 table_2 → 右网格切换 → 切回 Canvas 验证 Inspector 同步
    await n2.click();
    await page.locator('[data-testid="list-tree-node-table_2"].is-active').waitFor();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_2（1 字段）/, "切表后网格标题必须跟随");
    assert.equal(await page.locator('tr[data-testid^="list-view-row-table_2-"]').count(), 1, "右网格必须切换为 table_2 字段行");
    await page.locator('[data-testid="btn-view-canvas"]').click();
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    assert.ok(
      (await page.locator('[data-testid="inspector-table-name"]').inputValue()) === "table_2",
      "点树节点后 Inspector 必须显示 table_2"
    );
    // 树导航不落账
    assert.ok(state.lastPutBody, "树导航不应产生额外 PUT（lastPutBody 仍为建表时的快照）");
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 2, `树导航不得改动落账数据（actual=${state.lastPutBody?.diagram?.tables?.length}, tables=${JSON.stringify(state.lastPutBody?.diagram?.tables?.map(t => t.name))}, ops=${JSON.stringify(state.sentOps?.map(o => o?.type))}`);
  });

  // ─── ST-SP-LIST-01：列表视图全屏渲染（网格定位 + 层叠遮挡 + 编辑网格回归） ──
  await run(["ST-SP-LIST-01"], "列表视图全屏渲染与网格定位", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);

    // 空表：面板也必须占据主区行（grid-row: 2），展示空态而非布局塌陷
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    let box = await page.locator('[data-testid="list-view-panel"]').boundingBox();
    const appbar = await page.locator('[data-testid="app-bar"]').boundingBox();
    assert.ok(box && box.height > 300, `空表时列表面板必须占据主区行（实际高 ${box?.height}px）`);
    assert.ok(
      box.y <= appbar.y + appbar.height + 16,
      `面板必须紧贴 AppBar 下方（panel.y=${box.y}, appbar.bottom=${appbar.y + appbar.height}）`
    );
    // fix-pg-types-listview-zindex-lock-engine: 绘制层叠断言——几何尺寸无法发现「被 aurora
    // 背景层盖死」；面板中心点 elementFromPoint 命中的最上层元素必须位于面板内
    const hitPanel = await page.evaluate(() => {
      const panel = document.querySelector('[data-testid="list-view-panel"]');
      const r = panel.getBoundingClientRect();
      const el = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
      return el ? !!el.closest('[data-testid="list-view-panel"]') : false;
    });
    assert.ok(hitPanel, "面板中心点命中的最上层元素必须位于面板内（不得被 .cdb-aurora 背景层遮挡）");
    await page.locator('[data-testid="list-view-toolbar"]:visible').waitFor();
    await page.locator('[data-testid="list-view-empty"]:visible').waitFor();

    // 切回画布建两张表 → 再切列表：组头真实可见 + 10 列编辑网格 + 双击字段行跳回画布选中表
    await page.locator('[data-testid="btn-view-canvas"]').click();
    await createTwoTables(page);
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="list-view-panel"]:visible').waitFor();
    box = await page.locator('[data-testid="list-view-panel"]').boundingBox();
    assert.ok(box && box.height > 300, `有表时列表面板必须占据主区行（实际高 ${box?.height}px）`);
    await page.locator('[data-testid="list-tree-node-table_1"]:visible').waitFor();
    await page.locator('[data-testid="list-tree-node-table_2"]:visible').waitFor();
    // 树+单表网格：组头行锚点已移除，右网格仅当前表字段行
    assert.equal(await page.locator('[data-testid^="list-view-group-"]').count(), 0, "组头行锚点必须已移除");
    assert.equal(await page.locator('tr[data-testid^="list-view-row-"]').count(), 1, "右网格只渲染当前选中表的字段行");
    assert.equal(await page.locator('[data-testid="list-view-table"] thead th').count(), 10, "编辑网格必须为 10 列");

    // 树切换 table_2 → 右网格出 table_2 字段行 → 双击字段行跳回画布选中该表
    await page.locator('[data-testid="list-tree-node-table_2"]').click();
    await page.locator('[data-testid="list-tree-node-table_2"].is-active').waitFor();
    const row2 = page.locator('tr[data-testid^="list-view-row-table_2-"]');
    await row2.waitFor({ state: "visible" });
    await row2.dblclick();
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    assert.equal(
      await page.locator('[data-testid="inspector-table-name"]').inputValue(),
      "table_2",
      "双击字段行必须跳回画布并选中该表"
    );
  });

  // ─── ST-SP-LIST-02：列表视图行内编辑落账（PDManer 式字段明细网格，PG 引擎） ──
  await run(["ST-SP-LIST-02"], "列表视图行内编辑落账", async page => {
    const state = await installApi(page, { diagramDatabase: "postgresql" });
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);
    const table1Fields = () => state.lastPutBody?.diagram?.tables?.find(t => t.name === "table_1")?.fields;

    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();
    // 规格路径：左树单击 table_1 节点选表（默认首表即 table_1，点击验证树交互不打断选中链路）
    await page.locator('[data-testid="list-tree-node-table_1"]').click();
    await page.locator('[data-testid="list-tree-node-table_1"].is-active').waitFor();
    // （first() 锚定首行：增字段后 table_1 有 2 行，后续操作均作用于首行）
    const row1 = page.locator('tr[data-testid^="list-view-row-table_1-"]').first();
    await row1.waitFor();

    // 类型下拉选项 = PG 基类型清单（无 VARCHAR(255) 整串项）
    const typeOpts = await row1.locator('select[data-testid^="list-field-type-"] option').allTextContents();
    for (const t of ["UUID", "INT", "BIGINT", "SERIAL", "VARCHAR", "TEXT", "BOOLEAN", "TIMESTAMP", "NUMERIC"]) {
      assert.ok(typeOpts.includes(t), `PG 基类型清单必须含 ${t}`);
    }
    assert.ok(!typeOpts.some(o => o.includes("(")), "清单必须为裸基类型，不得含参数化整串");

    // 点击字段行 → 选中高亮
    await row1.click();
    await page.locator('tr[data-testid^="list-view-row-table_1-"].is-selected').waitFor();

    // 行内改字段代码 → blur 落账
    const nameInput = row1.locator('[data-testid^="list-field-name-"]');
    await nameInput.fill("order_id");
    await page.locator('[data-testid="list-view-toolbar"]').click(); // blur
    await waitSaved(page);
    assert.equal(table1Fields()?.[0]?.name, "order_id", "字段代码编辑必须落账 fields[0].name");

    // 类型切 VARCHAR（即改即账合成 VARCHAR(255)）→ 长度输 32 → blur 落账 VARCHAR(32)
    await row1.locator('select[data-testid^="list-field-type-"]').selectOption("VARCHAR");
    await waitSaved(page);
    assert.equal(table1Fields()?.[0]?.type_, "VARCHAR(255)", "切 VARCHAR 必须按缺省长度合成 VARCHAR(255)");
    const lenInput = row1.locator('[data-testid^="list-field-len-"]');
    assert.equal(await lenInput.isDisabled(), false, "VARCHAR 时长度列必须可编辑");
    await lenInput.fill("32");
    await page.locator('[data-testid="list-view-toolbar"]').click(); // blur
    await waitSaved(page);
    assert.equal(table1Fields()?.[0]?.type_, "VARCHAR(32)", "长度编辑必须合成 VARCHAR(32)（core-01a §2.2）");

    // 勾选「不为空」→ 即改即账
    await row1.locator('[data-testid^="list-field-nn-"]').check();
    await waitSaved(page);
    assert.equal(table1Fields()?.[0]?.not_null, true, "勾选不为空必须落账");

    // 增字段（作用于选中表）→ fields 长度 2
    await page.locator('[data-testid="list-add-field"]').click();
    await waitSaved(page);
    assert.equal(table1Fields()?.length, 2, "增字段后必须落账 2 个字段");
    assert.equal(table1Fields()?.[1]?.name, "new_field", "新字段默认名必须为 new_field");

    // 选中首行 → 下移 → 字段顺序变化
    await row1.click();
    await page.locator('[data-testid="list-move-down"]').click();
    await waitSaved(page);
    assert.equal(table1Fields()?.[0]?.name, "new_field", "下移后 new_field 必须排到首位");

    // 删字段（选中首行 new_field）→ 长度回落 1 → Ctrl+Z 撤销恢复
    await page.locator('tr[data-testid^="list-view-row-table_1-"]').first().click();
    await page.locator('[data-testid="list-del-field"]').click();
    await waitSaved(page);
    assert.equal(table1Fields()?.length, 1, "删字段后必须落账 1 个字段");
    await page.keyboard.press("Control+z");
    await waitSaved(page);
    assert.equal(table1Fields()?.length, 2, "Ctrl+Z 必须恢复被删字段");
  });

  // ─── ST-SP-LIST-03：列表视图表树导航（搜索过滤 + 切表渲染 + 空态 + 删表回落） ──
  await run(["ST-SP-LIST-03"], "列表视图表树导航", async page => {
    const state = await installApi(page, { diagramDatabase: "postgresql" });
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);
    // 第三张表改名 users（Inspector blur 落账）
    await page.keyboard.press("t");
    const nameInput = page.locator('[data-testid="inspector-table-name"]');
    await nameInput.waitFor();
    await nameInput.fill("users");
    await nameInput.evaluate(el => el.blur());
    await waitSaved(page);
    assert.ok(
      state.lastPutBody?.diagram?.tables?.some(t => t.name === "users"),
      "改名 users 必须落账"
    );

    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();

    // 1. 三节点 + 首表默认选中 + 右网格仅 table_1 字段行
    await page.locator('[data-testid="list-tree-node-table_1"]:visible').waitFor();
    await page.locator('[data-testid="list-tree-node-table_2"]:visible').waitFor();
    await page.locator('[data-testid="list-tree-node-users"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid="list-tree-node-table_1"].is-active').count(), 1, "首表必须默认选中高亮");
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_1（1 字段）/);
    assert.equal(await page.locator('tr[data-testid^="list-view-row-"]').count(), 1, "右网格只渲染当前表字段行");

    // 2. 搜索 user → 只剩 users 节点；点击 → 网格切换为 users 字段明细
    await page.locator('[data-testid="list-tree-search"]').fill("user");
    assert.equal(await page.locator('[data-testid^="list-tree-node-"]').count(), 1, "搜索 user 后树必须只剩 users 节点");
    await page.locator('[data-testid="list-tree-node-users"]').click();
    await page.locator('[data-testid="list-tree-node-users"].is-active').waitFor();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /users（1 字段）/, "点击 users 节点后网格必须切换");

    // 3. 选中 users 字段行 → 清空搜索恢复 3 节点 → 切 table_2 → 字段行选中态清空（删/移禁用）
    await page.locator('tr[data-testid^="list-view-row-users-"]').first().click();
    assert.equal(await page.locator('[data-testid="list-del-field"]').isDisabled(), false, "选中行后删字段必须可用");
    await page.locator('[data-testid="list-tree-search"]').fill("");
    assert.equal(await page.locator('[data-testid^="list-tree-node-"]').count(), 3, "清空搜索必须恢复 3 节点");
    await page.locator('[data-testid="list-tree-node-table_2"]').click();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_2（1 字段）/);
    assert.equal(await page.locator('[data-testid="list-del-field"]').isDisabled(), true, "切表后字段行选中态必须清空（删字段禁用）");
    assert.equal(await page.locator('[data-testid="list-move-up"]').isDisabled(), true, "切表后上移必须禁用");

    // 4. 无命中关键字 → 树内空态；右网格保持当前表
    await page.locator('[data-testid="list-tree-search"]').fill("zzz");
    await page.locator('[data-testid="list-tree-empty"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid^="list-tree-node-"]').count(), 0, "无命中时树节点必须为 0");
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_2/, "搜索无命中时右网格必须保持当前表");
    await page.locator('[data-testid="list-tree-search"]').fill("");
    await page.locator('[data-testid="list-tree-node-table_1"]:visible').waitFor();

    // 导航全程不落账（最后一次 PUT 仍是改名 users 的 3 表快照）
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, `树导航不得改动落账数据（actual=${state.lastPutBody?.diagram?.tables?.length}, tables=${JSON.stringify(state.lastPutBody?.diagram?.tables?.map(t => t.name))}, ops=${JSON.stringify(state.sentOps?.map(o => o?.type))}`);

    // 5. 双击树节点跳画布选中该表 → 删除当前表（table_2）→ 重进列表视图 → 回落首表
    await page.locator('[data-testid="list-tree-node-table_2"]').dblclick();
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid="inspector-table-name"]').inputValue(), "table_2", "双击树节点必须跳画布并选中该表");
    await page.locator('[data-testid="btn-delete-table"]').click();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 2, "删表必须落账（剩 2 表）");
    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid="list-tree-node-table_1"].is-active').count(), 1, "当前表被删后必须回落首张剩余表");
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_1（1 字段）/);
  });

  // ─── ST-DB-02：引擎创建后锁定 + PG 清单内容（fix-pg-types-listview-zindex-lock-engine） ──
  await run(["ST-DB-02"], "引擎锁定/PG 清单/UUID 无占位重复", async page => {
    const state = await installApi(page, { diagramDatabase: "postgresql" });
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 引擎创建后锁定：AppBar 下拉显示 postgresql 且禁用（只读标识，不产生切换请求）
    const dbSel = page.locator('[data-testid="app-db-select"]');
    assert.equal(await dbSel.inputValue(), "postgresql", "AppBar 引擎下拉必须显示创建时选定的引擎");
    assert.equal(await dbSel.isDisabled(), true, "房间编辑器内引擎下拉必须禁用（引擎创建后锁定）");

    // Inspector 类型清单 = PG 基类型清单：含 UUID/BOOLEAN/SERIAL，无 DATETIME、无参数化整串项
    const f = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(f.x, f.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    const typeSel = page.locator('[data-testid="inspector-table-form"] select[data-testid^="type-"]').first();
    await typeSel.waitFor();
    const opts = await typeSel.locator("option").allTextContents();
    for (const t of ["UUID", "INT", "BIGINT", "SERIAL", "VARCHAR", "TEXT", "BOOLEAN", "TIMESTAMP", "NUMERIC"]) {
      assert.ok(opts.includes(t), `PG 基类型清单必须含 ${t}`);
    }
    assert.ok(!opts.includes("DATETIME"), "PG 清单不得含 DATETIME");
    assert.ok(!opts.some(o => o.includes("(")), "清单必须为裸基类型，不得含 VARCHAR(255)/NUMERIC(10,2) 整串项");
    // 当前类型基类型 UUID 已在清单内 → 占位兜底不再触发，UUID 不得重复（修复前出现两个 UUID）
    assert.equal(opts.filter(o => o === "UUID").length, 1, "UUID 已在清单内，不得出现占位重复项");
  });

  // ─── ST-CR-02：拖表跟手 + 松手 GRID_SIZE=20 吸附 ──────────────────────────
  await run(["ST-CR-02"], "拖表过程连线跟手；松手吸附 20 网格", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 先建一条关系（点击两点直接落账，p0-fix 定点 3），再 Esc 退出关系工具
    await page.keyboard.press("r");
    const p1 = await canvasPoint(page, TABLE1_FIELD);
    await page.mouse.click(p1.x, p1.y);
    const p2 = await canvasPoint(page, TABLE2_FIELD);
    await page.mouse.click(p2.x, p2.y);
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
