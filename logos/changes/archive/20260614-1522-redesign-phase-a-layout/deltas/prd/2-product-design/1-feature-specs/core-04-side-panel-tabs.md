# Delta — core-04-side-panel-tabs.md
# 模块：core | 提案：redesign-phase-a-layout
# 路径：`logos/changes/redesign-phase-a-layout/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`

## MODIFIED — §1 侧边栏布局

> **替换** 主文档 `core-04-side-panel-tabs.md` §1 整节。

## 1. 侧边栏布局（V2 / Phase A）

V2 取消 V1 的「左栏 280px + 7 Tab 列表」布局。浏览职能拆分为：

| 原 Tab 职能 | V2 承接方式 | Phase |
|-------------|-------------|-------|
| Tables 列表浏览 | Command Palette（`Ctrl+K`） | Phase D |
| Tables「+ 建表」 | Tool Rail `⊕ 表` | **Phase A** |
| Areas / Notes 创建 | Tool Rail `⊕ 区域` / `⊕ 便签` | **Phase A** |
| Relationships 列表 | Inspector `InspectorReference` 摘要 + Command Palette | Phase D |
| Enums / Types 管理 | AppBar `⚙ 设置` 子菜单（保留模态入口） | Phase A 入口不变 |
| Issues 列表 | Tool Rail 徽章 → Inspector `InspectorIssues` | **Phase A** |
| 全局搜索 | Command Palette | Phase D |

### 1.1 Tool Rail 布局（替代左栏）

```
┌────┐
│ ⊕  │  ← 新建菜单（表 / 区域 / 便签）
│ 🔗 │  ← 关系工具（Phase B 接入逻辑；Phase A 显示 tooltip）
│ ✋ │  ← 平移工具（Toggle；亦可用空格）
│────│
│ ⚠3 │  ← Issues 徽章（数字 = 错误+警告）
└────┘
```

- 根节点：`data-testid="tool-rail"`
- 宽度：**48px**，图标按钮 `40×40px`，垂直排列
- 新建菜单：点击 `⊕` 展开弹出菜单（非模态），选项：
  - `新建表` → `data-testid="tool-new-table"`
  - `新建区域` → `tool-new-area`
  - `新建便签` → `tool-new-note`

### 1.2 Phase A 移除的 UI（数据层保留）

以下 Tab 的**列表 UI** 在 Phase A 不渲染，但 `EditorStore` 中 `tables / areas / notes / references / enums / types` signals **保持不变**：

- Tables Tab 列表（`tab-tables` / `tab-pane-tables`）
- Areas / Enums / Notes / Relationships / Types Tab
- 顶部搜索框（`search-input`）与类型筛选（`type-filter`）

> **迁移说明**：Phase D 将通过 Command Palette 恢复浏览能力。Phase A 代码批次删除 `LeftPanel` 组件，保留 `jump_to_table` / `jump_to_reference` 纯函数供 Issues 定位使用。

### 1.3 默认工具状态

- 进入编辑器：当前工具 = **选择**（`V`）
- Tool Rail 选择工具图标：`data-testid="tool-select"`（Phase A 无独立按钮，点击画布空白回到选择态）

## MODIFIED — §2 Tables Tab

> **替换** §2 首段说明；§2.2 操作表中与左栏列表相关的行标注为 Phase D。

### 2.0 Phase A 变更摘要

Tables Tab 列表 UI 移除。以下操作迁移：

| 原操作（§2.2） | V2 触发 | Phase |
|----------------|---------|-------|
| 单击项 → 画布高亮 | Command Palette 选择表 | D |
| 双击项 → 重命名 | 画布双击表名 / Inspector 表名输入 | **A** |
| 右键项 → 上下文菜单 | 画布表头右键 | **A** |
| 「+」→ 创建新表 | Tool Rail `⊕` → 新建表 | **A** |
| 拖拽项到画布 | 移除（低使用率） | — |

§2.1 列表项、§2.3 搜索/筛选在 Phase A **不适用**（整节保留原文档供 Phase D 恢复时参考，标题加注「Phase D 恢复」）。

## MODIFIED — §8 Issues Tab

> **替换** §8.1 用途段落 + §8.3 操作表。

### 8.1 用途（V2）

Issues 校验逻辑与 V1 相同，**展示位置**从左栏 Tab 迁移至：

1. Tool Rail 底部徽章（数字摘要）
2. Inspector `InspectorIssues` 子面板（完整列表）

### 8.3 操作（V2）

| 操作 | 触发 | 效果 |
|------|------|------|
| 查看列表 | 点击 Tool Rail `⚠` 徽章 | Inspector 展开并显示 `InspectorIssues` |
| 单击项 | Inspector 列表项 | 跳转到对象 + 画布闪烁 |
| 过滤下拉 | Inspector Issues 顶部 | 按级别筛选 |
| 手动校验 | AppBar `⚙` → Validate | 强制重新校验（与 V1 一致） |

## MODIFIED — §11 测试用例 ID 索引

> **追加** 到 §11 表格末尾。

| TC ID | 描述 | Phase A 状态 |
|-------|------|--------------|
| UT-SP-09 | 6 业务 Tab 切换 | ⏸️ Phase A 移除左栏 Tab，用例搁置至 Phase D |
| UT-SP-10 | 全局搜索跨 Tab 过滤 | ⏸️ 同上 |
| **UT-TR-01** | Tool Rail 渲染 + `data-testid="tool-rail"` | ✅ Phase A |
| **UT-TR-02** | `tool-new-table` 点击 → 创建表 | ✅ Phase A |
| **UT-TR-03** | Issues 徽章数字与校验结果一致 | ✅ Phase A |
| **UT-TR-04** | Issues 徽章点击 → Inspector Issues 视图 | ✅ Phase A |

## ADDED — §14 Phase A 边界补充

- ❌ 左栏 7 Tab 列表 UI（Phase D Command Palette 恢复浏览）
- ❌ 拖拽表项到画布
- ✅ Tool Rail 新建表/区域/便签
- ✅ Issues 徽章 + Inspector 子视图
- ✅ 画布 / Inspector 上的表与字段编辑（替代列表导航）
