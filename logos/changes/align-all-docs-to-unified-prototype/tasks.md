# 实现任务

## [delta] 规格变更

### 需求与场景基线

- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-00-scenario-overview.md` — 统一 S01～S05 状态、页面流、主原型与文档映射
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-01-requirements.md` — 补齐统一工作空间、权限、协作、响应式与可访问性需求
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-04-scenario-detail.md` — 对齐 S01/S02 与 S03～S05 连续体验和验收边界

### 产品设计与功能规格

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — 固化 auth / rooms / invite / room-editor 页面状态、层级与路由
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-S01-edit-and-save-design.md` — 对齐编辑、自动保存、冲突、命令与代码视图
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-S02-load-shared-diagram-design.md` — 对齐匿名只读分享与现行页面流
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md` — 对齐登录、注册、会话状态与进入 rooms
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — 对齐房间列表、创建、邀请、成员与角色
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md` — 对齐 WS、presence、Activity、断线重连与本地降级
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 对齐画布对象、工具、状态叠层、缩放与选中
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — 对齐表卡片、字段、约束与 Inspector 编辑
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 对齐关系工具、橡皮筋、确认与关系可见状态
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — 对齐 IO 入口、抽屉、格式与预览
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 对齐 Inspector 信息架构与移动端可达性
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 对齐 AppBar、菜单、模态、成员与设置入口
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — 对齐主原型颜色、排版、间距、层级与响应式 token
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md` — 对齐可见图标、尺寸、语义和无障碍标签
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md` — 对齐按钮、Popover、Modal、SideSheet、Tag 与状态组件
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md` — 对齐 SQL / DBML / JSON 代码视图、复制与返回路径
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md` — 对齐 auth / rooms / editor 全页面主题切换
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md` — 对齐微交互、Toast、抽屉和 reduced-motion

### 技术架构与场景时序

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — 对齐页面状态、模块职责、REST/WS 状态来源与原型/生产边界
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md` — 统一 S01～S05 技术状态、依赖与规格映射
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` — 补齐主原型编辑参与者、状态反馈和异常映射
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` — 补齐匿名分享页面状态和只读边界
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S03-user-auth.md` — 对齐 Auth 页面状态、会话反馈与 rooms 跳转
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md` — 对齐 Rooms / Invite / Editor 跳转和权限反馈
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md` — 对齐协作可见状态、排队、重连和本地降级

### 测试与验收规格

- [x] 产出 delta 到 `deltas/test/core-PU-unified-prototype-test-cases.md` — 建立主原型完整交互与视觉基线
- [x] 产出 delta 到 `deltas/test/core-V2-production-frontend-test-cases.md` — 建立原型功能到真实 REST/WS 生产链路的对齐矩阵
- [x] 产出 delta 到 `deltas/test/core-S01-test-cases.md` — 对齐编辑、保存、冲突与页面反馈
- [x] 产出 delta 到 `deltas/test/core-S02-test-cases.md` — 对齐匿名分享、错误与只读体验
- [x] 产出 delta 到 `deltas/test/core-S03-test-cases.md` — 对齐鉴权 API 与页面状态验收
- [x] 产出 delta 到 `deltas/test/core-S04-test-cases.md` — 对齐房间、邀请、成员与角色验收
- [x] 产出 delta 到 `deltas/test/core-S05-test-cases.md` — 对齐 WS / OT / presence / reconnect 验收
- [x] 产出 delta 到 `deltas/test/core-CR-canvas-test-cases.md` — 对齐画布、表拖动、关系跟随和视觉状态
- [x] 产出 delta 到 `deltas/test/core-PB-relationship-test-cases.md` — 对齐关系创建与确认链路
- [x] 产出 delta 到 `deltas/test/core-PC-import-export-test-cases.md` — 对齐导入导出抽屉和格式链路
- [x] 产出 delta 到 `deltas/test/core-UI-modals-test-cases.md` — 对齐主模态、遮罩、关闭和表单状态
- [x] 产出 delta 到 `deltas/test/core-UI-modals-2-test-cases.md` — 对齐剩余模态与历史能力边界
- [x] 产出 delta 到 `deltas/test/core-SP-side-panel-test-cases.md` — 对齐 Inspector / Tab / 搜索与响应式
- [x] 产出 delta 到 `deltas/test/core-KB-shortcut-test-cases.md` — 对齐快捷键、命令面板与 Esc 行为
- [x] 产出 delta 到 `deltas/test/core-PE-design-system-test-cases.md` — 对齐 token、图标、组件、主题、动效与视觉回归
- [x] 产出 delta 到 `deltas/reference/implementation/core-frontend-alignment-acceptance.md` — 重建主原型逐项生产验收标准
- [x] 产出 delta 到 `deltas/reference/implementation/core-implementation-checklist.md` — 区分已有能力、规格待实现和第二阶段待验证

## [code] 代码实现
