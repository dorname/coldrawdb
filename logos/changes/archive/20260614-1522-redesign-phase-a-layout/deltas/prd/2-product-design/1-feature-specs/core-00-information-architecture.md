# Delta — core-00-information-architecture.md
# 模块：core | 提案：redesign-phase-a-layout
# 路径：`logos/changes/redesign-phase-a-layout/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
# 对齐参考源：UX 重规划 Phase A（2026-06-14）

## MODIFIED — §1 顶层布局（Workspace）

> **替换** 主文档 `logos/resources/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
> §1「顶层布局（Workspace）」整节。merge 时用本节完整内容覆盖原 §1。

# 信息架构 — 顶层布局（V2 / Phase A）

## 1. 顶层布局（Workspace）

Phase A 将 V1 的「双顶栏 + 三栏（左 7 Tab / 画布 / 右属性）」重构为「单行 AppBar + Tool Rail + 画布 + 可折叠 Inspector」。

```
+--------------------------------------------------------------------------------+
| AppBar：Logo | 项目名* | ●保存状态 | [导入][导出▾] | ↶↷ | [分享] [⚙]          |
+---+----------------------------------------------------------------------------+
| T |                                                                            |
| o |                         EditorCanvas                                       |
| o |              （Table / Area / Note / Relationship）                         |
| l |                                                                            |
|   |         [空白时：居中引导卡片 — 创建表 / 导入 SQL]                           |
| R |                                                                            |
| a |                                                    ┌─────────────────────┐ |
| i |                                                    │ Inspector（可折叠）  │ |
| l |                                                    │ 选中对象属性编辑     │ |
|   |                                                    └─────────────────────┘ |
|48px                                                                            |
+---+----------------------------------------------------------------------------+
| StatusBar：缩放% | N表 M关系 | db:engine | rev:N          [Inspector 折叠按钮] |
+--------------------------------------------------------------------------------+
```

### 1.1 区域职责

| 区域 | 宽度 / 高度 | 职责 | 对应 V1 组件 |
|------|-------------|------|--------------|
| **AppBar** | 高 48px，全宽 | 项目级操作、保存反馈、导入/导出入口、撤销/重做 | `TopMenuBar` + `Toolbar` 合并 |
| **Tool Rail** | 宽 48px，纵向 | 创建工具、关系工具、平移模式、Issues 徽章 | 左栏 `LeftPanel` 7 Tab 降级 |
| **EditorCanvas** | `1fr`，主区域 | 画布渲染与直接操作（拖拽、选中、连线） | `editor_render::Canvas` |
| **Inspector** | 默认 320px，可折叠至 0 | 当前选中对象的属性编辑 | 右栏 `RightPanel` 升级 |
| **StatusBar** | 高 28px，全宽 | 缩放、统计、引擎、revision；Inspector 折叠控制 | `cdb-footer` 扩展 |

### 1.2 栅格定义

```css
/* App 顶层 */
.cdb-app {
  display: grid;
  grid-template-rows: 48px 1fr 28px;  /* AppBar / 主体 / StatusBar */
  height: 100vh;
}

/* 主体 */
.cdb-main {
  display: grid;
  grid-template-columns: 48px 1fr auto;  /* ToolRail / Canvas / Inspector */
  min-height: 0;
  overflow: hidden;
}

/* Inspector 折叠态 */
.cdb-main.cdb-is-inspector-collapsed {
  grid-template-columns: 48px 1fr 0;
}
```

- 最小视口宽度：**1024px**（与 V1 一致；< 1024px 显示「请使用更大屏幕」）
- Inspector 展开宽度：**320px**（可通过拖拽手柄调整，范围 280–480px，V2）
- Phase A 不实现 Inspector 宽度拖拽，固定 320px

### 1.3 视图模式（仅 2 种）

| 模式 | 入口 | 布局变化 |
|------|------|----------|
| **Canvas 视图**（默认） | 进入 `/editor/{id}` | 上述 V2 布局 |
| **SQL/DBML 视图** | AppBar「代码」按钮（Phase C 实现） | 全屏代码编辑器，隐藏 Tool Rail + Inspector |

Phase A 仅定义 Canvas 视图；SQL/DBML 视图入口在 AppBar 预留 `data-testid="btn-code-view"` 占位（disabled）。

### 1.4 模块边界（不变）

前端 4 模块分层与 V1 一致（`editor_data_access` → `editor_core` → `editor_panels` / `editor_render`）。

Phase A 布局变更仅影响 `editor_panels` 组件树：

```
AppRoot
├── AppBar          （原 TopMenuBar + Toolbar 合并）
├── ToolRail        （原 LeftPanel 精简）
├── CanvasContainer （原 cdb-canvas-container，含 FloatingControls）
├── Inspector       （原 RightPanel 升级，可折叠）
├── StatusBar       （原 cdb-footer 扩展）
├── ModalRoot       （不变，仅阻塞型决策）
└── ConflictDialog / ErrorToast（不变）
```

### 1.5 z-index 层级语义

| 层级 | 内容 |
|------|------|
| L0 | Canvas 对象（表、线、区域、便签） |
| L1 | 选中框 / 连接点 / 拖拽预览线 |
| L2 | 空白引导卡片 / 行内编辑 |
| L3 | Inspector 抽屉 |
| L4 | 模态（New / 删除确认 / 冲突解决） |
| L5 | Toast |

### 1.6 Phase A 边界

- ❌ Command Palette（`Ctrl+K`）— Phase D
- ❌ 导入/导出侧边抽屉 — Phase C
- ❌ 关系工具模式 + 确认条 — Phase B
- ❌ 左栏 7 Tab 浏览列表 — Phase A 移除 UI，数据与 store 保留；浏览能力 Phase D 由 Command Palette 承接
- ❌ SQL/DBML 全屏视图 — Phase C

## ADDED — §8 Phase A 迁移对照

> merge 时在主文档末尾（§7 对齐参考源之后）追加本节。

### V1 → V2 组件映射

| V1 | V2 | 迁移说明 |
|----|-----|----------|
| `TopMenuBar` + `Toolbar` | `AppBar` | 合并为单行；File 菜单保留子项，高频操作提升为按钮 |
| `LeftPanel`（7 Tab） | `ToolRail` | Tab 列表 UI 移除；Tables Tab 的「+ 建表」移至 Tool Rail `⊕` |
| `RightPanel` | `Inspector` | 增加折叠态；按选中对象类型切换内容 |
| `cdb-footer` | `StatusBar` | 增加 Inspector 折叠按钮、缩放显示 |
| 浮动 `FloatingControls` | 保留 | 位置不变（画布右下角 Zoom 控件） |

### testid 变更索引（代码批次对齐）

| 原 testid | 新 testid | 说明 |
|-----------|-----------|------|
| `top-menu-bar` | `app-bar` | AppBar 根节点 |
| `toolbar` | （移除） | 合并入 `app-bar` |
| `tab-tables` 等 7 个 | （Phase A 移除） | Phase D Command Palette 替代 |
| `editor-canvas` | `editor-canvas` | 不变 |
| （无） | `tool-rail` | 新增 |
| （无） | `inspector-panel` | 新增 |
| （无） | `btn-inspector-toggle` | StatusBar 折叠按钮 |
| （无） | `canvas-empty-guide` | 空白引导卡片 |
