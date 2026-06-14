# Delta — core-06-inspector-panel.md（新文件）
# 模块：core | 提案：redesign-phase-a-layout
# 路径：`logos/changes/redesign-phase-a-layout/deltas/prd/2-product-design/1-feature-specs/core-06-inspector-panel.md`
# merge 目标：`logos/resources/prd/2-product-design/1-feature-specs/core-06-inspector-panel.md`（ADDED 新文件）

## ADDED — Inspector 抽屉规格（Phase A）

> 模块：core | 提案：redesign-phase-a-layout
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-06-inspector-panel.md`
> 对齐参考源：UX 重规划 Phase A + 原 `core-04` 右栏 + `core-01a` 字段编辑

# Inspector 抽屉规格（V1 / Phase A）

## 1. 概述

Inspector 是 V2 布局中**唯一的属性编辑面板**，替代 V1 固定右栏 `RightPanel`。根据 `SelectionState` 动态切换内容；支持折叠以最大化画布面积。

## 2. 布局

```
+─ Inspector ──────────────────────── [×] ─┐
│ ┌─ 标题栏 ─────────────────────────────┐ │
│ │ 🔍 字段：user_id          [折叠]    │ │
│ └─────────────────────────────────────┘ │
│ ┌─ 内容区（可滚动）───────────────────┐ │
│ │  （按选中类型渲染子面板）            │ │
│ └─────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

- 根节点：`data-testid="inspector-panel"`
- 默认宽度：**320px**
- 折叠：宽度动画至 0（V1 无 transition，瞬时切换）；StatusBar `btn-inspector-toggle` 同步状态
- 标题栏右侧 `×`：等价折叠，不清除 SelectionState

## 3. 子面板清单

| 子面板 ID | 触发条件 | 内容 | 对齐规格 |
|-----------|----------|------|----------|
| `InspectorNone` | `kind=none` 且展开 | 项目概览：表数、关系数、引擎、revision | `core-02` |
| `InspectorTable` | `kind=table` | 表名、注释、颜色、字段列表、索引区 | `core-01a` §1 |
| `InspectorField` | `kind=field` | 字段全属性表单 | `core-01a` §2 |
| `InspectorReference` | `kind=reference` | cardinality、onUpdate、onDelete、翻转 | `core-01b` §3 |
| `InspectorArea` | `kind=area` | 区域名、颜色 | `core-04` §3 |
| `InspectorNote` | `kind=note` | 便签内容、颜色 | `core-04` §5 |
| `InspectorIssues` | Tool Rail Issues 徽章点击 | 校验错误列表 + 定位按钮 | `core-04` §8 |

## 4. InspectorField 详细结构

```
┌─ 字段：{name} ──────────────────┐
│ 名称    [____________]          │
│ 类型    [INT ▼]  长度 [____]    │
│ ── 约束 ──                      │
│ ☑ 主键  ☐ 唯一  ☑ 非空  ☐ 自增  │
│ 默认值  [____________]          │
│ 注释    [____________]          │
│ ── 关系 ──                      │
│ → {ref_display}        [跳转]   │
│ [+ 创建外键连接]（Phase B 启用） │
└─────────────────────────────────┘
```

**testid**：

- `inspector-field-name`
- `inspector-field-type`
- `inspector-field-pk`（主键 checkbox）
- `btn-add-field`（表级：在 `InspectorTable` 底部）

字段修改触发 `store` 更新 + debounce 自动保存（与 V1 右栏行为一致）。

## 5. InspectorTable 详细结构

- 表名输入：`inspector-table-name`（双击画布表名亦可行内编辑，两处双向绑定）
- 字段列表：每行显示 `name` + `type` + PK 图标；单击行 → `SelectionState = field`
- 底部 `[+ 添加字段]`：`btn-add-field`
- 索引区（折叠）：`[+ 添加索引]`（对齐 `core-01c` §1，Phase A 保留 UI 壳）

## 6. InspectorIssues 详细结构

替代 V1 左栏 Issues Tab 的**展示职能**（校验逻辑不变）。

每项：
- 级别图标（error / warning / info）
- 消息文本
- `[定位]` 按钮 → 调用 `jump_to_*` + 画布闪烁

顶部：级别筛选下拉（all / error / warning）

## 7. 折叠与展开行为

| 状态 | Inspector | SelectionState |
|------|-----------|----------------|
| 默认进入编辑器 | 折叠 | `none` |
| 选中任意对象 | **自动展开** | 对应 kind |
| 用户点击折叠 | 折叠 | 保持不变 |
| 双击画布空白 | 折叠 | `none` |
| 新建第一张表（引导卡片） | **自动展开** | `table` |

## 8. 与 V1 RightPanel 的差异

| 维度 | V1 RightPanel | V2 Inspector |
|------|---------------|--------------|
| 宽度 | 固定 300px | 默认 320px，可折叠 |
| 无选中时 | 显示「Select a field」 | 折叠；展开时显示项目概览 |
| 选中表 | 不完整（V1 偏字段） | 完整表属性 + 字段列表 |
| Issues | 独立左栏 Tab | Inspector 子视图 |
| Tab | Fields 单 Tab | 无 Tab，按选中类型切换 |

## 9. 测试用例 ID 索引

| TC ID | 描述 |
|-------|------|
| UT-IN-01 | 选中表 → Inspector 自动展开 + `InspectorTable` 渲染 |
| UT-IN-02 | 选中字段 → `InspectorField` 含 `inspector-field-name` |
| UT-IN-03 | 点击 StatusBar 折叠 → `cdb-is-inspector-collapsed` |
| UT-IN-04 | Issues 徽章点击 → `InspectorIssues` 列表渲染 |
| UT-IN-05 | 字段列表行点击 → SelectionState 切为 field |
| ST-IN-01 | e2e：建表 → 加字段 → Inspector 表单编辑 → 保存 |

> 详细步骤见 `core-PA-layout-test-cases.md` §3 Inspector 组。

## 10. Phase A 边界

- ❌ Inspector 宽度拖拽（V2）
- ❌ 「+ 创建外键连接」按钮功能（Phase B）
- ❌ 导入预览在 Inspector 显示（Phase C 用侧边抽屉）
- ❌ Command Palette 在 Inspector 内嵌（Phase D）

## 11. 对齐参考源

- UX 重规划 Phase A（2026-06-14）
- 原 `core-04-side-panel-tabs.md` §8 Issues
- 原 `core-01a-table-and-field.md`
- coldrawdb `frontend-rs/src/editor_panels.rs::RightPanel`（迁移基线）
