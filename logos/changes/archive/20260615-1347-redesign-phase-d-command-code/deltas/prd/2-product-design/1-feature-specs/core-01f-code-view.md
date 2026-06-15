# Delta — core-01f-code-view.md（新文件）

> merge 时作为新文件写入 `logos/resources/prd/2-product-design/1-feature-specs/core-01f-code-view.md`

## ADDED — 全文

> 模块：core | 提案：redesign-phase-d-command-code
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-01f-code-view.md`
> 对齐：drawdb `DBMLEditor.jsx`、`core-01d-import-export.md` 导出函数
> 最后更新：2026-06-14

# SQL/DBML 全屏代码视图规格（V2 / Phase D）

## 1. 概述

Phase D 实现 AppBar **代码**按钮（`btn-code-view`），在 Canvas 视图与全屏代码视图之间切换。

- V1：**只读预览** + 复制（复用 Phase C `export_diagram_sql` / `export_diagram_dbml`）
- 编辑后应用回画布：不在 Phase D 范围

## 2. 状态模型

```rust
enum EditorViewMode {
    Canvas,
    Code,
}

enum CodeFormat {
    Sql,
    Dbml,
}

// AppRoot
view_mode: RwSignal<EditorViewMode>
code_format: RwSignal<CodeFormat>
code_engine: RwSignal<String>  // SQL Tab 引擎，默认 generic
```

## 3. 布局（Code 模式）

```
+--------------------------------------------------------------------------------+
| AppBar：... [代码●] [分享] ...                                                  |
+--------------------------------------------------------------------------------+
| [ SQL ] [ DBML ]     引擎: [ generic ▼ ]          [复制] [返回画布]              |
+--------------------------------------------------------------------------------+
|                                                                                |
|  CREATE TABLE users (                                                          |
|    id INT PRIMARY KEY,                                                         |
|    ...                                                                         |
|  );                                                                            |
|                                                                                |  ← code-view-textarea (readonly)
|                                                                                |
+--------------------------------------------------------------------------------+
| StatusBar（保留：缩放隐藏或显示「代码视图」标识）                                  |
+--------------------------------------------------------------------------------+
```

- 隐藏：Tool Rail、Canvas、Inspector、IO 抽屉
- testid：`code-view` / `code-view-tabs` / `code-format-sql` / `code-format-dbml` / `code-engine-select` / `code-view-textarea` / `code-copy-btn` / `code-back-btn`

## 4. 入口

| 入口 | testid | 行为 |
|------|--------|------|
| AppBar 按钮 | `btn-code-view` | 切换 `Canvas ↔ Code` |
| View 菜单 | `cdb-menu-code-view` | 同按钮 |
| Esc（Code 模式） | — | 返回 Canvas |

`btn-code-view` Phase D **启用**（移除 Phase A disabled 占位）。

## 5. 预览生成

| Tab | 函数 | 刷新时机 |
|-----|------|----------|
| SQL | `export_diagram_sql(store, engine)` | 进入 Code 视图、切换引擎、store 变更（V1 进入时快照即可） |
| DBML | `export_diagram_dbml(store)` | 同上 |

## 6. 复制 / 下载

- **复制**：`navigator.clipboard.writeText` 或 `document.execCommand('copy')` fallback
- **下载**（可选 V1）：`code-download-btn` 触发 `.sql` / `.dbml` 文件下载；未实现则仅复制

## 7. 与 IO 抽屉关系

| 场景 | 行为 |
|------|------|
| Export 抽屉 | 侧边 400px 预览，可并存 Inspector 互斥 |
| Code 视图 | 全屏只读，替代画布区域 |
| 导入 | 仍走 ImportDrawer（File / AppBar / EmptyGuide），不在 Code 视图内嵌 |

## 8. 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-PD-04 | Code 视图 SQL 输出含 `CREATE TABLE` |
| UT-PD-05 | `view_mode=Code` 时 `.cdb-main` 含 `cdb-is-code-view` |
| UT-PD-06 | `btn-code-view` 非 disabled |

## 9. V1 边界

- ❌ 可编辑 textarea + Apply 同步画布
- ❌ JSON Tab（Export 抽屉已有）
- ❌ 语法高亮（Monaco）
- ❌ 与 bridge export API 对接
