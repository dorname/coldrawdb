# 导入 / 导出 IO 抽屉规格（V2 / Phase C）

## 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。IO 入口与格式以主原型「更多菜单 → 导入/导出抽屉」为准。

| 项 | 约定 |
|---|---|
| 页面流 | IO 仅在 `room-editor`（及 EmptyGuide 引导）触发；不依赖独立历史原型 |
| 演示 ≠ 生产 | 主原型导入为本地解析模拟；剪贴板/下载以 Toast 或本地 blob 演示。生产导入走 bridge API，导出客户端生成 |
| 实现状态 | bridge / 导出能力**后端或客户端已具备**；**生产前端部分接入**；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity` |

## 1. 概述

Phase C 将 V1 居中 **Import 模态** 与占位 **Export 按钮** 升级为画布右侧 **非模态 IO 抽屉**：

- 不占用 `ModalRoot` 遮罩层（L4），与 Inspector（L3）同级侧栏语义
- 导入走 `POST /api/v1/bridge/import/local`
- 导出在**客户端**从当前 `EditorStore` 生成 SQL / DBML / JSON 预览（V1 无服务端 export 端点）

## 2. 组件树

```
AppRoot
├── AppBar
│   └── 更多菜单 (btn-more-menu)
│         ├── btn-import  → ImportDrawer
│         └── btn-export  → ExportDrawer
├── ToolRail / Command Palette（可命令打开同一抽屉）
├── IoDrawer / SideSheet
│   ├── ImportDrawer  data-testid="import-drawer"
│   └── ExportDrawer  data-testid="export-drawer"
└── EmptyGuide → guide-import-sql → ImportDrawer
```

**主路径变更**：导入/导出**不再**以 AppBar 常驻 pill 为默认；统一经 **更多菜单**（与主原型 `renderMoreMenu` 一致）。`btn-import` / `btn-export` testid 保留在菜单项上。

## 3. 状态模型

```rust
enum IoDrawerKind {
    None,
    Import,
    Export,
}
```

| 信号 | 类型 | 说明 |
|------|------|------|
| `io_drawer` | `RwSignal<IoDrawerKind>` | 当前打开的抽屉 |
| `inspector_open_before_io` | 可选缓存 | 打开 IO 抽屉前 Inspector 是否展开，关闭时恢复 |

### 3.1 互斥规则

- 打开 Import / Export 抽屉时折叠 Inspector（`data-testid="inspector"`），关闭时恢复。
- 与成员抽屉 / 活动抽屉互斥（同一 `drawer` 槽位，主原型一次只开一个）。
- Code View 打开时不叠加 IO 抽屉为默认路径。

## 4. ImportDrawer

### 4.1 布局

```
┌─ 导入 ───────────────────────────── [×] ─┐
│ [ SQL ] [ DBML ] [ JSON ]                 │  ← format tabs
│ 数据库引擎: [ generic ▼ ]  （SQL 时显示）  │
│ ┌─────────────────────────────────────┐ │
│ │ 粘贴 SQL / 拖放 .sql 文件            │ │  ← import-textarea
│ └─────────────────────────────────────┘ │
│ 解析摘要: 3 条语句 · 预计 2 张表          │  ← import-parse-summary
│ [ 取消 ]              [ 导入并打开 ▶ ]   │
└──────────────────────────────────────────┘
```

- 宽度：**400px**（与 Inspector 默认 320px 区分，可挤压画布）
- testid：`import-drawer` / `import-format-tabs` / `import-engine-select` / `import-textarea` / `import-parse-summary` / `import-submit` / `import-cancel`

### 4.2 格式 Tab

| format | 引擎选择 | 解析预览 |
|--------|----------|----------|
| `sql` | 必填（默认 `generic`） | `parse_sql_statements` 语句数 + 简单表名启发式（可选） |
| `dbml` | 隐藏 | 行数 / `Table` 块计数（纯函数 `count_dbml_tables`） |
| `json` | 隐藏 | `serde_json` 校验 + tables 数组长度 |

### 4.3 文件拖放

- 接受扩展名：`.sql` / `.dbml` / `.json`
- 拖入后：自动切换对应 format Tab，内容写入 textarea
- 大小上限：读取 `bridge/config` 的 `maxImportSizeKb`（V1 可硬编码 5120 KB，与 bridge 默认一致）

### 4.4 提交行为

1. 校验：内容非空；SQL 需选引擎；JSON 需合法
2. `POST /api/v1/bridge/import/local` body：`{ format, content, engine?, title? }`
3. 成功（`status: success` 或返回 `diagramId`）→ `window.location` 跳转 `/editor/{diagramId}`
4. 失败 → 抽屉内 inline 错误 + ErrorToast
5. 按钮文案：**导入并打开**；提交中 disabled + `导入中...`

### 4.5 入口汇总

| 入口 | testid | 行为 |
|------|--------|------|
| AppBar | `btn-import` | 打开 ImportDrawer |
| File 菜单 | `cdb-menu-import` | 打开 ImportDrawer（不再开 Import 模态） |
| EmptyGuide | `guide-import-sql` | 打开 ImportDrawer |

## 5. ExportDrawer

### 5.1 布局

```
┌─ 导出 ───────────────────────────── [×] ─┐
│ [ SQL ] [ DBML ] [ JSON ]                 │
│ 数据库引擎: [ mysql ▼ ]  （SQL 时显示）    │
│ ┌─────────────────────────────────────┐ │
│ │ CREATE TABLE users ( ... );         │ │  ← export-preview（只读）
│ │ ...                                 │ │
│ └─────────────────────────────────────┘ │
│ [ 复制 ]  [ 下载 .sql ]                   │
└──────────────────────────────────────────┘
```

- testid：`export-drawer` / `export-format-tabs` / `export-engine-select` / `export-preview` / `export-copy` / `export-download`

### 5.2 预览生成（客户端）

| format | 函数 | V1 范围 |
|--------|------|---------|
| SQL | `export_diagram_sql(store, engine)` | 最小可用：`CREATE TABLE` + 列定义 + PK；FK 引用 `references` |
| DBML | `export_diagram_dbml(store)` | 表 + 字段 + `ref:` 关系 |
| JSON | `serde_json::to_string_pretty(diagram)` | 与 persistence JSON 同构 |

空 diagram：预览区显示「暂无表，无法导出」；复制/下载 disabled。

### 5.3 复制 / 下载

- **复制**：`navigator.clipboard.writeText`；成功按钮文案 `已复制` 2s
- **下载**：触发 `<a download>` blob；文件名 `{diagram_title}.{sql|dbml|json}`

## 6. 与 V1 模态的关系

| V1 组件 | Phase C 处理 |
|---------|--------------|
| `ImportModal` | 保留代码供 UT-MM-10；File 菜单与 AppBar **不再**打开；可标记 `#[deprecated]` 注释 |
| `ImportSourceModal` | 不变；ImportDrawer 内嵌 `local` 源（不单独弹源选择） |
| Export 模态（未实现） | 由 ExportDrawer 完全替代 |

## 7. 样式

```css
.cdb-io-drawer {
  width: 400px;
  border-left: 1px solid var(--cdb-color-border);
  background: var(--cdb-color-bg);
  display: flex;
  flex-direction: column;
  z-index: 30; /* L3，与 Inspector 同级 */
}
.cdb-main.cdb-has-io-drawer {
  grid-template-columns: 48px 1fr 0 auto; /* 折叠 Inspector，显示 IO 抽屉 */
}
```

## 8. 测试 ID 索引

| TC ID | 描述 |
|-------|------|
| UT-PC-01 | `parse_sql_statements` 驱动导入摘要 |
| UT-PC-02 | `export_diagram_sql` 非空 diagram 输出含 `CREATE TABLE` |
| UT-PC-03 | `export_diagram_dbml` 含 `Table` 块 |
| UT-PC-04 | `IoDrawerKind` 互斥：开 Import 折叠 Inspector |
| UT-PC-05 | `count_dbml_tables` 纯函数 |
| ST-PC-01 | e2e：btn-import → 粘贴 SQL → 解析摘要可见 |

详细步骤见 `core-PC-import-export-test-cases.md`。

## 导入 / 导出格式统一约束

导入与导出均支持 **SQL / DBML / JSON** 三分段（`import-format-tabs` / `export-format-tabs`）：

| format | 导入 | 导出 |
|--------|------|------|
| SQL | 粘贴/拖入；引擎选择（生产）；解析摘要 | 客户端 SQL 预览 + 复制/下载 |
| DBML | 粘贴/拖入；表块计数 | DBML 预览 |
| JSON | 合法 JSON + tables | persistence 同构 JSON |

抽屉标题副文案对齐主原型：「SQL · DBML · JSON」。

## 9. Phase C 边界

- ❌ SQL/DBML 全屏代码视图（`btn-code-view`）— Phase D
- ❌ Mermaid / PNG 导出 — 后续提案
- ❌ 导入任务异步轮询 UI（logs/retry）— V1 仅同步成功路径 + 错误展示
- ✅ 右侧 IO 抽屉（Import + Export）
- ✅ bridge import API 接线
- ✅ 客户端 SQL/DBML/JSON 导出预览

---

### 9.1 统一原型边界补充

- ❌ 将主原型「模拟导入」标为已完成生产 bridge 接线
- ❌ 以 V1 Import 模态为默认用户路径（模态仅保留回归 UT）
- ✅ 更多菜单 → 导入/导出抽屉；SQL/DBML/JSON
- ✅ Viewer：导入提交与改写 diagram 的导出下载策略按房间角色；只读角色不得提交导入写库（与 `canEdit` 对齐，具体权限见 S04）

# Delta — core-01d-import-export.md（修改）

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3 增量）

## 2 组件树（E3 SideSheet 升级）

**merge 时在 §2 末尾追加**：

### §2.x E3 SideSheet 重构

V1 IO 抽屉用内嵌 `<aside class="cdb-io-drawer">` 自实现。E3 升级为 `<SideSheet placement=Right width=400>` 组件（来自 `core-09-core-components.md` §9）：

```rust
<SideSheet
    visible=io_drawer_open
    title=move || format!("{kind:?}")
    placement=SideSheetPlacement::Right
    width=400
    mask={true}
    mask_closable={true}
>
    <ImportExportContent kind=kind />
</SideSheet>
```

**Props 差异**（V1 → E3）：
- `cdb-is-io-drawer-open` class → `visible: RwSignal<bool>` prop
- `cdb-io-drawer__close` 内嵌按钮 → `<Button variant=Tertiary icon=IconClose />` in header
- 关闭动画：手动 CSS → E3 内置 `slide-in-right` / `slide-out-right`（E6 接入）

## 4–5 ImportDrawer / ExportDrawer（E2 复制/下载图标）

**merge 时在 §4、§5 各追加**：

### §4.x / §5.x 抽屉内操作按钮（E2 + E3）

| 行为 | 组件 | 视觉 |
|---|---|---|
| 复制导入源 / 复制导出结果 | `<Button variant=Secondary icon=IconCopy>复制</Button>` | E3 Button Secondary |
| 下载导出文件 | `<Button variant=Primary icon=IconDownload>下载</Button>` | E3 Button Primary |
| 拖入文件 | `<div class="cdb-dropzone"><IconUpload />"拖入文件或点击选择"</div>` | E2 Icon + E3 Collapse-style border |
| 切换数据库（MySQL/PostgreSQL/SQLite/...） | `<Dropdown trigger=Click position=BottomLeft>` | E3 Dropdown |

**ImportDrawer** 头部增加 `<Tag color=Info size=Small>SQL/DBML/JSON</Tag>` 标识当前 format。
