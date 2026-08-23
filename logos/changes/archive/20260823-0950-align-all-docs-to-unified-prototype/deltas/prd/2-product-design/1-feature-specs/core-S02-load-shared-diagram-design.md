# Delta — core-S02-load-shared-diagram-design.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 模块：core | 场景：S02 | 原型：`core-01-editor-prototype.html`（Share 模态 + `?share=` 旁路）
> 与工作空间关系：分享只读为鉴权旁路；默认无 share 时走 auth/rooms，不再以旧 Landing 为主路径

## MODIFIED — 3.2 通过分享链接打开（冷启动）

### 3.2 通过分享链接打开（冷启动）

1. 用户访问 `/?share=abc-123-def`
2. 前端启动 → 解析 `share`（**不强制登录**）
3. 显示 loading
4. `GET /api/v1/diagrams/abc-123-def`
5. **200** → 只读渲染画布；写工具禁用；URL 保持 `share`
6. **404** → 错误提示「分享链接无效或图表已删除」；提供可达成的下一步（登录后进入工作空间 / 创建新图），**不得**假设旧 Landing「New」为唯一出口

## MODIFIED — 3.3 无 share 参数（默认 Landing）

### 3.3 无 share 参数（默认 Landing）

1. 用户访问 `/`（无参数）
2. **未登录** → 进入 auth（登录/注册），不弹分享错误
3. **已登录** → 进入 rooms
4. 不再将「Landing → 点击 New → 空白编辑器」写为现行默认主路径

## ADDED — 只读边界

- 分享只读不得被 S03 鉴权拦截阻断
- Viewer/只读态下：禁止 PUT、禁止关系拖拽创建、禁止邀请写操作
- 分享入口仍可通过编辑器内 Share 模态生成 URL（`btn-share` / `modal-share` / `share-url`）
