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

- **render 层**：`frontend-rs/editor_render` 使用 `<canvas>`（HTML5）+ 贝塞尔连线（自渲染，无 vDOM diff）
- **响应式**：基于 Leptos signals 细粒度更新（仅重绘变更部分）
- **性能预算**：100 张表 / 200 条关系 / 60fps（来源：Phase 4 W4 perf）

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
