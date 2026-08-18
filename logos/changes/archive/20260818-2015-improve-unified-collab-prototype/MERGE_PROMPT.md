# 合并指令

## 变更提案
- 提案名称：improve-unified-collab-prototype
- 提案目录：logos/changes/improve-unified-collab-prototype/

## 提案内容

# 变更提案：improve-unified-collab-prototype

> module: core | created: 2026-08-18

## 变更原因（Why）

当前产品原型按 S01 编辑器、S03 鉴权、S04 房间生命周期、S05 OT 协作拆分在 4 个 HTML 文件中，视觉基线与交互状态也分别维护。评审者无法在一次连续体验中完成「登录 → 进入项目 → 创建/加入房间 → 编辑 ER 图 → 邀请成员 → 多人实时协作 → 断线恢复」的完整任务链，也难以判断编辑器功能与多人协作功能如何共同工作。

本变更将已设计的 S01～S05 能力收拢到一个可直接打开、无需构建和网络服务的单文件原型中，同时统一视觉层次、玻璃态设计语言、动效、响应式布局与无障碍反馈，使该文件成为后续产品评审和前端实现对照的唯一主原型。

## 变更类型

设计级（交互原型整合与视觉升级；不改变生产 API、数据库或 Rust 应用代码）

## 变更范围（What）

- 影响的需求文档：无需求语义变更；沿用 S01～S05 既有需求和验收条件
- 影响的功能规格：
  - `core-00-information-architecture.md`：补充唯一主原型入口和页面状态流
  - `core-S03-user-auth-design.md`：原型引用收敛到主原型
  - `core-S04-room-lifecycle-design.md`：原型引用收敛到主原型
  - `core-S05-ot-collab-design.md`：原型引用收敛到主原型
- 影响的页面原型：
  - `core-01-editor-prototype.html`：重构为包含 HTML、CSS、SVG 图标与 JavaScript 的独立单文件主原型
  - 现有 `core-03/04/05-*-prototype.html` 仅保留为历史参考，不再作为主评审入口
- 影响的业务场景：S01 编辑并保存、S02 分享只读、S03 鉴权、S04 房间生命周期、S05 OT 实时协作
- 影响的 API：无；原型使用浏览器内模拟数据，不发起真实 API/WS 请求
- 影响的 DB 表：无
- 影响的编排测试：无；既有 API 编排不变

## 原型功能边界

### 一体化导航与鉴权

- 登录、注册、密码显隐、字段校验、错误态、提交 loading、会话续期、退出登录
- 房间/项目列表、最近访问、创建房间、邀请接受与过期分支
- 在同一文件内通过前端状态路由切换视图，不依赖页面跳转或外部资源

### ER 编辑器核心能力

- AppBar、ToolRail、可缩放网格画布、Inspector、StatusBar、通知与模态层级
- 表/字段的创建、选择、拖拽、重命名、类型/约束编辑与删除
- 关系创建与可视连线、区域/便签入口、撤销/重做、自动保存与 revision 反馈
- 表/关系搜索，导入/导出抽屉，SQL/DBML/JSON 预览，代码视图，分享设置，主题切换与命令面板

### 多人协作交互

- 房间徽标、在线成员、角色与权限、邀请链接、成员角色修改和移除确认
- 远端光标与选区、远端创建/修改操作、Activity 时间线、server revision 与 OT 合并反馈
- 断线、操作排队、重连同步、重连失败、本地降级、Viewer 只读等完整状态机
- 所有协作行为由可重复触发的模拟控制器驱动，并在界面中明确标注为原型模拟

### 视觉与体验

- 采用设计 token、柔和渐变背景、半透明玻璃面板、背景模糊、精细描边和层次阴影
- Light/Dark 双主题；遵守 `prefers-reduced-motion`；窄屏提供可用的折叠与抽屉布局
- 使用内联 SVG 图标，不依赖 emoji、外部字体、CDN、共享 CSS 或图片资源
- 完整 hover/focus/active/disabled/loading/empty/error 状态，并保留关键 `data-testid` 锚点

## 验收标准

- **PU-AC-01 单文件**：断网直接打开 `core-01-editor-prototype.html`，样式、图标、数据和全部交互均可用；文件不引用本地/远程 CSS、JS、字体或图片。
- **PU-AC-02 连续主链**：可在单文件内完成注册或登录 → 房间列表 → 进入协作编辑器 → 新建表/字段 → 创建关系 → 邀请成员 → 模拟远端操作 → 导出结果。
- **PU-AC-03 编辑完整性**：表与字段的新增、编辑、拖拽、删除、撤销、重做、自动保存和 revision 在 UI 中形成闭环。
- **PU-AC-04 协作完整性**：Owner/Editor/Viewer 权限差异、远端光标、Activity、OT revision、断线排队、恢复同步与失败降级均可演示。
- **PU-AC-05 浮层完整性**：导入/导出、代码视图、分享、邀请、成员管理、设置、确认、命令面板均可打开和关闭，且不会遗留遮罩拦截界面。
- **PU-AC-06 视觉质量**：Light/Dark 下文字对比清晰，玻璃态不损害可读性；桌面和窄屏无关键操作不可达。
- **PU-AC-07 可访问性**：关键控件有可感知标签和键盘焦点；Escape 可关闭顶层浮层；降低动态偏好下关闭非必要动画。
- **PU-AC-08 可诊断性**：原型内置只读诊断入口，能够检查关键 DOM 锚点、单文件依赖、浮层状态和协作状态数据。

## 部署影响

- 是否需要部署：否
- 部署原因：仅修改 OpenLogos 产品设计文档与静态原型，不进入运行时应用包
- 影响环境：本地评审
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（Git 可直接回退文档变更）
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: unified-editor
    prototype: core-01-editor-prototype.html
    description: S01～S05 单文件一体化 ER 编辑与多人协作主原型
```

## 变更概述（How）

以 `core-01-editor-prototype.html` 为唯一主原型，在文件内建立轻量状态仓库、视图状态机、编辑命令栈和协作连接状态机。所有交互通过事件委托和可预测的模拟数据运行；编辑命令同时驱动画布、Inspector、保存状态、revision 与 Activity，使演示状态保持一致。

视觉层以现有 `--cdb-*` token 为基础重新组织，统一 AppBar、ToolRail、Canvas、Inspector、SideSheet、Modal、Popover、Toast 和 StatusBar 的玻璃态层级。保留 S01～S05 的关键 `data-testid`，并增加内置诊断函数用于静态与浏览器验收。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/test/core-PU-unified-prototype-test-cases.md

- Delta 文件：`logos/changes/improve-unified-collab-prototype/deltas/test/core-PU-unified-prototype-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(improve-unified-collab-prototype): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive improve-unified-collab-prototype`。
