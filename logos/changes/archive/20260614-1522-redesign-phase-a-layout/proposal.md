# 变更提案：redesign-phase-a-layout

> module: core | created: 2026-06-14
> 前置：`wire-editor-canvas` 代码批次已完成，需归档后再激活本提案 guard

## 变更原因

基于 UX 交互重规划（2026-06-14 评审），现有 V1 界面存在以下结构性问题：

1. **三栏 + 7 Tab 左栏**认知负荷过高，主任务（建表 / 编字段 / 连关系）被 Areas / Enums / Notes / Types 等次要功能稀释。
2. **顶栏双行**（菜单栏 + 工具栏）占用垂直空间，保存 / 导入 / 导出等项目级操作埋在 File 菜单深处。
3. **编辑入口分散**：画布、左栏列表、右栏属性、模态四处可改，选中态语义不一致。
4. **空白画布无引导**，新用户不知从何开始。

本提案 **Phase A** 聚焦布局重构与选中态统一，为后续 Phase B（关系工具）、Phase C（导入导出抽屉）、Phase D（Command Palette）奠定基础。

## 变更类型

设计级变更（Phase A 仅产出规格 delta，代码实现留待 merge 后单独批次）

## 变更范围

- 影响的需求文档：无（交互优化，不改变 S01/S02 业务验收语义）
- 影响的功能规格：
  - `core-00-information-architecture.md` §1 顶层布局
  - `core-01-editor-canvas.md` §5.2.3 布局栅格、§6 联动规则；新增空白引导、选中态模型
  - `core-04-side-panel-tabs.md` §1 侧栏布局（左栏降级为 Tool Rail）
  - `core-05-top-menu-modals.md` §1 顶栏布局（双行合并为 AppBar）
  - **新增** `core-06-inspector-panel.md`（Inspector 抽屉规格）
  - `core-01-editor-prototype.html`（静态原型对齐新布局）
- 影响的业务场景：S01（编辑保存）、S02（加载分享）— 交互路径变更，API 不变
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：HP-01~HP-05 选择器可能需随 testid 调整（Phase A delta 预埋，代码批次再改）

## 部署影响

- 是否需要部署：否（Phase A 仅规格文档）
- 部署原因：无代码变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

### Phase A 交付物（本提案范围）

1. **单行 AppBar**：合并原顶栏菜单 + 工具栏；项目名、保存状态、导入/导出、撤销/重做常驻可见。
2. **Tool Rail（48px 左轨）**：替代 280px 左栏 + 7 Tab；图标化暴露「新建表 / 区域 / 便签 / 关系工具 / 平移」；Issues 以徽章呈现。
3. **可折叠 Inspector 抽屉**：替代固定 300px 右栏；无选中时折叠；选中表/字段/关系时展开属性编辑。
4. **选中态统一模型**：画布选中 ↔ Inspector 内容 ↔ 列表跳转三处同步。
5. **空白画布引导卡片**：零表时居中显示「创建第一张表 / 导入 SQL」入口。

### 不在 Phase A 范围（后续提案）

- 关系工具模式 + 确认条（Phase B）
- 导入/导出侧边抽屉 + 预览（Phase C）
- Command Palette `Ctrl+K`（Phase D）
- 左栏 7 Tab 浏览能力迁移至 Command Palette（Phase D）
