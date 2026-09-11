# Delta — core-S02-load-shared-diagram.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> Phase 2 输入：`core-S02-load-shared-diagram-design.md` | 原型：`core-01-editor-prototype.html`（Share 模态 + `?share=`）
> 页面状态：`share-readonly`（鉴权旁路）；与默认 `auth → rooms → room-editor` 主路径并行
> API/DB：本提案不新增端点；仅补页面状态与只读边界映射

## MODIFIED — 1. 场景描述

**成功标志**：编辑器加载完整 diagram；URL 保持 `share` 参数；画布 **只读**（写工具禁用）。

**废止**：原文「可继续编辑」作为分享链接默认成功语义 → 改为 **匿名只读**（与 Phase 2 / 主原型一致）。若用户需写权限，须走 S03 登录 + S04 房间成员路径，而非分享旁路。

## MODIFIED — 3.2 无 share 参数 — Landing 默认路径

### 3.2 无 share 参数 — Landing 默认路径

替换「Landing 或空白编辑器 / New → POST diagrams」为现行默认：

1. 用户访问 `/`（无 query）→ **不**弹分享错误
2. **未登录** → 进入 `auth`（登录/注册）
3. **已登录** → 进入 `rooms`
4. 不再将「Landing → New → 空白 `/editor`」写为现行默认主路径

## ADDED — 页面状态与参与者

| 角色 | 模块 | 说明 |
|---|---|---|
| Router / Entry | `frontend-rs` `lib.rs` | 解析 `?share=`；**跳过**鉴权拦截 |
| EditorDataAccess | `editor_data_access` | `GET /api/v1/diagrams/{id}`（匿名） |
| EditorCore | `editor_core` | `set_diagram` + `readonly=true` |
| ShareModal | AppBar | `[data-testid="modal-share"]` / `share-url`（生成旁路链接） |

主原型演示加载失败/成功；生产以真实 GET 为准。

## ADDED — 异常映射（前端）

| 条件 | 前端 | 下一步 |
|---|---|---|
| 200 | 只读渲染；禁用 PUT / 关系创建 / 邀请写 | 保持 `?share=` |
| 404 | Toast「分享链接无效或图表已删除」 | 登录后进 rooms / 可达的创建入口（不得假设旧 Landing New） |
| 网络失败 | 加载失败 + 重试 | 同 S01 退避策略 |
| 非法 UUID | 前端拦截，不发请求 | 「无效链接」 |

## MODIFIED — 9. V1 边界

- ✅ `?share=` **不被** S03 鉴权拦截（旁路）
- ❌ 分享链接不授予 room 写权限（写权限仍走 S04 成员）
- ❌ 链接过期 / 访问统计 — 仍 Out of Scope（除非独立变更）

## MODIFIED — 10. 对齐参考源

- `core-00-information-architecture.md` — `share-readonly` 状态
- `core-S03-user-auth.md` — 旁路与私有路由区分
