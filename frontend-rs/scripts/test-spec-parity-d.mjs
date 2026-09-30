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
//   ST-PB-05         Idle 字段连接点拖连（无需关系工具）
//   ST-CR-MULTI-01   框选多表后整组拖动
//   ST-PB-11         点击关系线选中后 hl 探针暴露 sel_ref_id（fix-issues-42-44 / #44）
// fix-issue-47-relation-hit-precision-and-highlight 新增：
//   ST-PB-13         点击关系线高亮关系线与两端表（R-HL-REF-01~03）
//   ST-PB-14         点击端点记号附近 10px 热区选中关系（fix-relation-mouse-hit-precision / R-HIT-06）
// p0-fix 定点 3 新增：
//   ST-PB-03         点击连线选中 + Inspector 详情/删除（不弹详情模态）
//   ST-PB-04         选中连线 Delete / Backspace 双键删除落账 0 条
//   ST-AN-01         拖框创建区域 → Inspector 编辑 → Delete 键删除落账 0 个
//   ST-AN-02         点击放置便签 → Inspector 编辑内容 → 按钮删除落账 0 个
//   ST-CR-NOTE-01    便签拖动落账（mousemove 不 PUT）+ 内容连续键入不丢字
//   ST-CR-AREA-01    区域拖动落账（宽高不变）
//   ST-CR-AREA-02    区域 resize 手柄 + Inspector 宽高双向（fix-open-issues-26-33 / #27）
//   ST-CR-TAG-01     Inspector 字段 tag 受控输入连续键入不丢字
//   ST-LV-01         ListView 表树+单表网格（树节点选中表；行内编辑控件；组头锚点移除）
//   ST-SP-LIST-01    列表视图全屏渲染（网格定位 + 层叠遮挡 + 树+单表 10 列编辑网格）
//   ST-SP-LIST-02    列表视图行内编辑落账（名称/类型长度/勾选/增删移动/撤销）
//   ST-SP-LIST-03    列表视图表树导航（搜索过滤 + 切表渲染 + 空态 + 删表回落）
//   ST-SP-LIST-GROUP-01 列表视图按 Area 分组（几何包含 + 折叠/搜索自动展开，fix-issue-50）
//   ST-SP-LIST-REL-01   列表视图关联表摘要 + 树节点关联高亮/非关联弱化（fix-issue-50）
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
//   ST-S04-UI-17     房间卡片重命名（fix-issue-45-room-rename）：owner 入口 + 模态 + PATCH 落账 + 即时一致
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
    stdio: ["ignore", "ignore", "ignore"], // serve: 避免 pipe 无人读堵死
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
      // fix-open-issues-26-33（ST-CR-AREA-02）：persistGet 时回放最近一次 PUT 落账文档
      // （刷新持久性断言；缺省保持空图，既有用例回归不破）
      const persisted = options.persistGet ? state.lastPutBody?.diagram : null;
      // fix-open-issues-26-33（ST-PB-10 只读段）：presetDiagram 预置图文档（只读房间造图用）
      const preset = options.presetDiagram ?? null;
      return response(route, 200, {
        code: 0, request_id: "load-diagram",
        // fix-pg-types-listview-zindex-lock-engine: diagramDatabase 透传（缺省 null = Generic 缺省，回归不破）
        data: preset
          ? { id: "diagram-new", name: "架构评审室", database: options.diagramDatabase ?? null, revision: diagramRev, ...preset }
          : persisted
            ? { id: "diagram-new", name: "架构评审室", database: options.diagramDatabase ?? null, revision: diagramRev, ...persisted }
            : { id: "diagram-new", name: "架构评审室", database: options.diagramDatabase ?? null, revision: diagramRev, tables: [], references: [], areas: [], notes: [] },
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
    // fix-issue-45-room-rename（issue #45，ST-S04-UI-17）：PATCH /rooms/{id} 重命名
    if (url.pathname.startsWith("/api/v1/rooms/") && request.method() === "PATCH") {
      const roomId = url.pathname.split("/")[4];
      const body = request.postDataJSON();
      state.renamedRooms = state.renamedRooms ?? [];
      state.renamedRooms.push({ id: roomId, name: body.name });
      const trimmed = (body.name ?? "").trim();
      if (!trimmed || trimmed.length > 64) {
        return response(route, 422, { code: "VALIDATION_ERROR", message: "房间名称长度 1-64" });
      }
      return response(route, 200, {
        id: roomId, name: trimmed, diagramId: "diagram-own", ownerId: "owner-1",
        diagramTitle: "我的评审室", myRole: "owner", memberCount: 1,
      });
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

// 画布坐标（默认 pan0 zoom1；表 i 落位 x=180+i*55, y=145+i*35；表头 43 / 字段行 35；表宽 230）
const TABLE1_FIELD = { x: 220, y: 205 }; // table_1 首字段行（避开 table_2 x 重叠区；点击选取用）
const TABLE2_FIELD = { x: 295, y: 240 }; // table_2 首字段行
const TABLE2_HEADER = { x: 350, y: 201.5 }; // table_2 表头中心（rev 命中优先 table_2）
// #3：拖连仅左右连接点（FIELD_PORT_HIT_RADIUS=10）；行中心会走点击选取而非 rubber-band
const TABLE1_PORT_END = { x: 410, y: 205 }; // table_1 右侧 port（180+230）
// 松手落点走 hit_test_field（字段行矩形），table_2 拖开 +300,+100 后首字段行中心
const TABLE2_FIELD_AFTER_NUDGE = { x: TABLE2_FIELD.x + 300, y: TABLE2_FIELD.y + 100 };

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
      await page.locator('[data-testid="tool-relationship-create"].cdb-is-active').count(), 0,
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
    await page.locator('[data-testid="tool-relationship-create"].cdb-is-active').waitFor();
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
      await page.locator('[data-testid="tool-relationship-create"].cdb-is-active').count(), 0,
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
    for (const testid of ["btn-more-menu", "tool-add-table", "tool-relationship-create"]) {
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

    // 先把 table_2 拖开，避免重叠遮挡目标左侧 port
    const header = await canvasPoint(page, TABLE2_HEADER);
    await page.mouse.move(header.x, header.y);
    await page.mouse.down();
    await page.mouse.move(header.x + 300, header.y + 100, { steps: 5 });
    await page.mouse.up();
    await waitSaved(page);
    await waitForCanvasStable(page);

    await page.keyboard.press("r");
    await page.locator('[data-testid="rel-tool-hint"]:visible').waitFor();
    await waitForCanvasStable(page);
    // #3：必须从字段左右连接点起拖（非整行中心）
    const from = await canvasPoint(page, TABLE1_PORT_END);
    const to = await canvasPoint(page, TABLE2_FIELD_AFTER_NUDGE);
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

  // ─── ST-PB-05：Idle 下字段连接点拖连（无需关系工具，fix-remote-github-issues #3） ──
  await run(["ST-PB-05"], "不点关系工具即可字段拖连", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    const header = await canvasPoint(page, TABLE2_HEADER);
    await page.mouse.move(header.x, header.y);
    await page.mouse.down();
    await page.mouse.move(header.x + 300, header.y + 100, { steps: 5 });
    await page.mouse.up();
    await waitSaved(page);
    await waitForCanvasStable(page);

    // 不按 R、不点 tool-relationship-create
    assert.equal(await page.locator('[data-testid="rel-tool-hint"]').count(), 0, "Idle 下不得出现关系工具提示");
    const from = await canvasPoint(page, TABLE1_PORT_END);
    const to = await canvasPoint(page, TABLE2_FIELD_AFTER_NUDGE);
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move((from.x + to.x) / 2, (from.y + to.y) / 2, { steps: 4 });
    const rubber = page.locator('[data-testid="rel-rubber-band"]');
    await page.waitForTimeout(150);
    assert.equal(await rubber.getAttribute("hidden"), null, "Idle 拖连也必须出现 rubber-band");
    assert.notEqual((await rubber.getAttribute("d")) ?? "", "", "rubber-band 必须有路径");
    await page.mouse.move(to.x, to.y, { steps: 2 });
    await page.mouse.up();

    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 1, "Idle 拖连松开后必须落账 1 条关系");
  });

  // ─── ST-CR-MULTI-01：框选 ≥2 表后整组拖动（fix-remote-github-issues #5） ──
  await run(["ST-CR-MULTI-01"], "框选后多表整组拖动", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);

    // 拖开 table_2，避免默认重叠导致框选/命中歧义
    const header2 = await canvasPoint(page, TABLE2_HEADER);
    await page.mouse.move(header2.x, header2.y);
    await page.mouse.down();
    await page.mouse.move(header2.x + 300, header2.y + 100, { steps: 5 });
    await page.mouse.up();
    await waitSaved(page);
    await waitForCanvasStable(page);

    const before = state.lastPutBody?.diagram?.tables ?? [];
    const t1Before = before.find(t => t.name === "table_1");
    const t2Before = before.find(t => t.name === "table_2");
    assert.ok(t1Before && t2Before, "框选前必须已有 table_1/table_2");

    await page.locator('[data-testid="tool-marquee"]').click();
    // 框选覆盖两表（世界坐标约 170,130 → 800,420）
    const a = await canvasPoint(page, { x: 170, y: 130 });
    const b = await canvasPoint(page, { x: 800, y: 420 });
    await page.mouse.move(a.x, a.y);
    await page.mouse.down();
    await page.mouse.move(b.x, b.y, { steps: 6 });
    await page.mouse.up();
    await page.waitForTimeout(100);

    // 拖 table_1 表头 → 两表同步位移
    const h1 = await canvasPoint(page, { x: 295, y: 166.5 }); // table_1 表头中心
    await page.mouse.move(h1.x, h1.y);
    await page.mouse.down();
    await page.mouse.move(h1.x + 80, h1.y + 60, { steps: 5 });
    await page.mouse.up();
    await waitSaved(page);

    const after = state.lastPutBody?.diagram?.tables ?? [];
    const t1After = after.find(t => t.name === "table_1");
    const t2After = after.find(t => t.name === "table_2");
    assert.ok(t1After && t2After, "整组拖动后 PUT 必须仍含两表");
    assert.ok(t1After.x - t1Before.x >= 40, `table_1 x 应随拖动增加（${t1Before.x} → ${t1After.x}）`);
    assert.ok(t2After.x - t2Before.x >= 40, `table_2 x 应同步增加（${t2Before.x} → ${t2After.x}）`);
    assert.ok(t1After.y - t1Before.y >= 20, `table_1 y 应随拖动增加（${t1Before.y} → ${t1After.y}）`);
    assert.ok(t2After.y - t2Before.y >= 20, `table_2 y 应同步增加（${t2Before.y} → ${t2After.y}）`);
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

  // ─── ST-CR-AREA-02：区域 resize 手柄 + Inspector 双向（fix-open-issues-26-33 / #27） ──
  await run(["ST-CR-AREA-02"], "区域 resize 手柄 + Inspector 双向", async page => {
    const state = await installApi(page, { persistGet: true });
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
    assert.ok(created, "拖框松开后必须落账 1 个区域");

    // R-ARESZ-01：未选中不渲染/不命中手柄——先点击区域中心选中（<4px 纯点击不落账）
    const center0 = await canvasPoint(page, { x: created.x + created.width / 2, y: created.y + created.height / 2 });
    const putsBeforePick = state.putCalls;
    await page.mouse.click(center0.x, center0.y);
    assert.equal(state.putCalls, putsBeforePick, "纯点击选中不得触发 PUT");

    // 拖 SE 角手柄放大：mousemove 期间不落账，mouseup 一次写回（R-ARESZ-02）
    const se = await canvasPoint(page, { x: created.x + created.width, y: created.y + created.height });
    const putsBeforeResize = state.putCalls;
    await page.mouse.move(se.x, se.y);
    await page.mouse.down();
    await page.mouse.move(se.x + 60, se.y + 50, { steps: 6 });
    assert.equal(state.putCalls, putsBeforeResize, "resize mousemove 期间不得触发 PUT");
    await page.mouse.up();
    await waitSaved(page);
    const resized = state.lastPutBody?.diagram?.areas?.[0];
    assert.ok(resized.width >= created.width + 40, `SE 拖拽后宽度必须增大（${created.width} → ${resized.width}）`);
    assert.ok(resized.height >= created.height + 30, `SE 拖拽后高度必须增大（${created.height} → ${resized.height}）`);
    assert.equal(resized.x, created.x, "SE 拖拽 x 必须锚定不变");
    assert.equal(resized.y, created.y, "SE 拖拽 y 必须锚定不变");

    // Inspector 宽高显示画布新值（双向之 画布→表单，R-ARESZ-04）
    const wInput = page.locator('[data-testid="inspector-area-width"]');
    const hInput = page.locator('[data-testid="inspector-area-height"]');
    assert.ok(Math.abs(parseFloat(await wInput.inputValue()) - resized.width) < 1,
      `Inspector 宽度必须显示画布新值（${resized.width}）`);
    assert.ok(Math.abs(parseFloat(await hInput.inputValue()) - resized.height) < 1,
      `Inspector 高度必须显示画布新值（${resized.height}）`);

    // 表单改宽度 blur → 画布同步（双向之 表单→画布，R-ARESZ-04）
    const putsBeforeForm = state.putCalls;
    const formWidth = Math.round(resized.width) + 40;
    await wInput.fill(String(formWidth));
    await page.locator('[data-testid="inspector-title"]').click();
    await waitSaved(page);
    assert.ok(state.putCalls > putsBeforeForm, "表单 blur 必须触发落账");
    const fromForm = state.lastPutBody?.diagram?.areas?.[0];
    assert.equal(fromForm.width, formWidth, "表单宽度必须落账到画布");
    assert.equal(fromForm.height, resized.height, "表单改宽不得改高");

    // Undo 一次恢复表单修改前宽度（SetAreaRect 单条命令 = 一次撤销单元）
    await page.keyboard.press("Control+z");
    await waitSaved(page);
    const undone = state.lastPutBody?.diagram?.areas?.[0];
    assert.ok(Math.abs(undone.width - resized.width) < 1,
      `一次 Undo 必须恢复表单改前宽度（${resized.width} → ${undone.width}）`);

    // 刷新后尺寸保持（persistGet 回放最近 PUT 文档；token 在 localStorage 跨刷新存活，无需重登）
    await page.reload();
    await createRoomAndEnter(page);
    const center = await canvasPoint(page, { x: undone.x + undone.width / 2, y: undone.y + undone.height / 2 });
    await page.mouse.click(center.x, center.y);
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    const wAfter = parseFloat(await page.locator('[data-testid="inspector-area-width"]').inputValue());
    assert.ok(Math.abs(wAfter - undone.width) < 1, `刷新后宽度必须保持（${undone.width} → ${wAfter}）`);
  });

  // ─── ST-PB-10：批量线型 apply-all + 命令面板（fix-open-issues-26-33 / #28，core-01b §4.6） ──
  await run(["ST-PB-10"], "批量线型 apply-all + 命令面板", async page => {
    const state = await installApi(page, { persistGet: true });
    await login(page);
    await createRoomAndEnter(page);

    // 导入 3 表 3 FK（line_type 全默认 bezier）
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-import"]').click();
    await page.locator('[data-testid="import-drawer"]:visible').waitFor();
    await page.locator('[data-testid="import-textarea"]').fill(
      "CREATE TABLE users (id UUID PRIMARY KEY);\n" +
      "CREATE TABLE orders (id UUID PRIMARY KEY, user_id UUID REFERENCES users(id));\n" +
      "CREATE TABLE items (id UUID PRIMARY KEY, order_id UUID REFERENCES orders(id), owner_id UUID REFERENCES users(id));",
    );
    await page.locator('[data-testid="import-parse-summary"]:visible').waitFor();
    await page.locator('[data-testid="import-submit"]').click();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.length, 3, "导入必须落账 3 条关系");
    const rid0 = state.lastPutBody.diagram.references[0].id;

    // Command Palette 选中首条关系 → Inspector 关系面板（不依赖画布连线命中）
    await page.keyboard.press("Control+k");
    await page.locator(`[data-testid="palette-item-${rid0}"]`).click();
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();

    // 制造混合线型：首条改 straight（单条落账），其余保持默认 bezier
    await page.locator('[data-testid="inspector-rel-line-type"]').selectOption("straight");
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.references?.[0]?.line_type, "straight", "单条改线型必须落账");

    // apply-all → 全部 straight（单条命令事务）
    const beforeAll = state.lastPutBody?.diagram?.references ?? [];
    await page.locator('[data-testid="inspector-rel-line-type-apply-all"]').click();
    await waitSaved(page);
    const afterAll = state.lastPutBody?.diagram?.references ?? [];
    assert.ok(afterAll.every(r => r.line_type === "straight"), "apply-all 后全部关系必须为 straight");

    // 一次 Undo 恢复批量前各自值（rid0=straight 不变，其余恢复 apply-all 前各自原值）
    await page.keyboard.press("Control+z");
    await waitSaved(page);
    const undone = state.lastPutBody?.diagram?.references ?? [];
    assert.equal(undone.find(r => r.id === rid0)?.line_type, "straight", "Undo 不得串改单条直改的值");
    for (const b of beforeAll.filter(r => r.id !== rid0)) {
      assert.equal(
        undone.find(r => r.id === b.id)?.line_type ?? "",
        b.line_type ?? "",
        `Undo 必须恢复 ${b.id} 到批量前原值（${b.line_type ?? '""'}）`,
      );
    }

    // 再 apply-all → 刷新后保持（persistGet 回放 + Inspector 控件回显）
    await page.locator('[data-testid="inspector-rel-line-type-apply-all"]').click();
    await waitSaved(page);
    await page.reload();
    await createRoomAndEnter(page);
    await page.keyboard.press("Control+k");
    await page.locator(`[data-testid="palette-item-${rid0}"]`).click();
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    assert.equal(
      await page.locator('[data-testid="inspector-rel-line-type"]').inputValue(),
      "straight",
      "刷新后线型必须保持 straight",
    );

    // palette 命令：全部关系设为 orthogonal
    await page.keyboard.press("Escape"); // 关 Inspector 焦点干扰
    await page.keyboard.press("Control+k");
    await page.locator('[data-testid="palette-action-line-type-orthogonal"]').click();
    await waitSaved(page);
    const afterPalette = state.lastPutBody?.diagram?.references ?? [];
    assert.ok(afterPalette.every(r => r.line_type === "orthogonal"), "palette 命令后全部关系必须为 orthogonal");
  });

  // ─── ST-PB-10（只读段）：viewer 房间入口禁用 / 命令不生效 ──
  await run(["ST-PB-10"], "只读房间批量线型入口禁用", async page => {
    const presetDiagram = {
      tables: [
        { id: "t1", name: "books", x: 100, y: 100, color: "", comment: "",
          fields: [{ id: "f1", name: "id", type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
          indices: [] },
        { id: "t2", name: "authors", x: 420, y: 100, color: "", comment: "",
          fields: [{ id: "f2", name: "id", type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" },
                   { id: "f3", name: "book_id", type_: "INT", default: "", check: "", primary: false, unique: false, not_null: false, increment: false, comment: "", tag: "", dict_code: "" }],
          indices: [] },
      ],
      references: [
        { id: "r-ro", name: "", start_table_id: "t2", end_table_id: "t1", start_field_id: "f3", end_field_id: "f1",
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" },
      ],
      areas: [], notes: [],
    };
    const state = await installApi(page, {
      roomsList: [{
        id: "room-view", name: "只读评审室", diagramId: "diagram-new", diagramTitle: "只读评审室",
        myRole: "viewer", memberCount: 2, updatedAt: "2026-08-23T00:02:00Z",
      }],
      presetDiagram,
    });
    await login(page);
    await page.locator('[data-testid="room-card-room-view"]:visible').click();
    await page.locator('[data-testid="room-editor-page"]:visible').waitFor();

    // palette 选中关系（只读可选中查看）→ apply-all 按钮禁用
    await page.keyboard.press("Control+k");
    await page.locator('[data-testid="palette-item-r-ro"]').click();
    await page.locator('[data-testid="inspector-reference-form"]:visible').waitFor();
    assert.ok(
      await page.locator('[data-testid="inspector-rel-line-type-apply-all"]').isDisabled(),
      "只读房间 apply-all 必须禁用",
    );

    // palette 命令不生效（read_only 守卫）：执行后无任何 PUT
    await page.keyboard.press("Escape");
    await page.keyboard.press("Control+k");
    await page.locator('[data-testid="palette-action-line-type-orthogonal"]').click();
    await page.waitForTimeout(600);
    assert.equal(state.putCalls, 0, "只读房间批量线型命令不得产生 PUT");
  });

  // ─── ST-PE-10：选中高亮与注释可读性目视锚点（fix-open-issues-26-33 / #31 #32，
  //     core-PE UT-PE-HL-01 / UT-PE-CMT-01 的 e2e 面） ──────────────────────────
  await run(["ST-PE-10"], "选中高亮参数探针 + 双主题注释可读性锚点", async page => {
    const field = (id, name, extra = {}) => ({
      id, name, type_: "INT", default: "", check: "", primary: false, unique: false,
      not_null: false, increment: false, comment: "", tag: "", dict_code: "", ...extra,
    });
    const presetDiagram = {
      tables: [
        // 三张实色表头（蓝/绿/紫）+ 一张默认色；ta 为 ≥3 关系中心表（r1/r2 直出、经 tc 间接 r3）
        { id: "ta", name: "orders", x: 120, y: 140, color: "#3788e5", comment: "订单主表",
          fields: [field("fa1", "id", { primary: true, comment: "主键" }),
                   field("fa2", "user_id", { comment: "下单用户" })], indices: [] },
        { id: "tb", name: "users", x: 480, y: 100, color: "#19a974", comment: "用户表",
          fields: [field("fb1", "id", { primary: true, comment: "用户主键" })], indices: [] },
        { id: "tc", name: "items", x: 480, y: 360, color: "#aa8cff", comment: "订单明细",
          fields: [field("fc1", "id", { primary: true })], indices: [] },
        { id: "td", name: "audit_log", x: 880, y: 360, color: "", comment: "",
          fields: [field("fd1", "id", { primary: true })], indices: [] },
      ],
      references: [
        { id: "r1", name: "", start_table_id: "ta", end_table_id: "tb", start_field_id: "fa2", end_field_id: "fb1",
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" },
        { id: "r2", name: "", start_table_id: "ta", end_table_id: "tc", start_field_id: "fa2", end_field_id: "fc1",
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" },
        { id: "r3", name: "", start_table_id: "tc", end_table_id: "td", start_field_id: "fc1", end_field_id: "fd1",
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" },
      ],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    // 首帧渲染后探针可用
    await page.waitForFunction(() => !!window.__cdb_hl_probe, null, { timeout: 8_000 });
    const probe = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe));

    // GIVEN：注释显示模式 = 英文名+注释（默认 NameComment，按钮文案锚点）
    const cmtBtnText = await page.locator('[data-testid="canvas-comment-display"]').innerText();
    assert.ok(cmtBtnText.includes("英文名+注释"), `注释模式必须为英文名+注释（实际：${cmtBtnText}）`);

    // 无选中基线：全部默认参数
    let p = await probe();
    assert.equal(p.any_sel, false, "未选中时 any_sel 必须为 false");
    assert.equal(p.table_alpha_min, 1, "未选中时全表 alpha=1");
    assert.equal(p.rel_width_scale_max, 1, "未选中时全线默认线宽");
    assert.equal(p.related_emphasis, false, "未选中时不得有相关强调线（fix-31）");

    // WHEN：点击选中中心表 ta（表头区域，避开字段连接点）
    const head = await canvasPoint(page, { x: 120 + 100, y: 140 + 16 });
    await page.mouse.click(head.x, head.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe).any_sel === true, null, { timeout: 4_000 },
    );

    // THEN 1：相关线 1.5× 提亮、非相关表 alpha 0.5 退背景（canvas 参数探针）
    p = await probe();
    assert.ok(Math.abs(p.rel_width_scale_max - 1.5) < 1e-9,
      `相关连线必须加粗 1.5×（实际 ${p.rel_width_scale_max}）`);
    assert.ok(Math.abs(p.table_alpha_min - 0.5) < 1e-9,
      `非相关表必须降透明度至 0.5（实际 ${p.table_alpha_min}）`);
    assert.equal(p.related_emphasis, true,
      "相关连线必须以选中色族强调渲染（fix-31：palette.selected 主色 + selected_soft 光晕）");

    // THEN 2：三种表头色 × 亮/暗主题注释可读性——截图锚点（UT-PE-CMT-01 保证对比度参数）
    const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
    const darkShot = await canvas.screenshot();
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-theme-toggle"]').click();
    await page.waitForTimeout(250); // 等 theme effect 重绘一帧
    assert.equal(await page.locator("html").getAttribute("data-mode"), "light", "必须切到亮主题");
    const lightShot = await canvas.screenshot();
    assert.ok(!darkShot.equals(lightShot), "主题切换后画布必须重绘（亮暗调色板不同）");
    // 高亮参数不随主题漂移（同样的输入产生同样的视觉参数）
    p = await probe();
    assert.ok(Math.abs(p.rel_width_scale_max - 1.5) < 1e-9, "亮主题下相关线仍必须 1.5×");
    assert.ok(Math.abs(p.table_alpha_min - 0.5) < 1e-9, "亮主题下非相关表仍必须 0.5");
    assert.equal(p.related_emphasis, true, "亮主题下相关线仍必须选中色族强调（fix-31）");

    // THEN 3：取消选中（Escape 收拢可能开着的更多菜单 + 点击左下空旷区）→ 全图恢复默认
    await page.keyboard.press("Escape");
    const blank = await canvasPoint(page, { x: 200, y: 600 });
    await page.mouse.click(blank.x, blank.y);
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe).any_sel === false, null, { timeout: 4_000 },
    );
    p = await probe();
    assert.equal(p.table_alpha_min, 1, "取消选中后全表必须恢复 alpha=1");
    assert.equal(p.rel_width_scale_max, 1, "取消选中后全线必须恢复默认线宽");
    assert.equal(p.related_emphasis, false, "取消选中后相关强调必须复位（fix-31）");
  });

  // ─── ST-PE-09：视觉体系统一目视锚点（fix-open-issues-26-33 / #33，
  //     core-07 §15.5 / core-08 §11；UT-PE-VIS-01 的 e2e 面） ──────────────────
  await run(["ST-PE-09"], "徽章图标族 + crow's foot 端点 + ToolRail 统一 + 双主题无漂移", async page => {
    const field = (id, name, extra = {}) => ({
      id, name, type_: "INT", default: "", check: "", primary: false, unique: false,
      not_null: false, increment: false, comment: "", tag: "", dict_code: "", ...extra,
    });
    // GIVEN：≥2 张表（PK/FK/NOT NULL/UNIQUE 字段各一）+ ≥1 条关系。
    // 徽章台账：t1.id=PK+NN、t1.email=UQ、t2.id=PK+NN、t2.user_id=FK（one_to_many → end 侧）= 6 枚
    const presetDiagram = {
      tables: [
        { id: "t1", name: "users", x: 120, y: 140, color: "#3788e5", comment: "用户表",
          fields: [field("f1", "id", { primary: true, not_null: true }),
                   field("f2", "email", { unique: true, type_: "VARCHAR" })], indices: [] },
        { id: "t2", name: "orders", x: 520, y: 160, color: "#19a974", comment: "订单表",
          fields: [field("f3", "id", { primary: true, not_null: true }),
                   field("f4", "user_id", { not_null: false })], indices: [] },
      ],
      references: [
        { id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f4",
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" },
      ],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_vis_probe, null, { timeout: 8_000 });
    const probe = () => page.evaluate(() => JSON.parse(window.__cdb_vis_probe));

    // THEN 1：表卡/徽章/端点统一常量（canvas 参数探针）+ 徽章图标族计数（无字块角标）
    let p = await probe();
    assert.equal(p.badge_size, 12, "徽章统一外接尺寸 12px");
    assert.equal(p.badge_stroke, 1.5, "徽章统一描边 1.5px");
    assert.equal(p.endpoint_size, 10, "关系端点统一 10px");
    assert.equal(p.corner_radius, 8, "表卡圆角统一 8px");
    assert.equal(p.endpoint_style, "crowsfoot", "关系端点必须为 crow's foot 族");
    assert.equal(p.text_badge, false, "字块角标渲染路径必须移除");
    assert.equal(p.badges_drawn, 6, `徽章图标族计数必须为 6（实际 ${p.badges_drawn}）`);

    // THEN 2：ToolRail 图标描边/尺寸/激活态一致（DOM 断言）
    const railIconCheck = () => page.evaluate(() => {
      const svgs = [...document.querySelectorAll(".cdb-tool-rail svg")];
      const wraps = [...document.querySelectorAll(".cdb-tool-rail .cdb-icon-wrap--md")];
      const active = document.querySelector(".cdb-tool-btn.cdb-is-active");
      const activeStyle = active ? getComputedStyle(active) : null;
      return {
        svgCount: svgs.length,
        allStroke15: svgs.every(el => el.getAttribute("stroke-width") === "1.5"),
        wrapCount: wraps.length,
        allWrap20: wraps.every(el => {
          const r = el.getBoundingClientRect();
          return Math.abs(r.width - 20) < 0.5 && Math.abs(r.height - 20) < 0.5;
        }),
        hasActive: !!active,
        activeBordered: !!activeStyle && !/rgba?\(0, 0, 0, 0\)|transparent/.test(activeStyle.borderColor),
      };
    });
    // 激活态：点框选工具使其进入激活
    await page.locator('[data-testid="tool-marquee"]').click();
    let rail = await railIconCheck();
    assert.ok(rail.svgCount >= 6, `ToolRail 图标数 ≥6（实际 ${rail.svgCount}）`);
    assert.ok(rail.allStroke15, "ToolRail 全部图标描边必须为 1.5");
    assert.ok(rail.wrapCount >= 6 && rail.allWrap20, "ToolRail IconBox 必须统一 20px");
    assert.ok(rail.hasActive && rail.activeBordered, "激活态必须存在且为主色描边（非透明边框）");

    // THEN 3：暗主题截图锚点 → 切亮主题 → 参数无漂移 + 画布重绘 + ToolRail 一致
    const canvas = page.locator('[data-testid="editor-canvas-container"] canvas');
    const darkShot = await canvas.screenshot();
    await page.keyboard.press("Escape");
    await page.locator('[data-testid="btn-more-menu"]').click();
    await page.locator('[data-testid="btn-theme-toggle"]').click();
    await page.waitForTimeout(250); // 等 theme effect 重绘一帧
    assert.equal(await page.locator("html").getAttribute("data-mode"), "light", "必须切到亮主题");
    const lightShot = await canvas.screenshot();
    assert.ok(!darkShot.equals(lightShot), "主题切换后画布必须重绘（亮暗调色板不同）");
    p = await probe();
    assert.equal(p.badge_size, 12, "亮主题徽章尺寸无漂移");
    assert.equal(p.badge_stroke, 1.5, "亮主题徽章描边无漂移");
    assert.equal(p.corner_radius, 8, "亮主题圆角无漂移");
    assert.equal(p.badges_drawn, 6, "亮主题徽章计数无漂移");
    rail = await railIconCheck();
    assert.ok(rail.allStroke15 && rail.allWrap20, "亮主题 ToolRail 描边/尺寸无漂移");
    assert.ok(rail.hasActive && rail.activeBordered, "亮主题激活态规则无漂移");
  });

  // ─── ST-CR-LOD-01：显式维度切换 e2e（fix-issues-36-37 / #36，core-01 §5.12 改写） ──
  // 口径：维度由显式切换决定（快捷键 V / ToolRail / 右键菜单），不随 zoom 串档；
  // zoom 仅在表维度内驱动字号/线宽补偿。0.35 夹紧裁决同旧注记（夹紧上限优先）。
  await run(["ST-CR-LOD-01"], "显式维度切换 + 缩放不串档 + 三入口一致 + 持久", async page => {
    const field = (id, name, extra = {}) => ({
      id, name, type_: "INT", default: "", check: "", primary: false, unique: false,
      not_null: false, increment: false, comment: "", tag: "", dict_code: "", ...extra,
    });
    // 20 表（5 列 × 4 行）+ 20 关系（链 19 条 + 跨首尾 1 条，≥5 对 FK 表对）
    const tables = [];
    const references = [];
    for (let i = 0; i < 20; i++) {
      const col = i % 5;
      const row = Math.floor(i / 5);
      tables.push({
        id: `t${i}`, name: `table_${i}`, x: 80 + col * 340, y: 80 + row * 240, color: "",
        // #35：t5/t7 带中文注释（表维度注释可见性断言）
        comment: i === 5 ? "订单主表注释" : i === 7 ? "用户信息表注释" : "",
        fields: [field(`f${i}_id`, "id", { primary: true }), field(`f${i}_fk`, "ref_id")],
        indices: [],
      });
      if (i < 19) {
        references.push({
          id: `r${i}`, name: "", start_table_id: `t${i}`, end_table_id: `t${i + 1}`,
          start_field_id: `f${i}_fk`, end_field_id: `f${i + 1}_id`,
          type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT",
          color: "", line_type: "", stroke_style: "",
        });
      }
    }
    references.push({
      id: "r19", name: "", start_table_id: "t19", end_table_id: "t0",
      start_field_id: "f19_fk", end_field_id: "f0_id",
      type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT",
      color: "", line_type: "", stroke_style: "",
    });
    await installApi(page, { presetDiagram: { tables, references, areas: [], notes: [] } });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    const lodProbe = () => page.evaluate(() => JSON.parse(window.__cdb_lod_probe));
    const hlProbe = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe));
    const zoomOut = page.locator('[data-testid="btn-zoom-out"]').first();
    const zoomIn = page.locator('[data-testid="btn-zoom-in"]').first();
    const waitZoom = (cmp, timeout = 4_000) =>
      page.waitForFunction(cmp, null, { timeout });
    const dimBtn = page.locator('[data-testid="canvas-view-dimension"]');
    const railBtn = page.locator('[data-testid="tool-view-dimension"]');

    // 基线 100%：默认字段维度、详情档、默认字号线宽
    let p = await lodProbe();
    assert.equal(p.view_dimension, "field", "默认必须为字段维度（R-VIEW-DIM-01）");
    assert.equal(p.tier, "detail", "字段维度必须详情档渲染");
    assert.equal(p.lod_scale, 1, "字段维度线宽补偿必须为 1");
    assert.equal(p.font_world, 13, "字段维度表名必须为默认 13px");
    assert.equal(p.anchor_mode, "field", "字段维度关系线必须字段锚定（R-LOD-08）");
    await dimBtn.waitFor();
    assert.equal(await dimBtn.innerText(), "维度：字段", "指示器必须为「维度：字段」（R-VIEW-DIM-03）");

    // 先在 100% 点击选中 t5（i=5 → col0/row1 → 世界 (80,320)，表头中心）
    const head = await canvasPoint(page, { x: 80 + 100, y: 320 + 21 });
    await page.mouse.click(head.x, head.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe).any_sel === true, null, { timeout: 4_000, polling: 100 },
    );

    // WHEN：缩至 ≈50%（zoom-out ×3 → 0.512）——THEN 1：维度不随缩放串档（核心）
    await zoomOut.click(); await zoomOut.click(); await zoomOut.click();
    await waitZoom(() => JSON.parse(window.__cdb_lod_probe).zoom < 0.6);
    p = await lodProbe();
    assert.equal(p.view_dimension, "field", "缩到 51% 维度必须仍为字段（R-VIEW-DIM-01）");
    assert.equal(p.tier, "detail", `0.512 必须仍详情档（实际 ${p.tier}）`);
    assert.equal(p.anchor_mode, "field", "0.512 必须仍字段锚定");
    assert.equal(p.lod_scale, 1, "字段维度任意缩放不得补偿线宽");
    assert.equal(p.font_world, 13, "字段维度任意缩放不得补偿字号");

    // WHEN：快捷键 V 切表维度（R-VIEW-DIM-02 入口 ③）
    await page.keyboard.press("v");
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe).view_dimension === "table", null, { timeout: 4_000, polling: 100 },
    );
    p = await lodProbe();
    // THEN 2：表维度——拓扑渲染 + 维度内补偿 + 中文注释可见 + 表级锚定（#35 沿用）
    assert.equal(p.tier, "topology", "V 后必须表维度渲染");
    assert.equal(p.anchor_mode, "table", "表维度关系线必须表级锚定（R-LOD-08）");
    assert.ok(p.font_world * p.zoom >= 11.0,
      `表维度表名屏幕字号必须 ≥11px（${p.font_world}×${p.zoom}=${(p.font_world * p.zoom).toFixed(2)}）`);
    assert.ok(p.lod_scale >= 1.4, `表维度 0.512 线宽补偿必须 ≥1.4（实际 ${p.lod_scale}）`);
    assert.ok(p.topo_comments > 0,
      `表维度 NameComment 模式中文注释必须渲染（R-LOD-02，实际计数 ${p.topo_comments}）`);
    // THEN 3a：三入口状态一致——指示器文案 + ToolRail 激活态
    assert.equal(await dimBtn.innerText(), "维度：表", "切表维度后指示器必须为「维度：表」");
    assert.ok(await railBtn.evaluate(el => el.classList.contains("cdb-is-active")),
      "表维度下 ToolRail 维度按钮必须为激活态");
    // THEN 2b：选中 t5 保持 → 相关连线 §4.4 × R-LOD-04 叠乘加粗
    const hl = await hlProbe();
    assert.equal(hl.any_sel, true, "切换维度后选中态必须保持");
    assert.ok(hl.rel_width_scale_max >= 2.1,
      `表维度选中后相关线必须叠乘加粗 ≥2.1×（实际 ${hl.rel_width_scale_max}）`);

    // WHEN：续缩至 ≈35%（zoom-out ×2 → 0.328）——维度不变，补偿随 zoom 加深
    await zoomOut.click(); await zoomOut.click();
    await waitZoom(() => JSON.parse(window.__cdb_lod_probe).zoom < 0.4);
    p = await lodProbe();
    assert.equal(p.view_dimension, "table", "表维度内缩放不得串档");
    assert.equal(p.tier, "topology", "0.328 必须保持表维度渲染");
    assert.equal(p.font_world, 30, "0.328 表名字号必须触发夹紧上限 30px");
    assert.ok(p.lod_scale >= 2.2, `0.328 线宽补偿必须 ≥2.2（实际 ${p.lod_scale}）`);

    // WHEN：回 100%（zoom-in ×5）——维度仍表（关键新行为：zoom 不再驱动维度）
    for (let i = 0; i < 5; i++) await zoomIn.click();
    await waitZoom(() => JSON.parse(window.__cdb_lod_probe).zoom > 0.95);
    p = await lodProbe();
    assert.equal(p.view_dimension, "table", "回 100% 必须仍为表维度（缩放只负责缩放）");
    assert.equal(p.tier, "topology", "回 100% 必须保持表维度渲染");
    assert.equal(p.anchor_mode, "table", "回 100% 必须保持表级锚定");
    assert.ok(Math.abs(p.font_world - 11.0) < 0.2,
      `表维度 ≈100% 字号必须为 11/zoom ≈ 11px（实际 ${p.font_world}@zoom ${p.zoom}）`);
    assert.ok(p.lod_scale <= 1.01, "表维度 100% 线宽补偿必须回落 ≈1");

    // WHEN：ToolRail 按钮切回字段维度（R-VIEW-DIM-02 入口 ②）
    await railBtn.click();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe).view_dimension === "field", null, { timeout: 4_000, polling: 100 },
    );
    p = await lodProbe();
    assert.equal(p.tier, "detail", "ToolRail 切回必须恢复详情档");
    assert.equal(p.anchor_mode, "field", "ToolRail 切回必须恢复字段锚定");
    assert.equal(p.topo_comments, 0, "字段维度拓扑注释计数必须复位");
    assert.equal(await dimBtn.innerText(), "维度：字段", "指示器必须随 ToolRail 切换一致");
    assert.ok(!(await railBtn.evaluate(el => el.classList.contains("cdb-is-active"))),
      "字段维度下 ToolRail 维度按钮不得为激活态");

    // WHEN：右键菜单切表维度（R-VIEW-DIM-02 入口 ①）
    const blank = await canvasPoint(page, { x: 640, y: 480 });
    await page.mouse.click(blank.x, blank.y, { button: "right" });
    const ctxMenu = page.locator('[data-testid="canvas-context-menu"]:visible');
    await ctxMenu.waitFor();
    await page.locator('[data-testid="ctx-toggle-dimension"]').click();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe).view_dimension === "table", null, { timeout: 4_000, polling: 100 },
    );
    assert.equal(await dimBtn.innerText(), "维度：表", "右键菜单切换后指示器必须一致");
    assert.equal((await lodProbe()).tier, "topology", "右键菜单切换后必须表维度渲染");
    assert.equal(
      await page.locator('[data-testid="canvas-context-menu"]:visible').count(), 0,
      "维度切换后右键菜单必须关闭",
    );

    // THEN 4：持久化（R-VIEW-DIM-04）——刷新后仍为表维度
    // （token 在 localStorage 跨刷新存活，无需重登；刷新回落房间列表，重新进入）
    await page.reload();
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    p = await lodProbe();
    assert.equal(p.view_dimension, "table", "刷新后维度必须持久为表（cdb.view-dimension）");
    assert.equal(p.tier, "topology", "刷新后必须保持表维度渲染");
  });

  // ─── ST-KB-SEL-01：Ctrl+A 全选 + Delete 删除 + 撤销 e2e（fix-issues-36-37 / #37，core-01 §5.13） ──
  await run(["ST-KB-SEL-01"], "全选图元 + 删除 + 撤销/重做 + 输入框豁免", async page => {
    const field = (id, extra = {}) => ({
      id, name: "id", type_: "INT", default: "", check: "", primary: true, unique: false,
      not_null: true, increment: false, comment: "", tag: "", dict_code: "", ...extra,
    });
    const mkTable = (id, x) => ({
      id, name: `tbl_${id}`, x, y: 100, color: "", comment: "",
      fields: [field(`${id}_f1`)], indices: [],
    });
    const mkRef = (id, from, to) => ({
      id, name: "", start_table_id: from, end_table_id: to,
      start_field_id: `${from}_f1`, end_field_id: `${to}_f1`,
      type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT",
      color: "", line_type: "", stroke_style: "",
    });
    await installApi(page, {
      presetDiagram: {
        tables: [mkTable("t1", 60), mkTable("t2", 380), mkTable("t3", 700)],
        references: [mkRef("r1", "t1", "t2"), mkRef("r2", "t2", "t3")],
        areas: [{ id: "a1", x: 40, y: 60, width: 900, height: 300, color: "", name: "区域一" }],
        notes: [{ id: "n1", x: 80, y: 420, content: "便签一", color: "" }],
      },
    });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    const lodProbe = () => page.evaluate(() => JSON.parse(window.__cdb_lod_probe));
    const hlProbe = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe));
    const counts = p => [p.tables_n, p.refs_n, p.notes_n, p.areas_n];
    const waitCounts = (want, timeout = 4_000) => page.waitForFunction(
      w => {
        const pr = window.__cdb_lod_probe && JSON.parse(window.__cdb_lod_probe);
        return pr && [pr.tables_n, pr.refs_n, pr.notes_n, pr.areas_n].join(",") === w.join(",");
      },
      want, { timeout, polling: 100 },
    );

    // GIVEN：基线计数 3 表 / 2 关系 / 1 便签 / 1 区域
    let p = await lodProbe();
    assert.deepEqual(counts(p), [3, 2, 1, 1], `基线计数必须 3/2/1/1（实际 ${counts(p)}）`);

    // 画布空白点击获得焦点（并清空任何既有选中）
    const blank = await canvasPoint(page, { x: 1150, y: 620 });
    await page.mouse.click(blank.x, blank.y);

    // WHEN 1：Ctrl+A —— THEN（R-KBSEL-01）：页面无文本高亮 + 全部图元入选中集
    await page.keyboard.press("Control+a");
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe).multi_n === 3, null, { timeout: 4_000, polling: 100 },
    );
    const domSel = await page.evaluate(() => String(window.getSelection()));
    assert.equal(domSel, "", "Ctrl+A 不得产生浏览器页面文本全选（须 preventDefault）");
    let hl = await hlProbe();
    assert.equal(hl.multi_n, 3, "Ctrl+A 后 3 张表必须全部进入多选集");

    // WHEN 2：Delete —— THEN（R-KBSEL-02）：全部选中图元删除（表级联关系）
    await page.keyboard.press("Delete");
    await waitCounts([0, 0, 0, 0]);
    p = await lodProbe();
    assert.deepEqual(counts(p), [0, 0, 0, 0], "Delete 后画布图元必须清空（含级联关系）");

    // WHEN 3：Ctrl+Z —— THEN（R-KBSEL-03）：一次 Undo 恢复整次表/关系删除
    // 口径：便签/区域删除不入撤销栈（既有行为），undo 只恢复表与级联关系 → [3,2,0,0]
    await page.keyboard.press("Control+z");
    await waitCounts([3, 2, 0, 0]);
    // WHEN 3b：Ctrl+Y 重放删除（对称 redo）
    await page.keyboard.press("Control+y");
    await waitCounts([0, 0, 0, 0]);
    await page.keyboard.press("Control+z");
    await waitCounts([3, 2, 0, 0]);

    // WHEN 4：单选 t1 + Delete —— THEN：该表与其级联关系 r1 消失，可撤销
    const t1Head = await canvasPoint(page, { x: 60 + 100, y: 100 + 21 });
    await page.mouse.click(t1Head.x, t1Head.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    await page.keyboard.press("Delete");
    await waitCounts([2, 1, 0, 0]);
    p = await lodProbe();
    assert.deepEqual(counts(p), [2, 1, 0, 0], "删 t1 必须级联 r1（剩 t2/t3 + r2，便签/区域此前已删且不可撤销）");
    await page.keyboard.press("Control+z");
    await waitCounts([3, 2, 0, 0]);

    // WHEN 5：输入框内 Ctrl+A / Delete —— THEN（R-KBSEL-04）：原生文本行为，不触画布
    await page.mouse.click(t1Head.x, t1Head.y);
    await page.locator('[data-testid="inspector-table-form"]:visible').waitFor();
    const nameInput = page.locator('[data-testid="inspector-table-name"]');
    await nameInput.click();
    await page.keyboard.press("Control+a");
    await nameInput.pressSequentially("tbl_renamed");
    assert.equal(await nameInput.inputValue(), "tbl_renamed", "输入框内 Ctrl+A 必须为原生文本全选");
    hl = await hlProbe();
    assert.equal(hl.multi_n, 1, "输入框内 Ctrl+A 不得触发画布全选（多选集须保持仅 t1，不得扩为 3）");
    await page.keyboard.press("Delete"); // 删除输入框内残留字符（若有）——不得删画布图元
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe).tables_n === 3, null, { timeout: 2_000, polling: 100 },
    ).catch(() => {});
    p = await lodProbe();
    assert.deepEqual(counts(p), [3, 2, 0, 0], "输入框内 Delete 不得删除画布图元");
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
    // feat-issue-50（issue #50）：追加 .cdb-list-tree-node 限定按钮节点——
    // 关联标记 span（list-tree-node-related/-unrelated）也以 list-tree-node- 开头
    assert.equal(await page.locator('[data-testid^="list-tree-node-"].cdb-list-tree-node').count(), 1, "搜索 user 后树必须只剩 users 节点");
    await page.locator('[data-testid="list-tree-node-users"]').click();
    await page.locator('[data-testid="list-tree-node-users"].is-active').waitFor();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /users（1 字段）/, "点击 users 节点后网格必须切换");

    // 3. 选中 users 字段行 → 清空搜索恢复 3 节点 → 切 table_2 → 字段行选中态清空（删/移禁用）
    await page.locator('tr[data-testid^="list-view-row-users-"]').first().click();
    assert.equal(await page.locator('[data-testid="list-del-field"]').isDisabled(), false, "选中行后删字段必须可用");
    await page.locator('[data-testid="list-tree-search"]').fill("");
    assert.equal(await page.locator('[data-testid^="list-tree-node-"].cdb-list-tree-node').count(), 3, "清空搜索必须恢复 3 节点");
    await page.locator('[data-testid="list-tree-node-table_2"]').click();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_2（1 字段）/);
    assert.equal(await page.locator('[data-testid="list-del-field"]').isDisabled(), true, "切表后字段行选中态必须清空（删字段禁用）");
    assert.equal(await page.locator('[data-testid="list-move-up"]').isDisabled(), true, "切表后上移必须禁用");

    // 4. 无命中关键字 → 树内空态；右网格保持当前表
    await page.locator('[data-testid="list-tree-search"]').fill("zzz");
    await page.locator('[data-testid="list-tree-empty"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid^="list-tree-node-"].cdb-list-tree-node').count(), 0, "无命中时树节点必须为 0");
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

  // ─── ST-SP-LIST-GROUP-01：列表视图按 Area 分组（几何包含 + 折叠/搜索自动展开，fix-issue-50 / #50）───
  await run(["ST-SP-LIST-GROUP-01"], "列表视图按 Area 几何包含分组且可折叠/搜索自动展开", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await createTwoTables(page);
    await page.keyboard.press("t"); // 第三张表 table_3（不进区域）
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });

    // 画一个覆盖 table_1/table_2 中心 (295,166.5)/(350,201.5)、不含 table_3 中心 (405,236.5) 的区域
    // （表 i 落位 x=180+i*55, y=145+i*35；分组依据 = 表卡中心是否落入 Area 矩形）
    await page.locator('[data-testid="tool-new-area"]').click();
    const from = await canvasPoint(page, { x: 120, y: 120 });
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    const to = await canvasPoint(page, { x: 420, y: 225 });
    await page.mouse.move(to.x, to.y, { steps: 5 });
    await page.mouse.up();
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    await page.keyboard.press("Escape");
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 1, "拖框建区域必须落账 1 个 Area");
    const area = state.lastPutBody.diagram.areas[0];

    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();

    // 1. 分组头：Area 组（区域名 + 表数 2）+ 未分组（表数 1，table_3）
    const areaHeader = page.locator(`[data-testid="list-tree-group-${area.id}"]`);
    await areaHeader.waitFor();
    assert.ok((await areaHeader.textContent()).includes(area.name), "分组头必须显示区域名");
    assert.match(await areaHeader.textContent(), /2/, "Area 组必须显示表数 2");
    const ugHeader = page.locator('[data-testid="list-tree-group-ungrouped"]');
    await ugHeader.waitFor();
    assert.match(await ugHeader.textContent(), /1/, "未分组必须显示表数 1");
    await page.locator('[data-testid="list-tree-node-table_3"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid^="list-tree-node-"].cdb-list-tree-node').count(), 3, "分组后仍渲染全量 3 节点");

    // 2. 折叠 Area 组 → 组内节点隐藏；再展开恢复
    await areaHeader.click();
    await page.locator('[data-testid="list-tree-node-table_1"]').waitFor({ state: "hidden" });
    assert.ok(await page.locator('[data-testid="list-tree-node-table_1"]').isHidden(), "折叠后 Area 组内节点必须隐藏");
    await areaHeader.click();
    await page.locator('[data-testid="list-tree-node-table_1"]:visible').waitFor();

    // 3. 折叠态下搜索 table_1 → 含匹配表的 Area 组自动展开；无匹配的未分组自动折叠（不再渲染）
    await areaHeader.click();
    await page.locator('[data-testid="list-tree-node-table_1"]').waitFor({ state: "hidden" });
    await page.locator('[data-testid="list-tree-search"]').fill("table_1");
    await page.locator('[data-testid="list-tree-node-table_1"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid^="list-tree-group-"]').count(), 1, "搜索时只渲染含匹配表的分组");
    await page.locator('[data-testid="list-tree-search"]').fill("");
    await page.locator('[data-testid="list-tree-node-table_3"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid^="list-tree-group-"]').count(), 2, "清空搜索恢复 Area + 未分组两组");

    // 导航全程不改动落账数据（3 表 1 区域）
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, "分组导航不得改动落账表数");
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 1, "分组导航不得改动落账区域数");
  });

  // ─── ST-SP-LIST-REL-01：选中表时右侧展示关联表清单 + 树中关联表高亮/非关联弱化（fix-issue-50 / #50）───
  await run(["ST-SP-LIST-REL-01"], "列表视图关联表摘要与树节点关联高亮", async page => {
    const state = await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    // 先建 table_1 → table_2 关系（ST-CR-02 验证过的稳定组合：2 表 + createVisibleRelation）
    await createVisibleRelation(page);
    // 再建第三张表 table_3：不参与关系（关系工具已由 helper 的 Escape 退出）
    await page.keyboard.press("t");
    await page.locator('[data-testid="save-state"][data-state="saved"]').waitFor({ timeout: 8_000 });
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, "setup 后必须恰 3 张表（无凭空表）");

    await page.locator('[data-testid="btn-list-view"]').click();
    await page.locator('[data-testid="tab-pane-list-view"]:visible').waitFor();

    // 1. 默认选中首表 table_1 → 摘要区显示出边 table_2；芯片展示表名 + 映射后关系类型标签
    //（core-04 §10.5：1:1 / 1:N / N:M；cardinality_label 把存储值 one_to_* / many_to_* 映射为标签）
    const summary = page.locator('[data-testid="list-related-tables"]');
    await summary.waitFor();
    const summaryText = await summary.textContent();
    assert.ok(summaryText.includes("出边"), "table_1 的关联 table_2 必须归入出边");
    assert.match(summaryText, /table_2（(1:1|1:N|N:M)）/, `芯片必须展示表名与关系类型标签（actual=${summaryText}）`);

    // 2. 树中关联标记：table_2（关联）⇄ 高亮、table_3（非关联）弱化标记、当前表 table_1 无标记
    assert.equal(await page.locator('[data-testid="list-tree-node-related"]').count(), 1, "关联表节点必须追加 related 标记");
    assert.equal(await page.locator('[data-testid="list-tree-node-unrelated"]').count(), 1, "非关联表节点必须追加 unrelated 标记");
    assert.ok(
      await page.locator('[data-testid="list-tree-node-table_2"].is-related').count() === 1,
      "关联表节点必须高亮（is-related）"
    );
    assert.ok(
      await page.locator('[data-testid="list-tree-node-table_3"].is-unrelated').count() === 1,
      "非关联表节点必须弱化（is-unrelated）"
    );
    assert.ok(
      await page.locator('[data-testid="list-tree-node-table_1"].is-related').count() === 0
        && await page.locator('[data-testid="list-tree-node-table_1"].is-unrelated').count() === 0,
      "当前表自身既不算关联也不算非关联"
    );

    // 3. 点击关联表芯片 → 左树定位并切换选中（table_2 高亮 + 右网格切到 table_2）
    await page.locator('[data-testid="list-related-table-table_2"]').click();
    await page.locator('[data-testid="list-tree-node-table_2"].is-active').waitFor();
    assert.match(await page.locator('[data-testid="list-view-title"]').textContent(), /table_2（\d+ 字段）/, "点击芯片后右网格必须切到 table_2");
    // 选中切换后标记随当前表重算：table_1（关联）⇄，table_3 仍弱化
    assert.equal(await page.locator('[data-testid="list-tree-node-related"]').count(), 1, "切换后关联标记必须随当前表重算");
    assert.ok(
      await page.locator('[data-testid="list-tree-node-table_1"].is-related').count() === 1,
      "切换到 table_2 后 table_1 必须成为关联高亮节点"
    );

    // 4. 选中无关联的 table_3 → 摘要区隐藏不占位，标记全部消失
    await page.locator('[data-testid="list-tree-node-table_3"]').click();
    assert.equal(await page.locator('[data-testid="list-related-tables"]').count(), 0, "无关联时摘要区必须隐藏不占位");
    assert.equal(await page.locator('[data-testid="list-tree-node-related"]').count(), 0, "无关联时 related 标记必须消失");
    assert.equal(await page.locator('[data-testid="list-tree-node-unrelated"]').count(), 0, "无关联时 unrelated 标记必须消失");

    // 导航全程不改动落账数据（3 表；关系由 WS op 落账，前端 UI 关系来自本地 store 回放）
    assert.equal(state.lastPutBody?.diagram?.tables?.length, 3, "关联导航不得改动落账表数");
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

  // ─── ST-KB-UNDO-01：pan 后 Ctrl+Z / Ctrl+Y 仍生效（fix-issues-38-41 / #38，§5.14）───
  await run(["ST-KB-UNDO-01"], "拖动画布后撤销/重做快捷键不失效", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    const tablesN = () => page.evaluate(() => JSON.parse(window.__cdb_lod_probe).tables_n);
    const waitTablesN = (want) => page.waitForFunction(
      w => JSON.parse(window.__cdb_lod_probe)?.tables_n === w, want, { timeout: 6_000, polling: 100 },
    );

    // GIVEN：空画布（0 表）
    assert.equal(await tablesN(), 0, "基线应为 0 表");

    // WHEN 1：T 建表（undoable op，Command::AddTable 入栈）
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();
    await waitSaved(page);
    await waitTablesN(1);

    // WHEN 2：空白拖动 pan（位移 ≥50px，纯视图操作）
    const panOnce = async () => {
      const blank = await canvasPoint(page, { x: 900, y: 600 });
      await page.mouse.move(blank.x, blank.y);
      await page.mouse.down();
      await page.mouse.move(blank.x - 80, blank.y - 60, { steps: 5 });
      await page.mouse.up();
      await page.waitForTimeout(250); // rAF 落账
    };
    await panOnce();

    // THEN（R-PAN-UNDO-02）：pan 后 Ctrl+Z 撤销建表
    await page.keyboard.press("Control+z");
    await waitTablesN(0);
    // Ctrl+Y 重做
    await page.keyboard.press("Control+y");
    await waitTablesN(1);
    // 再 pan 一次 → Ctrl+Z 仍生效（R-PAN-UNDO-04 回归锚点）
    await panOnce();
    await page.keyboard.press("Control+z");
    await waitTablesN(0);
  });

  // ─── ST-CR-PAN-02：选中表后 pan 保留选中 / 空白单击清选（fix-issues-38-41 / #39，§5.15）───
  await run(["ST-CR-PAN-02"], "选中表时平移画布保持选中、空白单击才清选", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_hl_probe && !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    const hl = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe));

    // GIVEN：T 建表后点击表头完成画布选中（render 层 selected_id 由画布点击驱动，
    // T 建表只开 Inspector 不置 selected_id）
    await page.keyboard.press("t");
    const inspector = page.locator('[data-testid="inspector-table-name"]');
    await inspector.waitFor();
    await waitSaved(page);
    const header = await canvasPoint(page, { x: 295, y: 166.5 }); // 表1 落位 (180,145) 表头中心
    await page.mouse.move(header.x, header.y);
    await page.mouse.down();
    await page.mouse.up();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe)?.sel_table === true,
      null, { timeout: 6_000, polling: 100 },
    );

    // WHEN 1：空白拖动 pan（位移 100px ≥ 4px 阈值）
    const blank = await canvasPoint(page, { x: 900, y: 600 });
    await page.mouse.move(blank.x, blank.y);
    await page.mouse.down();
    await page.mouse.move(blank.x - 80, blank.y - 60, { steps: 5 });
    await page.mouse.up();
    await page.waitForTimeout(250); // rAF 落账

    // THEN（R-PAN-SEL-03/04）：pan 保留选中——Inspector 仍在、sel_table 仍 true
    assert.equal(await inspector.isVisible(), true, "pan 后 Inspector 应保持显示（R-PAN-SEL-04）");
    const h1 = await hl();
    assert.equal(h1.sel_table, true, "pan 后 sel_table 应保持 true（R-PAN-SEL-03）");

    // WHEN 2：空白单击（零位移 < 4px → click 语义；取 canvas 内安全空白点，防越出画布）
    const blank2 = await canvasPoint(page, { x: 950, y: 580 });
    await page.mouse.move(blank2.x, blank2.y);
    await page.mouse.down();
    await page.mouse.up();
    await page.waitForTimeout(250);

    // THEN（R-PAN-SEL-02）：单击清选——sel_table/any_sel 均 false，Inspector 关闭
    await page.waitForFunction(
      () => {
        const p = JSON.parse(window.__cdb_hl_probe);
        return p && p.sel_table === false && p.any_sel === false;
      },
      null, { timeout: 6_000, polling: 100 },
    );
    assert.equal(await inspector.isVisible().catch(() => false), false, "单击空白后 Inspector 应关闭（R-PAN-SEL-02）");

    // WHEN 3：Shift+框选回该表（R-PAN-SEL-05：框选口径不受 #39 影响）
    // canvasPoint 为 canvas 内屏幕像素（不做世界换算）；pan(-80,-60) 后表约占屏幕
    // (100..330, 85..190)，框选矩形取 (60,60)-(560,400) 完全覆盖
    const tl = await canvasPoint(page, { x: 60, y: 60 });
    const br = await canvasPoint(page, { x: 560, y: 400 });
    await page.keyboard.down("Shift");
    await page.mouse.move(tl.x, tl.y);
    await page.mouse.down();
    await page.mouse.move(br.x, br.y, { steps: 5 });
    await page.mouse.up();
    await page.keyboard.up("Shift");
    await page.waitForTimeout(250);
    const h3 = await hl();
    assert.equal(h3.multi_n >= 1 || h3.sel_table, true, "Shift 框选应重新选中该表（R-PAN-SEL-05）");
  });

  // ─── ST-CR-FONT-01：标签字号倍率循环 + 持久化 + 低 zoom 钳制（fix-issues-38-41 / #40，§5.16）───
  await run(["ST-CR-FONT-01"], "画布标签字号倍率可调且屏幕最小 10px 钳制", async page => {
    await installApi(page);
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    const lod = () => page.evaluate(() => JSON.parse(window.__cdb_lod_probe));

    // GIVEN：建表（有标签文本可渲染）
    await page.keyboard.press("t");
    await page.locator('[data-testid="inspector-table-name"]').waitFor();
    await waitSaved(page);

    // 基线（R-FONT-01）：默认倍率 1.0，详情档表名世界字号 13px
    const p0 = await lod();
    assert.equal(p0.label_font_scale, 1.0, "默认倍率必须为 1.0（R-FONT-01）");
    assert.ok(Math.abs(p0.label_font_world - 13.0) < 1e-6, `基线表名世界字号必须 13px（实际 ${p0.label_font_world}）`);

    // WHEN 1：点击字号按钮 → 1.25（R-FONT-02 循环入口）
    const btn = page.locator('[data-testid="canvas-font-scale"]');
    await btn.click();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe)?.label_font_scale === 1.25,
      null, { timeout: 6_000, polling: 100 },
    );
    assert.equal(await btn.textContent(), "字号：125%", "按钮文案必须随档位更新（R-FONT-02）");
    const p1 = await lod();
    assert.ok(Math.abs(p1.label_font_world - 16.25) < 1e-6, `1.25× 后表名字号必须放大为 16.25（实际 ${p1.label_font_world}）`);

    // WHEN 2：reload → 倍率经 localStorage 持久保持（R-FONT-01）
    // （刷新回落房间列表，需重新进入房间——同 ST-AN-01/ST-VIEW-DIM 先例）
    await page.reload();
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe)?.label_font_scale === 1.25,
      null, { timeout: 6_000, polling: 100 },
    );

    // WHEN 3：缩小到 zoom ≤ 0.35（R-FONT-04 低 zoom 钳制）——滚轮缩放（与用户操作同口径，
    // 规避 floating-controls 按钮在某些布局下的可见性抖动）
    const center = await canvasPoint(page, { x: 700, y: 450 });
    await page.mouse.move(center.x, center.y);
    for (let i = 0; i < 40; i++) {
      const z = (await lod()).zoom;
      if (z <= 0.35) break;
      await page.mouse.wheel(0, 240);
      await page.waitForTimeout(120);
    }
    const p2 = await lod();
    assert.ok(p2.zoom <= 0.35, `zoom 必须缩到 ≤0.35（实际 ${p2.zoom}）`);
    // THEN（R-FONT-04/05）：触发下限钳制且屏幕字号 ≥10px
    assert.equal(p2.min_font_clamped, true, "低 zoom 下必须触发 10px 下限钳制（R-FONT-05）");
    const screenPx = p2.label_font_world * p2.zoom;
    assert.ok(screenPx >= 10.0 - 1e-6, `钳制后表名屏幕字号必须 ≥10px（实际 ${screenPx}）`);
  });

  // ─── ST-CR-FONT-02：缩小全景后所有表卡字体/尺寸一致（#48 R-WIDTH-06~08 / R-FONT-07 / R-LOD-09）───
  await run(["ST-CR-FONT-02"], "缩小全景后离屏与可见表字体尺寸一致", async page => {
    // GIVEN：预置两表，t2 初始在远处，zoom=1 时离屏
    const presetDiagram = {
      tables: [
        { id: "t1", name: "LongTableNameVisible", x: 120, y: 120, color: "", comment: "",
          fields: [{ id: "f1", name: "very_long_field_name_abc", type_: "VARCHAR", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
          indices: [] },
        { id: "t2", name: "LongTableNameOffscreen", x: 1800, y: 1200, color: "", comment: "",
          fields: [{ id: "f2", name: "another_long_field_xyz", type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
          indices: [] },
      ],
      references: [], areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_font_consistency_probe, null, { timeout: 8_000 });

    const probe = () => page.evaluate(() => JSON.parse(window.__cdb_font_consistency_probe));
    const pBase = await probe();
    assert.equal(pBase.visible_n, 1, "zoom=1 时仅 t1 在视口内（t2 离屏）");
    assert.equal(pBase.font_bucket_consistent, true, "单表必然 consistent");
    const baseMinW = pBase.min_aabb_w;

    // WHEN 1：缩放到 zoom ≤ 0.2，使离屏 t2 进入视口
    const lod = () => page.evaluate(() => JSON.parse(window.__cdb_lod_probe));
    const center = await canvasPoint(page, { x: 700, y: 450 });
    await page.mouse.move(center.x, center.y);
    for (let i = 0; i < 60; i++) {
      const z = (await lod()).zoom;
      if (z <= 0.2) break;
      await page.mouse.wheel(0, 240);
      await page.waitForTimeout(120);
    }
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_font_consistency_probe)?.visible_n >= 2,
      null, { timeout: 6_000, polling: 100 },
    );
    const pLow = await probe();
    const zLow = (await lod()).zoom;
    assert.ok(zLow <= 0.2, `zoom 必须缩到 ≤0.2（实际 ${zLow}）`);
    assert.equal(pLow.visible_n, 2, `低 zoom 全景下两表都必须可见（实际 ${pLow.visible_n}）`);
    assert.equal(pLow.font_bucket_consistent, true, "两表 effective 字号必须一致（无斑块差异）");
    assert.ok(pLow.min_aabb_w > baseMinW, `低 zoom 下表宽应随 effective 字号放大（${pLow.min_aabb_w} > ${baseMinW}）`);

    // WHEN 2：缩回到 zoom≈1 后切字号到 1.5 档，表框再次加宽（低 zoom 已触发 10px 下限钳制，
    // 此时切字号不会继续放大；回退到非钳制 zoom 才能观测字号倍率对表宽的影响）
    for (let i = 0; i < 80; i++) {
      const z = (await lod()).zoom;
      if (z >= 0.99) break;
      await page.mouse.wheel(0, -120);
      await page.waitForTimeout(80);
    }
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe)?.zoom >= 0.99,
      null, { timeout: 8_000, polling: 100 },
    );
    const pZoomBack = await probe();
    const btn = page.locator('[data-testid="canvas-font-scale"]');
    // 1.0 → 1.25 → 1.5
    await btn.click();
    await btn.click();
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_lod_probe)?.label_font_scale === 1.5,
      null, { timeout: 6_000, polling: 100 },
    );
    const pScale = await probe();
    assert.equal(pScale.font_bucket_consistent, true, "1.5× 字号下仍 consistent");
    assert.ok(pScale.min_aabb_w > pZoomBack.min_aabb_w, `1.5× 字号下表框应再次加宽（${pScale.min_aabb_w} > ${pZoomBack.min_aabb_w}）`);
  });

  // ─── ST-AN-03：区域锁定全链路（fix-issues-38-41 / #41，§5.17 R-AREALOCK-02~06）───
  await run(["ST-AN-03"], "区域锁定：右键锁定 → 拖动无效 → 持久化 → Inspector 解锁恢复", async page => {
    const state = await installApi(page, { persistGet: true });
    await login(page);
    await createRoomAndEnter(page);
    // 空画布有 EmptyGuide 引导层会拦截 pointer——先建两表（同 ST-AN-01 先例）
    await createTwoTables(page);

    // GIVEN：拖框建区域（屏幕 (600,400)-(900,600)，zoom=1 世界=屏幕）
    await page.locator('[data-testid="tool-new-area"]').click();
    const from = await canvasPoint(page, { x: 600, y: 400 });
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move(from.x + 300, from.y + 200, { steps: 5 });
    await page.mouse.up();
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.length, 1, "区域必须已落账");
    assert.equal(state.lastPutBody.diagram.areas[0].locked ?? false, false, "新建区域默认未锁定（R-AREALOCK-01）");
    const baseX = state.lastPutBody.diagram.areas[0].x;

    // WHEN 1：右键区域 → 菜单出现「锁定区域」（R-AREALOCK-02a）
    await page.mouse.click(from.x + 60, from.y + 20, { button: "right" });
    const lockItem = page.locator('[data-testid="ctx-toggle-area-lock"]');
    await lockItem.waitFor();
    assert.equal(await lockItem.textContent(), "锁定区域", "未锁定时菜单项必须为「锁定区域」");
    await lockItem.click();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.[0]?.locked, true, "锁定必须随 PUT 落库（R-AREALOCK-06）");

    // THEN 1（R-AREALOCK-03）：锁定态拖动 no-op——拖区域标题栏 +100px 不触发新落账
    const putsBefore = state.putCalls;
    await page.mouse.move(from.x + 60, from.y + 20);
    await page.mouse.down();
    await page.mouse.move(from.x + 160, from.y + 20, { steps: 5 });
    await page.mouse.up();
    await page.waitForTimeout(600); // 等过 1s debounce 前窗口的一半以上无保存即不再触发（无 dirty）
    assert.equal(state.putCalls, putsBefore, "锁定区域拖动不得触发落账（拖动 no-op）");

    // THEN 2（R-AREALOCK-04）：锁定区域仍可选中——点击标题栏开 Inspector
    await page.mouse.click(from.x + 60, from.y + 20);
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    const lockCheckbox = page.locator('[data-testid="inspector-area-locked"]');
    await lockCheckbox.waitFor();
    assert.equal(await lockCheckbox.isChecked(), true, "Inspector 复选框必须反映锁定态（R-AREALOCK-02b）");

    // WHEN 2：reload → 锁定持久保持（R-AREALOCK-06，persistGet 回放最近 PUT 文档）
    await page.reload();
    await createRoomAndEnter(page); // 刷新回落房间列表，重新进入（同 ST-AN-01 先例）
    await page.locator('[data-testid="editor-canvas-container"] canvas').waitFor();
    await page.waitForFunction(() => !!window.__cdb_lod_probe, null, { timeout: 8_000 });
    await page.mouse.click(from.x + 60, from.y + 20, { button: "right" });
    const unlockItem = page.locator('[data-testid="ctx-toggle-area-lock"]');
    await unlockItem.waitFor();
    assert.equal(await unlockItem.textContent(), "解锁区域", "reload 后必须仍为锁定态（菜单显示「解锁区域」）");
    await page.mouse.click(from.x + 450, from.y - 150); // 左键点空白关菜单（容器 on:click 关闭）
    await page.waitForTimeout(150);

    // WHEN 3：Inspector 复选框解锁（R-AREALOCK-02b）
    await page.mouse.click(from.x + 60, from.y + 20);
    await page.locator('[data-testid="inspector-area-form"]:visible').waitFor();
    await page.locator('[data-testid="inspector-area-locked"]').uncheck();
    await waitSaved(page);
    assert.equal(state.lastPutBody?.diagram?.areas?.[0]?.locked, false, "解锁必须落库");

    // THEN 3（R-AREALOCK-03）：解锁后拖动恢复——拖 +100px 落账 x=baseX+100
    await page.mouse.move(from.x + 60, from.y + 20);
    await page.mouse.down();
    await page.mouse.move(from.x + 160, from.y + 20, { steps: 5 });
    await page.mouse.up();
    await waitSaved(page);
    const newX = state.lastPutBody?.diagram?.areas?.[0]?.x;
    assert.ok(
      Math.abs(newX - (baseX + 100)) <= 1.0,
      `解锁后拖动必须落账新坐标（期望 ≈${baseX + 100}，实际 ${newX}；同时证明锁定态拖动确实未位移）`,
    );
  });
  // ─── ST-PB-11：密集关系线点击目的一致 + 空白不误选（fix-issues-42-44 / #44，§4.7 R-HIT-02/03）───
  await run(["ST-PB-11"], "密集近平行关系线点击选中与点击目标一致", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: id, x, y, color: "", comment: "",
      fields: [{ id: fid, name: fid, type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
      indices: [],
    });
    const mkR = (id, st, sf, et, ef) => ({
      id, name: "", start_table_id: st, end_table_id: et, start_field_id: sf, end_field_id: ef,
      type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "",
    });
    // 两条近平行水平线：r1（t1→t2，线 y=190.5）与 r2（t3→t4，线 y=200.5），间距 10px
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 600, 130, "f2"),
               mkT("t3", 100, 140, "f3"), mkT("t4", 600, 140, "f4")],
      references: [mkR("r1", "t1", "f1", "t2", "f2"), mkR("r2", "t3", "f3", "t4", "f4")],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_hl_probe, null, { timeout: 8_000 });
    const selRefId = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe)?.sel_ref_id);

    const midX = (100 + 230 + 600) / 2; // 465（两表之间的线中段）
    const y1 = 130 + 43 + 35 / 2;       // 190.5（r1 线）
    // 距 r1 线 6px、距 r2 线 4px——两线同在 8px 带宽内，必须选中距离最小的 r2（R-HIT-02）
    let pt = await canvasPoint(page, { x: midX, y: y1 + 6 });
    await page.mouse.click(pt.x, pt.y);
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe)?.sel_ref_id === "r2", null, { timeout: 4_000, polling: 100 },
    );
    // 紧贴 r1（1px；r2 相距 9px 超阈值）→ r1
    pt = await canvasPoint(page, { x: midX, y: y1 + 1 });
    await page.mouse.click(pt.x, pt.y);
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe)?.sel_ref_id === "r1", null, { timeout: 4_000, polling: 100 },
    );
    // 全部超阈值空白 → 不误选（R-HIT-03；#39 口径：空白单击清选）
    pt = await canvasPoint(page, { x: midX, y: y1 - 30 });
    await page.mouse.click(pt.x, pt.y);
    await page.waitForFunction(
      () => {
        const p = JSON.parse(window.__cdb_hl_probe);
        return p && p.sel_ref === false && p.sel_ref_id === null;
      }, null, { timeout: 4_000, polling: 100 },
    );
  });

  // ─── ST-PB-13：点击关系线高亮关系线与两端表（fix-issue-47 / #47 R-HL-REF-01~03） ──
  await run(["ST-PB-13"], "点击关系线高亮关系线与两端表", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: id, x, y, color: "", comment: "",
      fields: [{ id: fid, name: fid, type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
      indices: [],
    });
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 600, 130, "f2")],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f2",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_hl_probe, null, { timeout: 8_000 });

    const probe = () => page.evaluate(() => JSON.parse(window.__cdb_hl_probe));
    const mid = await canvasPoint(page, { x: 465, y: 190.5 });
    await page.mouse.click(mid.x, mid.y);

    // R-HL-REF-01：关系线被选中，探针暴露 sel_ref_id
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe)?.sel_ref_id === "r1", null, { timeout: 4_000, polling: 100 },
    );

    // R-HL-REF-02/03：两端表 t1、t2 均进入高亮状态
    const p = await probe();
    assert.ok(Array.isArray(p.rel_endpoint_ids), "rel_endpoint_ids 必须为数组");
    assert.deepEqual(new Set(p.rel_endpoint_ids), new Set(["t1", "t2"]), "选中关系线的两端表必须为 t1 与 t2");

    // 点击空白清除高亮
    const blank = await canvasPoint(page, { x: 465, y: 400 });
    await page.mouse.click(blank.x, blank.y);
    await page.waitForFunction(
      () => {
        const p2 = JSON.parse(window.__cdb_hl_probe);
        return p2 && p2.sel_ref === false && p2.sel_ref_id === null;
      }, null, { timeout: 4_000, polling: 100 },
    );
  });

  // ─── ST-PB-14：点击端点记号附近 10px 热区选中关系（fix-relation-mouse-hit-precision / R-HIT-06）──
  await run(["ST-PB-14"], "点击端点记号附近 10px 热区选中关系", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: id, x, y, color: "", comment: "",
      fields: [{ id: fid, name: fid, type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
      indices: [],
    });
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 600, 130, "f2")],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f2",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_hl_probe, null, { timeout: 8_000 });

    // R-HIT-06：点击右端端点中心垂直向下 9px 处（距主线 9px > 8px 主线带宽；距端点中心 9px <= 10px 热区）
    const endpoint = await canvasPoint(page, { x: 600, y: 199.5 });
    await page.mouse.click(endpoint.x, endpoint.y);
    await page.waitForFunction(
      () => JSON.parse(window.__cdb_hl_probe)?.sel_ref_id === "r1", null, { timeout: 4_000, polling: 100 },
    );

    // 点击空白清除高亮
    const blank = await canvasPoint(page, { x: 465, y: 400 });
    await page.mouse.click(blank.x, blank.y);
    await page.waitForFunction(
      () => {
        const p2 = JSON.parse(window.__cdb_hl_probe);
        return p2 && p2.sel_ref === false && p2.sel_ref_id === null;
      }, null, { timeout: 4_000, polling: 100 },
    );
  });

  // ─── ST-PB-12：悬浮关系线 tooltip（fix-issues-42-44 / #43，§4.8 R-HOV-01~05）───
  await run(["ST-PB-12"], "悬浮关系线角落摘要 tooltip（B1）+ 同命中零位移 + 不改选中态", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: id, x, y, color: "", comment: "",
      fields: [{ id: fid, name: fid, type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
      indices: [],
    });
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 600, 130, "f2")],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f2",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);
    await page.waitForFunction(() => !!window.__cdb_hl_probe, null, { timeout: 8_000 });

    // WHEN：悬浮线中段（R-HOV-01）
    const mid = await canvasPoint(page, { x: 465, y: 190.5 });
    await page.mouse.move(mid.x, mid.y);
    // THEN：tooltip 出现且文案为 源表.源字段 → 目标表.目标字段（R-HOV-02）
    const tip = page.locator('[data-testid="rel-hover-tooltip"]');
    await tip.waitFor({ timeout: 4_000 });
    assert.equal(await tip.textContent(), "t1.f1 → t2.f2", "tooltip 必须为 源表.源字段 → 目标表.目标字段");
    // fix-issue-46（#46 R-HOV-03 B1）：tooltip 为画布容器左下角固定摘要区——
    // 同一命中下指针微动，tooltip 位置不变（R-PERF-HOV-01/02 抖动回归锚点）
    const box1 = await tip.boundingBox();
    const stack = await page.locator(".cdb-canvas-stack").boundingBox();
    assert.ok(box1 && stack, "tooltip 与 canvas-stack 均可测量");
    assert.ok(Math.abs(box1.x - stack.x - 12) < 4, "tooltip 应锚定容器左边缘 12px");
    assert.ok(Math.abs(stack.y + stack.height - (box1.y + box1.height) - 12) < 4, "tooltip 应锚定容器底边 12px");
    await page.mouse.move(mid.x + 30, mid.y + 3); // 沿线微动，仍命中同一关系
    await page.waitForTimeout(150); // 等待 rAF 帧
    const box2 = await tip.boundingBox();
    assert.deepEqual(box2, box1, "同一命中下 tooltip 不得随指针微动而位移（R-PERF-HOV-01/02）");
    // AND：选中态不变（R-HOV-04）
    const probe = await page.evaluate(() => JSON.parse(window.__cdb_hl_probe));
    assert.equal(probe.sel_ref, false, "悬停不得改变关系选中态（R-HOV-04）");
    // WHEN：移出命中带宽 → tooltip 消失（R-HOV-05）
    const away = await canvasPoint(page, { x: 465, y: 400 });
    await page.mouse.move(away.x, away.y);
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="rel-hover-tooltip"]'), null, { timeout: 4_000, polling: 100 },
    );
  });
  // ─── ST-CR-ANCHOR-01：comment_mode 切换后关系锚点与当前卡宽对齐（fix-issue-49 / #49 R-WIDTH-08）───
  await run(["ST-CR-ANCHOR-01"], "comment_mode 切换后关系锚点与当前卡宽对齐", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: id, x, y, color: "", comment: "表级注释",
      fields: [{ id: fid, name: fid, type_: "INT", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "字段注释用于撑宽字段注释字段注释", tag: "", dict_code: "" }],
      indices: [],
    });
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 600, 130, "f2")],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f2",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);

    // 辅助：从 data-follow-path 解析端点
    async function relationEndpoints() {
      const d = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");
      assert.ok(d && d.startsWith("M"), "关系线 data-follow-path 必须存在");
      const nums = d.match(/-?\d+\.?\d*/g).map(Number);
      const [x1, y1, , , , , x2, y2] = nums;
      return { x1, y1, x2, y2 };
    }

    // 默认 NameComment（含注释，表宽 > 230）
    const nc = await relationEndpoints();
    const ncPath = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");

    // 切换到 Name only：字段/表注释不渲染，表宽回落 230
    const btn = page.locator('[data-testid="canvas-comment-display"]');
    for (let i = 0; i < 3; i++) {
      const text = await btn.textContent();
      if (text?.includes("仅英文名")) break;
      await btn.click();
      await page.waitForTimeout(80);
    }
    await page.waitForFunction(
      () => document.querySelector('[data-testid="canvas-comment-display"]')?.textContent?.includes("仅英文名"),
      { timeout: 3_000 },
    );
    // 等 path 随 mode 切换重绘
    await page.waitForFunction(
      (prev) => document.querySelector('[data-testid="editor-canvas-container"] canvas')?.getAttribute("data-follow-path") !== prev,
      ncPath,
      { timeout: 3_000, polling: 100 },
    );
    const name = await relationEndpoints();

    // NameComment 因注释撑宽，右锚点必须比 Name 更靠右；Name 模式贴齐 230 宽表卡
    assert.ok(nc.x1 > name.x1, `NameComment 右锚点应宽于 Name（${nc.x1} > ${name.x1}）`);
    assert.ok(Math.abs(name.x1 - 330) < 2, `Name 模式下 t1 右锚点应贴齐 330（x1=${name.x1}）`);
    assert.ok(Math.abs(name.x2 - 600) < 2, `Name 模式下 t2 左锚点应贴齐 600（x2=${name.x2}）`);
  });

  // ─── ST-CR-TOPO-WIDTH-01：表维度下长中文注释卡宽放宽、可读性提升（fix-issue-48 / #48 R-LOD-10）───
  // R-LOD-11 修订：Tier 上限仅约束字段行需求——本用例改由长字段行驱动宽度，
  // 保证表维度（640）> 字段维度（480）的钳制结构可观测；表头需求驱动时两维度
  // 均不受 Tier 上限钳制（ST-CR-HEADER-WIDTH-01 覆盖）。
  await run(["ST-CR-TOPO-WIDTH-01"], "表维度下长中文注释卡宽放宽且关系锚点同步", async page => {
    const mkT = (id, x, y, fid) => ({
      id, name: "asset_object_registry", x, y, color: "", comment: "统一资产主表",
      fields: [{ id: fid, name: "very_long_field_name_for_layout_test_padding", type_: "VARCHAR_WITH_LONG_TYPE_NAME", default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" }],
      indices: [],
    });
    const presetDiagram = {
      tables: [mkT("t1", 100, 130, "f1"), mkT("t2", 700, 130, "f2")],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "f1", end_field_id: "f2",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);

    async function relationEndpoints() {
      const d = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");
      assert.ok(d && d.startsWith("M"), "关系线 data-follow-path 必须存在");
      const nums = d.match(/-?\d+\.?\d*/g).map(Number);
      const [x1, y1, , , , , x2, y2] = nums;
      return { x1, y1, x2, y2 };
    }

    // 默认字段维度
    const detail = await relationEndpoints();
    const detailPath = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");

    // 切换到表维度
    const dimBtn = page.locator('[data-testid="canvas-view-dimension"]');
    for (let i = 0; i < 3; i++) {
      const text = await dimBtn.textContent();
      if (text?.includes("维度：表")) break;
      await dimBtn.click();
      await page.waitForTimeout(80);
    }
    await page.waitForFunction(
      () => document.querySelector('[data-testid="canvas-view-dimension"]')?.textContent?.includes("维度：表"),
      { timeout: 3_000 },
    );
    await page.waitForFunction(
      (prev) => document.querySelector('[data-testid="editor-canvas-container"] canvas')?.getAttribute("data-follow-path") !== prev,
      detailPath,
      { timeout: 3_000, polling: 100 },
    );
    const topo = await relationEndpoints();

    // t1 右侧锚点右移（表维度更宽）；t2 左侧锚点恒为 table.x=700，不随宽度变化
    assert.ok(topo.x1 > detail.x1, `表维度 t1 应更宽（${topo.x1} > ${detail.x1}）`);
    assert.ok(Math.abs(topo.x2 - 700) < 2, `t2 左锚点应贴齐 700（x2=${topo.x2}）`);

    // 切回字段维度，path 恢复
    for (let i = 0; i < 3; i++) {
      const text = await dimBtn.textContent();
      if (text?.includes("维度：字段")) break;
      await dimBtn.click();
      await page.waitForTimeout(80);
    }
    await page.waitForFunction(
      () => document.querySelector('[data-testid="canvas-view-dimension"]')?.textContent?.includes("维度：字段"),
      { timeout: 3_000 },
    );
    const back = await relationEndpoints();
    assert.ok(Math.abs(back.x1 - detail.x1) < 2, "切回字段维度后 t1 右锚点应恢复");
    assert.ok(Math.abs(back.x2 - detail.x2) < 2, "切回字段维度后 t2 左锚点应恢复");
  });

  // ─── ST-CR-HEADER-WIDTH-01：字段维度下长中英文表头自适应加宽（fix-table-header-width-autofit / R-LOD-11）───
  await run(["ST-CR-HEADER-WIDTH-01"], "字段维度下长中英文表头完整显示、auto 宽突破 480 不截断", async page => {
    const mkF = (id, name, ty) => ({ id, name, type_: ty, default: "", check: "", primary: true, unique: false, not_null: true, increment: false, comment: "", tag: "", dict_code: "" });
    // t1：auto 宽——表头需求（表名 18 ASCII + 注释 20 CJK + 6 ASCII + 计数/留白）≈515px > 480，
    // R-LOD-11 下应突破 TABLE_WIDTH_MAX，注释不被 `…` 截断（draw 侧 truncate 与宽度同源）。
    const longCmt = "资产类型字典；客户确认分类后由 seed 脚本初始化";
    const mkT = (id, x, width) => ({
      id, name: "asset_kind_catalog", x, y: 130, color: "", comment: longCmt,
      fields: [mkF(`${id}-f1`, "id", "BIGINT")], indices: [],
      ...(width !== undefined ? { width } : {}),
    });
    const presetDiagram = {
      // t2 为显式宽 400 对照（width 语义回归由 UT-CR-HEADER-WIDTH-01 步骤 5 数值覆盖，
      // e2e 侧以 t2 左缘锚点贴齐做烟雾级验证——左缘坐标与宽度解耦）。
      tables: [mkT("t1", 100), mkT("t2", 1000, 400)],
      references: [{ id: "r1", name: "", start_table_id: "t1", end_table_id: "t2", start_field_id: "t1-f1", end_field_id: "t2-f1",
        type_: "one_to_many", on_delete: "RESTRICT", on_update: "RESTRICT", color: "", line_type: "", stroke_style: "" }],
      areas: [], notes: [],
    };
    await installApi(page, { presetDiagram });
    await login(page);
    await createRoomAndEnter(page);

    // 默认字段维度，直接读取首条关系线端点：起点右缘 = t1.x + 有效宽，终点左缘 = t2.x
    const d = await page.locator('[data-testid="editor-canvas-container"] canvas').getAttribute("data-follow-path");
    assert.ok(d && d.startsWith("M"), "关系线 data-follow-path 必须存在");
    const nums = d.match(/-?\d+\.?\d*/g).map(Number);
    const [x1, , , , , , x2] = nums;

    const w1 = x1 - 100;
    // THEN 1/2：auto 宽突破 480（表头行需求不被 Tier 上限钳制 → 注释完整可见不截断）
    assert.ok(w1 > 480, `t1 auto 宽应突破 TABLE_WIDTH_MAX（w=${w1} > 480，表头注释不被钳制截断）`);
    assert.ok(w1 <= 720, `t1 auto 宽应 ≤ TABLE_WIDTH_MAX_HEADER（w=${w1} ≤ 720）`);
    // THEN 3：关系锚点随放宽后的宽度同源取缘——终点左缘贴齐 t2.x（不错位）
    assert.ok(Math.abs(x2 - 1000) < 2, `t2 左锚点应贴齐 1000（x2=${x2}）`);
  });

  // ─── ST-S04-UI-17：房间卡片重命名（fix-issue-45-room-rename，issue #45，core-S04 §2.4） ──
  await run(["ST-S04-UI-17"], "owner 重命名入口 + 模态 + PATCH 落账 + 列表即时一致 + 字段解耦", async page => {
    const state = await installApi(page, {
      roomsList: [
        { id: "room-own", name: "数据模型评审", diagramId: "diagram-own", diagramTitle: "数据模型评审",
          myRole: "owner", memberCount: 1, updatedAt: "2026-09-07T00:00:00Z" },
        { id: "room-edit", name: "协作评审室", diagramId: "diagram-edit", diagramTitle: "协作评审室",
          myRole: "editor", memberCount: 3, updatedAt: "2026-09-06T00:00:00Z" },
      ],
    });
    await login(page);

    // 重命名入口仅 owner 可见
    await page.locator('[data-testid="room-card-rename-room-own"]').waitFor();
    assert.equal(
      await page.locator('[data-testid="room-card-rename-room-edit"]').count(), 0,
      "editor 角色不得渲染重命名按钮",
    );

    // 点击重命名：不进房间，弹模态且预填当前名
    await page.locator('[data-testid="room-card-rename-room-own"]').click();
    await page.locator('[data-testid="modal-rename-room"]:visible').waitFor();
    assert.equal(await page.locator('[data-testid="room-editor-page"]:visible').count(), 0, "点击重命名不得进入房间");
    assert.equal(await page.locator('[data-testid="input-rename-room"]').inputValue(), "数据模型评审", "输入框应预填当前名");

    // 前端预校验：空名不放行（无 PATCH 请求）
    await page.locator('[data-testid="input-rename-room"]').fill("   ");
    await page.locator('[data-testid="btn-confirm-rename-room"]').click();
    await page.locator('[data-testid="rename-room-error"]:not(:empty)').waitFor();
    assert.equal((state.renamedRooms ?? []).length, 0, "空名不得发起 PATCH");

    // 合法改名：PATCH 落账 + 卡片即时更新 + Toast
    await page.locator('[data-testid="input-rename-room"]').fill("评审空间-新");
    await page.locator('[data-testid="btn-confirm-rename-room"]').click();
    await page.waitForFunction(
      () => !document.querySelector('[data-testid="modal-rename-room"]'),
      { timeout: 5_000 },
    );
    assert.deepEqual(state.renamedRooms, [{ id: "room-own", name: "评审空间-新" }], "必须发起 PATCH /rooms/room-own");
    const card = page.locator('[data-testid="room-list-item-room-own"]');
    await card.locator("h2", { hasText: "评审空间-新" }).waitFor({ timeout: 3_000 });
    const toast = page.locator('[data-testid="notice-toast"]:visible');
    await toast.waitFor({ timeout: 3_000 });
    assert.match((await toast.textContent()) ?? "", /已重命名/);
    // editor 卡片不受影响
    await page.locator('[data-testid="room-list-item-room-edit"]:visible').waitFor();
  });

} finally {
  await browser?.close();
  frontend?.kill("SIGTERM");
}

if (failed) process.exitCode = 1;
