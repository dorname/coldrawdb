# 解决方案 03：Browser 页面级 E2E 测试用例

## 1. 目标与范围

### 目标

- 为前端各页面建立 **可运行的 E2E 测试**，在真实浏览器中访问路由并断言关键元素与行为。
- 按 **页面/路由** 生成可维护的用例骨架，便于后续补充交互与边界用例。

### 范围

- 覆盖 `src/App.jsx` 中定义的路由：
  - `/` — LandingPage
  - `/editor` — Editor（主编辑器）
  - `/bug-report` — BugReport
  - `/templates` — Templates
  - `*` — NotFound
- 每个页面至少 1 个「访问 + 关键元素可见」用例；核心页面（如 Editor）可增加 1～2 个关键交互（如画布/侧栏可见、进入编辑器）。

---

## 2. 前置条件

### 当前状态

- `package.json` 中 **无 test / e2e 脚本**，未引入 Playwright、Cypress 或 Vitest。
- 运行 E2E 需 **同时启动前端与后端**：前端 `npm run dev`（默认如 `http://localhost:5173`），后端 `cargo run`（默认如 `localhost:6666`）；且 `vite.config.js` 将 `/api`、`/diagrams`、`/tables` 等代理到后端，否则编辑器加载/保存会失败。

### 依赖

- 本地或 CI 中需能同时起 frontend + backend，或使用 Docker Compose 一键启动后再跑 E2E。

---

## 3. 技术选型

- **推荐**：**Playwright**（与 Cursor browser MCP 兼容性好、多浏览器、选择器稳定）。
- **备选**：Cypress（对前端开发者友好，单浏览器为主）。
- **用例组织**：按页面分文件，便于维护：
  - `e2e/landing.spec.js`
  - `e2e/editor.spec.js`
  - `e2e/templates.spec.js`
  - `e2e/bug-report.spec.js`
  - `e2e/not-found.spec.js`

---

## 4. 分步实施

| 步骤 | 内容 |
|------|------|
| **Step 1** | 安装 Playwright：`npm i -D @playwright/test`，执行 `npx playwright install`；在 `package.json` 中增加脚本 `"test:e2e": "playwright test"`。 |
| **Step 2** | 配置 `playwright.config.js`：`baseURL` 指向本地 dev（如 `http://localhost:5173`），配置 timeout、retries；若需后端，在 CI 中先启动 backend、再启动 frontend、再执行 `playwright test`。 |
| **Step 3** | 按路由生成各 spec 文件骨架：每个文件一个 `describe(页面名)`，至少 1 个 `test('访问 + 关键元素可见')`（`page.goto(path)` -> `expect(locator).toBeVisible()`）；Editor 等复杂页指定关键选择器（见 Step 4）。 |
| **Step 4** | 在关键组件上增加 **data-testid**（如 `data-testid="editor-canvas"`、`data-testid="templates-list"`），在本文档附录中列出选择器约定，便于后续维护与稳定断言。 |
| **Step 5** | 编写最小用例集：Landing 首屏 CTA/logo 可见；Editor 画布或侧栏可见；Templates 列表或占位可见；BugReport 表单区域可见；访问 `/non-existent` 出现 404 文案或组件。 |
| **Step 6** | （可选）在 CI 中增加 job：启动 backend + frontend，再执行 `npm run test:e2e`。 |

---

## 5. 验收标准

- **E2E**：本地执行 `npm run test:e2e` 仅跑页面级冒烟（landing、not-found、templates、editor.smoke），全部通过；CRUD 不由 E2E 执行。
- **CRUD 联调**：通过 Cursor Browser 按 5.1 节步骤在本地验证通过。
- 文档中列出每页对应的用例名称与预期行为表（见附录）。

---

## 5.1 CRUD 与基础前后端联调（Cursor Browser 验证）

CRUD 与基础前后端联调 **不由 Playwright E2E 执行**，改为使用 **Cursor 自带 Browser**（Browser MCP）在本地进行联调验证。E2E 仅保留页面级冒烟（Landing、NotFound、Templates、Editor 可进入），`npm run test:e2e` 不包含 CRUD 用例。

### 目标与范围

- **目标**：在本地联调环境下，通过 Cursor Browser 人工/辅助操作验证前端（Editor/Templates）与后端（`/diagrams`、`/templates` 等）可稳定完成基础 CRUD。
- **范围**：以 **Editor 图表 CRUD** 为主，`/templates` 列表与打开为辅。

### Cursor Browser CRUD 联调步骤

- **前置**：本地已启动后端（`cargo run`）与前端（`npm run dev`），Vite 代理正常（如 `/api` → `localhost:6666`）。
- 在 Cursor 中打开 Browser，访问 `http://localhost:5173/editor`。
- **Create**：在编辑器中点击保存（或等待 autosave），在 Network 中确认 `POST /diagrams/add`（或等价接口）返回 200，且响应 body 中 `code === 200`、`data.id` 存在。
- **Read**：通过 Open 输入上一步得到的 diagram id，或刷新后确认加载最新；确认 `GET /diagrams/query/{id}` 或 `/diagrams/latest` 返回 200。
- **Update**：修改标题后保存，确认 `POST /diagrams/update` 200；可刷新或重新打开验证标题已持久化。
- **Delete**：File → Delete diagram → 确认，确认 `DELETE /diagrams/delete/{id}` 200，页面回到空白/默认态。
- **Templates**：访问 `/templates`，确认 `GET /templates/queryAll` 成功、列表区域有内容或无报错；可选点击某模板 Edit/Fork，确认 `GET /templates/query/{id}` 成功且 Editor 加载模板内容。

### 联调检查清单（Cursor Browser 联调时勾选）

- [ ] **图表 Create**：保存后 Network 有 `POST /diagrams/add` 200，`data.id` 存在
- [ ] **图表 Read**：Open 或加载最新后 `GET /diagrams/query/:id` 或 `/diagrams/latest` 200
- [ ] **图表 Update**：修改后保存有 `POST /diagrams/update` 200，刷新后数据一致
- [ ] **图表 Delete**：删除后有 `DELETE /diagrams/delete/:id` 200，页面重置
- [ ] **Templates 列表**：`/templates` 页有 `GET /templates/queryAll` 200，列表可见
- [ ] **（可选）Template 打开**：Edit/Fork 后 `GET /templates/query/:id` 200，Editor 展示模板

### Cursor Browser CRUD 联调脚本模版（口令）

为了减少每次联调时重复描述步骤，可以约定一个「口令 + 长提示」模版，今后只需一句话即可驱动 Cursor Browser 完整跑完 CRUD 联调：

- **推荐口令示例**：  
  - 「请按我们约定的 Cursor Browser CRUD 联调脚本，完整跑一遍 Editor + Templates 的 CRUD 联调检查，并给我勾选式总结。」

- **长提示模版核心内容**（可在需要时贴给助手，或固化到 `.cursor/rules` 中）：  
  1. **前置要求**：本地已启动后端 `cargo run`（`http://localhost:6666`）与前端 `npm run dev`（`http://localhost:5173`），Vite 代理正常；只使用 Browser MCP，不改代码、不重启服务。  
  2. **Create**：在 `http://localhost:5173/editor` 中点击保存，在 Network 中找到并校验 `POST /diagrams/add` 200，`code === 200` 且 `data.id` 存在，并记为 `diagramId`。  
  3. **Read**：刷新或通过 Open 打开 `diagramId`，确认有 `GET /diagrams/query/{diagramId}` 或 `/diagrams/latest` 200，`code === 200`，页面内容与创建时一致。  
  4. **Update**：修改标题为唯一值（如 `Cursor Browser CRUD Test`），再次保存，确认 `POST /diagrams/update` 200，`code === 200`，刷新/重开后标题为新值。  
  5. **Delete**：通过 `File -> Delete diagram` 删除该图，确认 `DELETE /diagrams/delete/{diagramId}` 200，`code === 200`，Editor 回到空白/默认态。  
  6. **Templates**：访问 `/templates`，确认 `GET /templates/queryAll` 200、`code === 200`，页面无错误；可选点击 Edit/Fork，确认 `GET /templates/query/{id}` 200 且 Editor 展示模板内容。  
  7. **输出格式**：最后用 checklist 形式输出每一步是否通过（`[x]/[ ]`），并在失败时附上 HTTP 状态码、关键响应字段及页面表现的简要说明。

有了上述口令与模版后，你只需在对话中引用它，助手即可使用 Cursor Browser 自动完成一轮标准化 CRUD 联调自检。

## 6. 参考与附录

### 附录 A：页面与路径、建议 data-testid、用例名称一览表

| 页面 | 路径 | 建议 data-testid（待在组件上添加） | 建议用例名称 |
|------|------|-----------------------------------|--------------|
| Landing | `/` | `landing-hero` 或 `landing-cta` | 首屏 CTA 或 logo 可见 |
| Editor | `/editor` | `editor-canvas`、`editor-side-panel` | 画布或侧栏可见；可选：进入编辑器后无报错 |
| Templates | `/templates` | `templates-list` 或 `templates-container` | 模板列表或占位可见 |
| BugReport | `/bug-report` | `bug-report-form` | 表单或提交区域可见 |
| NotFound | `/non-existent` | 无需 testid，可用文案 | 出现 404 文案（如 "looking for something"） |

### 附录 B：选择器约定

- **优先**：`data-testid="..."`，避免依赖 class 或 DOM 结构变化。
- **次选**：稳定的 role + name，如 `getByRole('link', { name: /docs/i })`。
- **避免**：仅靠 class 或深层 DOM 路径（易随样式/重构失效）。

补充（CRUD/联调用例建议）：

- **CRUD 相关建议 testid（示例）**：
  - `editor-save`：保存按钮/菜单项
  - `editor-rename`：Rename 入口
  - `editor-rename-input`：Rename 输入框
  - `editor-delete-diagram`：Delete diagram 入口
  - `editor-confirm-ok`：确认弹窗 OK
  - `templates-list`：模板列表容器
- **网络断言优先**：优先用 `page.waitForResponse()` 等待并断言对应接口返回（例如 `/diagrams/add`、`/diagrams/update`、`/diagrams/delete/`、`/templates/queryAll`），避免仅用延时等待 UI。

### 附录 C：示例 — e2e/editor.spec.js

```javascript
// @ts-check
const { test, expect } = require("@playwright/test");

test.describe("Editor", () => {
  test("访问 /editor 后画布或侧栏可见", async ({ page }) => {
    await page.goto("/editor");

    // 若已添加 data-testid="editor-canvas" 或 "editor-side-panel"
    const canvas = page.getByTestId("editor-canvas");
    const sidePanel = page.getByTestId("editor-side-panel");
    await expect(canvas.or(sidePanel)).toBeVisible({ timeout: 10000 });

    // 若无 testid，可用备用选择器（根据实际 DOM 调整）
    // await expect(page.locator(".theme").first()).toBeVisible({ timeout: 10000 });
  });
});
```

### 附录 D：最小用例集与预期行为

| 用例 | 路径 | 操作 | 预期 |
|------|------|------|------|
| Landing 首屏可见 | `/` | goto + 等待 | CTA 或 logo 或主标题可见 |
| Editor 可进入 | `/editor` | goto + 等待 | 画布或侧栏可见，无白屏 |
| Templates 列表可见 | `/templates` | goto + 等待 | 模板列表或占位区域可见 |
| BugReport 表单可见 | `/bug-report` | goto + 等待 | 表单或提交区域可见 |
| NotFound 展示 | `/non-existent` | goto | 出现 "looking for something" 等 404 文案或链接 |

### 附录 E：CRUD/联调验收补充

- **联调验收**：CRUD 联调通过 Cursor Browser 按 5.1 节「Cursor Browser CRUD 联调步骤」及「联调检查清单」在本地验证通过；E2E 仅需通过页面级冒烟。
- **建议断言**：在 Browser 的 Network 中确认 HTTP 200 及响应 body 中 `code === 200`、`data.id` 等关键字段。

### 相关代码位置

- 路由定义：`src/App.jsx`
- 页面组件：`src/pages/LandingPage.jsx`、`Editor.jsx`、`Templates.jsx`、`BugReport.jsx`、`NotFound.jsx`
- 编辑器框架：`src/components/Workspace.jsx`（可在此或子组件上加 data-testid）
- Vite 代理：`vite.config.js`
