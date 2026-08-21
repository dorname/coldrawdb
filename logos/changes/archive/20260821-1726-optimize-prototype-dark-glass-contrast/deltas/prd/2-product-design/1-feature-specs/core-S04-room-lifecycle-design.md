# Delta — core-S04-room-lifecycle-design.md（修改）

> module: core | proposal: optimize-prototype-dark-glass-contrast
> 仅更新色彩 / 字体 / 对比度说明，不改变任何交互语义、路由、权限规则与测试锚点。
> 色板与组件覆盖规则的实现载体为主原型 `core-01-editor-prototype.html`，对应 delta：`deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（编辑器 + 房间管理 SideSheet / 模态） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（房间列表 → 创建 → 编辑器内协作 → 邀请 / 加入） |
| 历史参考 | `core-04-collab-prototype.html`（存在未绑定演示控件，不再作为验收入口） |
| 生产实现 | 后端 rooms API/DB 与编排已实现；`frontend-rs` 房间列表、邀请和成员面板尚未接入 |
| 视觉基准 | 统一 token + 与编辑器一致的 Light/Dark 玻璃态 AppBar 栅格；Dark 模式采用高对比度暗色色板（以主原型 `html[data-mode="dark"]` token 组为准：`--bg:#050f13`、`--surface` 不透明度 .86、文字层级 `--text:#f2fdfe` / `--text-2:#b8d2d8` / `--text-3:#86a3ab`），房间卡片、空状态、用户菜单与邀请预览页文字对背景对比度均 ≥ WCAG AA 4.5:1 |
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
| 切换 dark 主题 | `data-mode` 切换；房间卡片（room-card）、新建房间卡片（new-room-card）、用户菜单（menu-item）、标签（tag/tag--brand）与邀请预览页在暗色玻璃背景下对比度 ≥ WCAG AA 4.5:1，卡片边框使用 `--line-strong`（.30）清晰可辨 |

`core-04-collab-prototype.html` 中成员按钮、ToolRail 和角色选择缺少完整事件绑定，只保留为历史参考，不纳入现行修复与验收。
