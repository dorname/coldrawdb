# S02：加载分享链接图表 — 交互设计

> 模块：core | 场景：S02 | 原型：`core-01-editor-prototype.html` §Share 模态
> 参考：drawdb `origin/main` → `Workspace.jsx`（`useSearchParams` + `get(gistId)`）/ `Modal/Share.jsx`
> Phase 1 输入：`core-04-scenario-detail.md` §S02

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用 |
| 原型形式 | HTML 模态 + URL 参数说明（drawdb 用 Gist；coldrawdb 用 `?share=<id>`） |
| 对齐差异 | drawdb main 通过 GitHub Gist + `shareId` 查询参数；coldrawdb V1 改为自建 API + `?share=` |

## 2. 涉及页面与元素

| 元素 | drawdb main | coldrawdb |
|---|---|---|
| 分享入口 | ControlPanel → Share 按钮 | AppBar `[data-testid="btn-share"]` |
| 分享模态 | `Modal/Share.jsx` | `[data-testid="modal-share"]` |
| 分享 URL | Gist URL | `https://coldrawdb.example.com/?share=<diagram_id>` |
| 加载逻辑 | `Workspace.jsx` 解析 `shareId` → `get()` | `editor_data_access` 解析 `share` → `GET /api/v1/diagrams/{id}` |

## 3. 交互流程

### 3.1 生成分享链接（编辑器内）

1. 用户点击 AppBar「分享」→ 打开 Share 模态
2. 模态展示只读 URL 输入框 `[data-testid="share-url"]`
3. 用户点击「复制链接」→ Toast「已复制」（实装时）
4. ESC / × / 背景点击关闭模态（遮罩必须从 DOM 移除）

### 3.2 通过分享链接打开（冷启动）

1. 用户访问 `/?share=abc-123-def`
2. WASM 启动 → 解析 URL 参数 `share`
3. 显示 loading（< 200ms 骨架或 spinner）
4. `GET /api/v1/diagrams/abc-123-def`
5. **200** → 渲染 diagram 到画布；URL 保持 `share` 参数
6. **404** → 错误 Toast + 「创建新图表」按钮 → 跳转 `/editor`

### 3.3 无 share 参数（默认 Landing）

1. 用户访问 `/`（无参数）
2. 不弹错误；展示 Landing 或空白编辑器
3. 用户点击「New」→ `POST /api/v1/diagrams` → 跳转 `/editor/{id}`

## 4. 验收条件（交互级）

##### 正常：生成分享链接

- **GIVEN** 用户在编辑器，diagram 已保存
- **WHEN** 用户点击 `[data-testid="btn-share"]`
- **THEN**
  - `[data-testid="modal-share"]` 打开（`.cdb-is-open`）
  - `[data-testid="share-url"]` 含 `?share=` 与当前 diagram id
  - 点击「复制链接」后模态可关闭，画布仍可交互（遮罩已移除）

##### 正常：有效分享链接加载

- **GIVEN** 服务端存在 id=`abc-123-def` 的 diagram（含 3 张表）
- **WHEN** 用户访问 `/?share=abc-123-def`
- **THEN**
  - 页面 loading < 200ms 后出现画布
  - 3 张表渲染到 `[data-testid="editor-canvas"]`
  - URL 仍含 `share=abc-123-def`

##### 异常：无效分享链接

- **GIVEN** id=`xxx-nonexistent` 不存在
- **WHEN** 用户访问 `/?share=xxx-nonexistent`
- **THEN**
  - 错误提示「分享链接无效或图表已删除」
  - 提供「创建新图表」入口
  - 画布保持空白，不 crash

##### 异常：share 参数缺失

- **GIVEN** 用户访问 `/` 无 query
- **WHEN** 页面加载完成
- **THEN** 不弹错误；可走 Landing → New 流程

## 5. 与 drawdb main 的对照

| drawdb main 行为 | coldrawdb V1 设计 |
|---|---|
| Gist API 存储 diagram JSON | SQLite + REST `PUT/GET /diagrams/{id}` |
| `shareId` URL 参数 | `share` URL 参数 |
| 客户端 Dexie 缓存列表（Open 模态） | 服务端 diagram 列表（V1 简化为 id 输入 / New） |
| 无 revision 冲突 | 乐观锁 + 409 模态（见 S01 设计） |

## 6. 原型操作指南

| 操作 | 预期 |
|---|---|
| 点击「分享」 | 打开 Share 模态，显示示例 URL |
| 点击 × / 关闭 | 模态关闭，可继续编辑画布 |
| （实装）访问 `?share=` URL | 按 §3.2 加载；原型仅演示模态生成链接 |
