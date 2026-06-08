## ADDED — 编辑器画布总规格

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
> 对齐参考源：drawdb §2.3 + CAP-CANVAS-01..09

# 编辑器画布总规格（V1）

## 1. 概述

编辑器画布是 V1 的核心交互区域。**所有画布对象在坐标系 `{x, y}` 中定位**，由 `editor-render` 模块渲染为 SVG / Canvas 元素。

## 2. 画布对象清单

| 对象 ID | 类型 | 渲染组件 | 交互能力 | 对齐 drawdb |
|---|---|---|---|---|
| OBJ-Table | `Table` | `<Table>` 矩形 + 字段列表 | 拖拽 / 缩放 / 编辑字段 | CAP-CANVAS-01 |
| OBJ-Field | `Field` | `<Table>` 内的行 | 编辑 / 删除 / 排序 | CAP-CANVAS-02 |
| OBJ-Relationship | `Relationship` | 贝塞尔 `<path>` + 端点标签 | 拖拽端点 / 编辑 | CAP-CANVAS-03 |
| OBJ-Index | `Index` | 嵌入 `Table` 元数据 | 编辑 | CAP-CANVAS-04 |
| OBJ-Area | `Area` | `<Area>` 矩形 + 标签 | 拖拽 / 缩放 | CAP-CANVAS-05 |
| OBJ-Note | `Note` | `<Note>` 富文本便签 | 编辑 / 拖拽 | CAP-CANVAS-06 |
| OBJ-Enum | `Enum` | 侧栏管理 | 编辑枚举值 | CAP-CANVAS-07（V1 仅前端状态） |
| OBJ-CustomType | `CustomType` | 顶部菜单管理 | 编辑类型 | CAP-CANVAS-08（V1 仅前端状态） |
| OBJ-Canvas | `Canvas` | 容器 | 平移 / 缩放 / 框选 | CAP-CANVAS-09 |

## 3. 交互手势

| 手势 | 效果 |
|---|---|
| 鼠标左键单击 | 选中对象（高亮） |
| 鼠标左键拖拽空白 | 框选多个对象 |
| 鼠标左键拖拽对象 | 移动对象 |
| 鼠标左键双击对象 | 进入编辑模式 |
| 鼠标右键 | 上下文菜单（编辑/删除/复制） |
| 滚轮 | 画布缩放（中心点 = 鼠标位置） |
| 空格 + 拖拽 | 画布平移 |
| Delete / Backspace | 删除选中对象 |
| Ctrl/Cmd + Z | 撤销 |
| Ctrl/Cmd + Shift + Z | 重做 |
| Ctrl/Cmd + D | 复制选中 |

## 4. 坐标系

- 坐标系：屏幕坐标系（x 向右、y 向下）
- 单位：像素
- 缩放范围：0.25x ~ 4x
- 缩放中心：默认鼠标位置（CAP-EDIT 模式）；可切换为画布中心
- 持久化：`{x, y}` 与对象状态一同存入 `core_diagram` JSON 字段

## 5. 渲染策略（coldrawdb V1）

### 5.1 渲染主体

- **render 层**：`frontend-rs/editor_render` 使用 `<canvas>`（HTML5）+ 贝塞尔连线（自渲染，无 vDOM diff）
- **响应式**：基于 Leptos signals 细粒度更新（仅重绘变更部分）
- **性能预算**：100 张表 / 200 条关系 / 60fps（来源：Phase 4 W4 perf）

### 5.2 样式底座（CSS Design Tokens + cdb- 前缀规则）— V1 必交付

#### 5.2.1 文件位置与加载

- 样式文件：**唯一** — `frontend-rs/src/styles.css`
- 加载方式：在 `frontend-rs/index.html` 头部 `<link rel="stylesheet" href="styles.css">`（Trunk 自动处理）
- **禁止**散落样式：组件内联 `style=`、散落 `<style>` 块、`!important` 覆盖

#### 5.2.2 设计 Token（CSS 变量）

定义在 `:root`，所有组件通过 `var(--*)` 引用：

```css
:root {
  /* 颜色 — 主色 coldrawdb teal（与 Logo 同色 #175e7a） */
  --cdb-color-primary: #175e7a;
  --cdb-color-primary-hover: #134c63;
  --cdb-color-primary-bg: #e6f1f5;

  /* 颜色 — 语义 */
  --cdb-color-success: #10b981;
  --cdb-color-warning: #f59e0b;
  --cdb-color-error: #ef4444;

  /* 颜色 — 中性 */
  --cdb-color-text: #1f2937;
  --cdb-color-text-muted: #6b7280;
  --cdb-color-border: #e5e7eb;
  --cdb-color-bg: #ffffff;
  --cdb-color-bg-subtle: #f9fafb;
  --cdb-color-bg-canvas: #f3f4f6;

  /* 间距（4px 栅格） */
  --cdb-space-1: 4px;
  --cdb-space-2: 8px;
  --cdb-space-3: 12px;
  --cdb-space-4: 16px;
  --cdb-space-6: 24px;
  --cdb-space-8: 32px;

  /* 字号 */
  --cdb-text-xs: 11px;
  --cdb-text-sm: 12px;
  --cdb-text-base: 14px;
  --cdb-text-lg: 16px;
  --cdb-text-xl: 18px;

  /* 圆角 */
  --cdb-radius-sm: 4px;
  --cdb-radius-md: 6px;
  --cdb-radius-lg: 8px;

  /* 阴影 */
  --cdb-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --cdb-shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07);
  --cdb-shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);
}
```

#### 5.2.3 布局栅格

- App 顶层：`display: grid; grid-template-rows: auto auto 1fr auto;`（顶栏 / 工具栏 / 主体 / 状态栏）
- 主体：`display: grid; grid-template-columns: 240px 1fr 320px;`（左栏 / 画布 / 右栏）
- 最小宽度 1024px，< 1024px 提示「请使用更大屏幕」（V1 不做响应式）

#### 5.2.4 `.cdb-*` 前缀规则

- **强制**：所有自定义 class 必须以 `cdb-` 开头（coldrawdb）
- **命名空间**：
  - `cdb-` 前缀 = 组件（如 `cdb-topbar`、`cdb-modal-overlay`）
  - `cdb-is-` 前缀 = 状态（如 `cdb-is-selected`、`cdb-is-open`）
  - `cdb-` 后 BEM：块 `cdb-modal` / 元素 `cdb-modal__header` / 修饰 `cdb-modal--lg`
- **禁止**：使用 Tailwind 工具类、无前缀的全局类

#### 5.2.5 与 Leptos class 属性的接驳

`view! { <div class="cdb-topbar"> }` 等同于 React 风格 className；Leptos 0.5 的 `class:` 条件语法生成复合 class。

#### 5.2.6 验收要点

| 验证项 | 工具 | 期望 |
|---|---|---|
| 仅一个 styles.css | `find frontend-rs -name "*.css" -not -path "*/node_modules/*"` | 仅 1 个匹配 |
| 无内联 style | `grep -rn 'style="' frontend-rs/src/` | 0 匹配 |
| 所有 class 带 cdb- 前缀 | `grep -rhoE 'class="[^"]+"' frontend-rs/src/ \| grep -oE '"[a-z][a-z0-9_-]*' \| sort -u \| grep -v '^"cdb-'` | 0 匹配 |
| 设计 token 全部使用 | `grep -c 'var(--cdb-' frontend-rs/src/styles.css` | ≥ 30 |

### 5.3 Areas / Notes / References 渲染（V1 必交付，连接 store）

- 当前 `editor_render::leptos_canvas::Canvas` 传空 `areas: Vec::new>()` / `notes: Vec::new>()`
- 目标：改为 `store.areas.get()` / `store.notes.get()`（需在 `EditorStore` 新增对应 RwSignal）
- `draw_area` / `draw_note` / `draw_bezier` 函数已存在，**复用** 不重写
- references 端点拖拽改 start/end_field_id（spec CAP-EDIT-02）

### 5.4 V1 边界（渲染层）

- ❌ CSS-in-JS 运行时（V1 用纯 CSS，避免 wasm 体积膨胀）
- ❌ 主题切换（V1 单一 light 主题）
- ❌ 暗色模式（V2）
- ❌ 自定义字体加载（V1 走系统字体栈）
- ❌ CSS 动画（V1 无 transition / animation，避免影响 60fps 性能预算）

## 6. 与侧栏 / 顶部菜单的联动

- 单击侧栏 Tables Tab 中的某表 → 画布中该表高亮并滚动到视口
- 单击 Relationships Tab 中的某关系 → 画布中该关系闪烁
- 顶部菜单"全选"→ 画布中所有对象选中
- 顶部菜单"删除" → 画布中删除选中对象

## 7. 撤销 / 重做集成

所有画布对象的修改（包括创建、编辑、移动、删除）都进入 `editor_core` 的 `UndoRedoContext`（CAP-EDIT-01）：

- 撤销栈深度：默认 50 步
- 不持久化（仅内存；进程结束即丢）
- 不支持协作 undo（V1 无协作）

## 8. V1 边界

- ❌ 自动布局算法（drawdb 有，V1 不实现）
- ❌ 多选移动时自动吸附到网格（V1 不实现）
- ❌ 触控板手势（V1 仅鼠标）
- ❌ 离线渲染（V1 完全依赖后端 API）

## 9. 详细规格

| 主题 | 详见 |
|---|---|
| 表与字段编辑 | `core-01a-table-and-field.md` |
| 关系编辑 | `core-01b-relationship.md` |
| 索引 / 枚举 / 自定义类型 | `core-01c-index-enum-custom-type.md` |

## 10. 对齐参考源

- drawdb `src/components/EditorCanvas/`（`Canvas.jsx` / `Table.jsx` / `Relationship.jsx` / `Area.jsx` / `Note.jsx`）
- drawdb `src/hooks/{useCanvas,useTransform,useSelect}.js`
- coldrawdb `frontend-rs/src/editor_render.rs`
- `docs/drawdb-capability-checklist.md` §1.1 / §2.3
