// capture-readme-screenshot.mjs — 生成 README 头图：真实后端 + 真实注册登录 + 演示模型 + Playwright 截图。
//
// 前置条件：
//   1) cd frontend-rs && trunk build --release false
//   2) cd backend && COLDRAWDB_STATIC_DIR=$(pwd)/../frontend-rs/dist cargo run --release --bin backend
//   3) node scripts/capture-readme-screenshot.mjs   （在 frontend-rs/ 下执行）
//
// 产物：仓库根 app-editor.png（1600×900 @2x，暗色主题编辑器界面）
import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { applyPlaywrightBrowserEnv } from "./resolve-playwright-browsers.mjs";

const BASE = process.env.CAPTURE_BASE_URL || "http://127.0.0.1:3000";
const OUT_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const OUT_FILE = path.join(OUT_DIR, "app-editor.png");
const EMAIL = `shot-${Date.now()}@coldrawdb.local`;
const PASSWORD = "Shot#2026-Pw";

applyPlaywrightBrowserEnv();
const { chromium } = await import("playwright");

async function postJson(path_, body, token) {
  const res = await fetch(BASE + path_, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  if (!res.ok) throw new Error(`POST ${path_} → ${res.status}: ${text}`);
  return text ? JSON.parse(text) : {};
}

async function getJson(path_, token) {
  const res = await fetch(BASE + path_, { headers: token ? { authorization: `Bearer ${token}` } : {} });
  const text = await res.text();
  if (!res.ok) throw new Error(`GET ${path_} → ${res.status}: ${text}`);
  return JSON.parse(text);
}

// ── 1) 注册 → 登录 → me（diagrams v1 虽无强制 JWT，但前端鉴权门需要会话）──
try {
  await postJson("/api/v1/auth/register", { email: EMAIL, password: PASSWORD, displayName: "Screenshot" });
} catch (e) {
  console.warn("register 非致命失败（可能邮箱已存在）:", e.message);
}
const token = await postJson("/api/v1/auth/login", { email: EMAIL, password: PASSWORD, rememberDevice: true });
const me = await getJson("/api/v1/auth/me", token.accessToken);

// 前端 AuthSession 持久化形态（snake_case，见 editor_data_access.rs persist_auth_session）
const session = {
  access_token: token.accessToken,
  expires_in: token.expiresIn,
  token_type: token.tokenType,
  user: {
    id: me.id ?? me.user?.id ?? "",
    email: me.email ?? me.user?.email ?? EMAIL,
    displayName: me.displayName ?? me.user?.displayName ?? "Screenshot",
    emailVerifiedAt: me.emailVerifiedAt ?? me.user?.emailVerifiedAt ?? null,
  },
};

// ── 2) 建图 → 填充演示模型 → 按当前 revision 全量保存 ──
const created = await postJson("/api/v1/diagrams", { name: "订单与结算数据模型", database: "postgresql" });
const diagramId = created.data.id;
const doc = (await getJson(`/api/v1/diagrams/${diagramId}`)).data;

const f = (id, name, type_, extra = {}) => ({
  id, name, type_, default: "", check: "", primary: false, unique: false, not_null: false, increment: false, comment: "", ...extra,
});
const table = (id, name, x, y, color, fields, comment = "") => ({
  id, name, x, y, color, comment, fields, indices: [],
});

const U = Date.now().toString(36); // 全局 id 空间：table/field/reference 主键跨图唯一，避免与历史行冲突
const uid = (id) => `${id}-${U}`;

doc.tables = [
  table(uid("t-customer"), "customers", 60, 140, "#175e7a", [
    f(uid("f-cust-id"), "id", "BIGINT", { primary: true, unique: true, not_null: true, increment: true }),
    f(uid("f-cust-email"), "email", "VARCHAR(255)", { not_null: true, unique: true }),
    f(uid("f-cust-name"), "display_name", "VARCHAR(80)"),
    f(uid("f-cust-created"), "created_at", "TIMESTAMPTZ", { not_null: true }),
  ], "注册用户"),
  table(uid("t-account"), "accounts", 560, 70, "#2b7de9", [
    f(uid("f-acc-id"), "id", "BIGINT", { primary: true, unique: true, not_null: true, increment: true }),
    f(uid("f-acc-cust"), "customer_id", "BIGINT", { not_null: true }),
    f(uid("f-acc-no"), "account_no", "VARCHAR(34)", { not_null: true, unique: true }),
    f(uid("f-acc-balance"), "balance", "NUMERIC(18,2)", { not_null: true }),
    f(uid("f-acc-status"), "status", "SMALLINT", { not_null: true, default: "1" }),
  ], "结算账户"),
  table(uid("t-txn"), "transactions", 560, 430, "#2f9e63", [
    f(uid("f-txn-id"), "id", "BIGINT", { primary: true, unique: true, not_null: true, increment: true }),
    f(uid("f-txn-acc"), "account_id", "BIGINT", { not_null: true }),
    f(uid("f-txn-amount"), "amount", "NUMERIC(18,2)", { not_null: true }),
    f(uid("f-txn-type"), "txn_type", "VARCHAR(16)", { not_null: true }),
    f(uid("f-txn-time"), "booked_at", "TIMESTAMPTZ", { not_null: true }),
  ], "交易流水"),
  table(uid("t-loan"), "loans", 880, 330, "#e8890c", [
    f(uid("f-loan-id"), "id", "BIGINT", { primary: true, unique: true, not_null: true, increment: true }),
    f(uid("f-loan-acc"), "account_id", "BIGINT", { not_null: true }),
    f(uid("f-loan-principal"), "principal", "NUMERIC(18,2)", { not_null: true }),
    f(uid("f-loan-rate"), "annual_rate", "NUMERIC(6,4)", { not_null: true }),
    f(uid("f-loan-due"), "due_date", "DATE"),
  ], "贷款合约"),
];

const ref = (id, name, st, sf, et, ef) => ({
  id, name, start_table_id: st, end_table_id: et, start_field_id: sf, end_field_id: ef,
  type_: "1:N", on_delete: "RESTRICT", on_update: "CASCADE",
});
doc.references = [
  ref(uid("r-1"), "客户拥有账户", uid("t-customer"), uid("f-cust-id"), uid("t-account"), uid("f-acc-cust")),
  ref(uid("r-2"), "账户产生流水", uid("t-account"), uid("f-acc-id"), uid("t-txn"), uid("f-txn-acc")),
  ref(uid("r-3"), "账户申请贷款", uid("t-account"), uid("f-acc-id"), uid("t-loan"), uid("f-loan-acc")),
];

await fetch(`${BASE}/api/v1/diagrams/${diagramId}`, {
  method: "PUT",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({ expected_revision: doc.revision, diagram: doc }),
}).then(async (res) => {
  if (!res.ok) throw new Error(`PUT diagram → ${res.status}: ${await res.text()}`);
});

// ── 3) 创建协作房间绑定该图（生产前端编辑器为房间制：工作空间点卡片进房）──
const room = await postJson("/api/v1/rooms", { name: "演示房间", diagramId: diagramId }, token.accessToken);

// ── 4) 打开工作空间（注入会话）→ 点击房间卡片进入编辑器 → 等待渲染 → 截图 ──
mkdirSync(OUT_DIR, { recursive: true });
const browser = await chromium.launch({ headless: true });
try {
  const context = await browser.newContext({
    viewport: { width: 1600, height: 900 },
    deviceScaleFactor: 2,
  });
  await context.addInitScript(([key, value]) => {
    window.localStorage.setItem(key, value);
  }, ["coldrawdb.auth_session.v1", JSON.stringify(session)]);

  const page = await context.newPage();
  page.on("pageerror", (e) => console.warn("[pageerror]", String(e).slice(0, 300)));
  await page.goto(`${BASE}/`, { waitUntil: "networkidle", timeout: 60_000 });
  await page.waitForSelector('[data-testid="rooms-list-page"]', { timeout: 30_000 });
  await page.locator(`[data-testid="room-card-${room.id}"]`).click();
  await page.waitForSelector('canvas[data-testid="editor-canvas"]', { state: "visible", timeout: 30_000 });
  await page.waitForTimeout(3_000); // 等待 WASM 经协作同步拉取文档并完成首帧渲染
  // 诊断：canvas 实际尺寸与可见性（应用默认 dark，无需切换主题）
  const diag = await page.evaluate(() => {
    const c = document.querySelector('canvas[data-testid="editor-canvas"]');
    const r = c?.getBoundingClientRect();
    return {
      rect: r ? { w: r.width, h: r.height, x: r.x, y: r.y } : null,
      attr: c ? { w: c.width, h: c.height } : null,
      bodyChildren: [...document.body.children].map((el) => `${el.tagName}.${el.className}`.slice(0, 80)),
      mode: document.documentElement.getAttribute("data-mode"),
    };
  });
  console.log("诊断:", JSON.stringify(diag));
  await page.screenshot({ path: OUT_FILE });
  console.log(`已生成截图: ${OUT_FILE} (diagram=${diagramId})`);
} finally {
  await browser.close();
}
