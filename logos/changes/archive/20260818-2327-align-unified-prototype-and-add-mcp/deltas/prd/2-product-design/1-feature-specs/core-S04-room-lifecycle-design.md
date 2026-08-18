# Delta — core-S04-room-lifecycle-design.md（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（编辑器 + 房间管理 SideSheet / 模态） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（房间列表 → 创建 → 编辑器内协作 → 邀请 / 加入） |
| 历史参考 | `core-04-collab-prototype.html`（存在未绑定演示控件，不再作为验收入口） |
| 生产实现 | 后端 rooms API/DB 与编排已实现；`frontend-rs` 房间列表、邀请和成员面板尚未接入 |
| 视觉基准 | 统一 token + 与编辑器一致的 Light/Dark 玻璃态 AppBar 栅格 |
| 痛点关联 | **P03** 团队评审——将「导出 JSON 邮件来回」替换为「同一 diagram 房间内协作入口」 |

## MODIFIED — 6. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认登录后 | 房间列表 |
| 「创建房间」 | 模态 → 提交 → 进入 room 编辑器视图 |
| 「邀请成员」 | 邀请模态 + 复制链接 |
| 「模拟 Viewer 视角」 | 切换只读 UI |
| 「接受邀请」视图 | 邀请预览页 → 加入 → 编辑器 |
| 「成员管理」 | SideSheet 改 role / 移除 |

`core-04-collab-prototype.html` 中成员按钮、ToolRail 和角色选择缺少完整事件绑定，只保留为历史参考，不纳入现行修复与验收。

## ADDED — 8. 生产实现状态

主原型使用本地模拟房间数据；真实 room CRUD、邀请、角色权限由 backend 提供，但生产前端尚未调用。规格中的 `data-testid` 是后续前端接入目标，不得据此把 S04 标记为全栈完成。

