# Delta — core-S04-room-lifecycle-design.md（修改）

> module: core | proposal: improve-unified-collab-prototype | 2026-08-18
> merge 时修改原型引用，并将下列章节追加到正式规格。

## MODIFIED — 1. 产品类型与原型策略

> 替换主文档 `core-S04-room-lifecycle-design.md` §1 整节；顶部元数据中的原型路径作为阶段历史记录保留。

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（编辑器 + 房间管理 SideSheet / 模态） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（房间列表 → 创建 → 编辑器内协作 → 邀请 / 加入） |
| 历史参考 | `core-04-collab-prototype.html`（保留，不再作为验收入口） |
| 视觉基准 | 统一 token + 与编辑器一致的 Light/Dark 玻璃态 AppBar 栅格 |
| 痛点关联 | **P03** 团队评审——将「导出 JSON 邮件来回」替换为「同一 diagram 房间内协作入口」 |

## ADDED — §9 单文件房间生命周期演示

### 9.1 房间视图

登录成功后进入房间与最近项目视图。默认展示「评审周会」和「架构对齐」两个确定性演示房间，每张卡片显示关联 diagram、成员头像、当前角色、最近活动和连接状态。

创建房间必须在同一文件的 Modal 内完成名称、diagram 与默认邀请角色校验；提交后创建本地 room 状态并直接进入协作编辑器。不得依赖真实路由或刷新。

### 9.2 编辑器内房间管理

| 能力 | Owner | Editor | Viewer |
|---|---:|---:|---:|
| 编辑表/字段/关系 | 是 | 是 | 否 |
| 创建邀请 | 是 | 是 | 否 |
| 修改成员角色 | 是 | 否 | 否 |
| 移除成员 | 是 | 否 | 否 |
| 接收远端操作/presence | 是 | 是 | 是 |
| 删除或归档房间 | 是 | 否 | 否 |

角色切换由协作演示控制台触发，必须即时更新 ToolRail、Inspector、邀请按钮、StatusBar 和可发送操作的能力；Viewer 的禁用不能只靠视觉灰显，事件处理也必须阻止写操作并给出原因 Toast。

### 9.3 邀请与成员

- 邀请 Modal 可选择 Editor/Viewer、生成确定性邀请 URL、模拟复制与打开邀请预览。
- 邀请预览覆盖有效、已过期两个分支；有效邀请接受后进入相同房间，过期邀请不显示加入按钮。
- 成员 SideSheet 展示在线态、角色与最后活动；Owner 可修改 Editor/Viewer 并在确认后移除成员。
- Owner 自身角色不可直接修改或移除；需在说明中提示先转让房间。

### 9.4 锚点

保留 `rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`。浮层关闭后，不得遗留可拦截点击的遮罩。
