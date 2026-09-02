# 产品优化批次 · 现状摸底（条目6 切片 1/3）

> 草稿，**未创建** `logos/changes/` 目录，未运行 `openlogos change`。
> 路径约定：草稿落 `.octos/proposals/draft-2026-09-02-product-batch/` 等待外环 review + operator 确认。

## 0. 项目基线（来自 `logos/logos-project.yaml`）

- 前端：Rust + Leptos 0.x + trunk + HTML5 Canvas（自绘）
- 后端：Rust + actix-web 4 + SeaORM
- 数据库：**仅 SQLite（WAL）**
- 场景：S01 编辑保存 ✅、S02 分享 ✅、S03 注册登录 in-progress、 S04 房间 in-progress、S05 OT in-progress、S06 MCP launched
- 测试：cargo test + Playwright（spec-parity a/b/c/d）+ 单文件原型

## 1. 六项需求 → 现状映射

### 需求 1：表结构列表视图（参考 pdmaner）

**现状**：
- 编辑器只有**画布视图**（HTML5 Canvas 自绘）
- `editor_panels.rs` 内的 `TableListSidebar`/`TableDrawer` 等都不存在（grep 无命中）
- 数据模型 `table`/`field` 已在 store，可被派生视图消费
- Inspector 是单表焦点视图（field 列表 + reference 列表），但**没有跨表全量列表**

**pdmaner 列表视图典型特性**（待 operator 确认边界）：
- 表名/字段名/类型/索引/备注 表格化
- 排序、过滤、批量重命名
- 双击跳到画布对应表

**触及模块**：`frontend-rs/src/editor_panels.rs`（新增 TableListSidebar/TableDrawer 组件）、`editor_core.rs`（store 派生 selectors）、Inspector 状态机。

**UI/UX 影响**：是（新增主视图入口，与 Inspector/Canvas 三向切换）。

### 需求 2：字段关系连接逻辑优化

**现状**：
- `editor_panels.rs:411` `&["one_to_one", "one_to_many", "many_to_one", "many_to_many"]` ——**4 选 1 必选 cardinality**
- `editor_panels.rs:3743` `rel-confirm-cardinality` —— 创建关系时弹下拉让用户选
- 用户原始诉求："连接时不要求选择 1:1/1:N，连接多个字段自然推导为 1:N 或 N:N"

**触及模块**：`editor_panels.rs` 的 relation 创建/编辑流、cardinality 推导规则、Inspector reference 面板。

**关键决策**：
- cardinality **派生还是可改**？派生后是否允许用户覆盖？
- 多字段连接是"两边任意字段数"还是"一边多字段一边单字段"？
- DB 落库：`reference.end_field_id` 单字段还是数组？

**UI/UX 影响**：是（去除确认条的下拉，关系创建流变化）。

### 需求 3：开始支持 PostgreSQL / MySQL

**现状**：
- 后端**硬编码 SQLite**：SeaORM + SQLite WAL（`backend/Cargo.toml` 推断）
- `diagrams_v1.rs` 测试 `database: "mysql"` 是字面量字段，**无实际 MySQL 连接**
- `editor_core.rs:109` 注释：dialect 字段已存在但**无运行时分支**
- 导入/导出：SQL/DBML/JSON 三种（**SQL 是通用 dialect**），DBML 是 ER 图通用
- 导出后给用户"在 PG/MySQL 执行"的语义**已具备**；**在线连接到 PG/MySQL 数据库执行 DDL/introspect 是新能力**

**触及模块**（影响面**最大**）：
- 后端：datasource 抽象层、新增 `sqlx` 或 `sea-orm` 多 dialect 配置、连接池管理
- 后端：新增 `/api/v1/datasources` CRUD、`/api/v1/introspect/{datasource_id}` 在线反射
- 前端：datasource 管理 UI、连接配置弹窗、introspect 流程
- MCP S06：新增 `mcp__datasource__*` 工具族
- DB 存储：`user`/`datasource`/`datasource_secret` 新表 + 加密 secret
- 测试：需 docker-compose 起 PG/MySQL 容器跑集成测试

**部署影响**：是（新增外部依赖：用户 PG/MySQL 实例可达）。

**UI/UX 影响**：是（datasource 管理页 + introspect 流程页）。

**工作量预估**：1 个 sprint 量级（≥ 1 周），影响面**远超**其他五项。

### 需求 4：画布自由度——调整表的宽高

**现状**：
- `editor_panels.rs:7473` `parse_table_width(input: &str)` **已存在**——输入解析逻辑有
- `editor_panels.rs:8136` UI 已暴露宽度调整入口
- `tests/tokens.rs` + UT-MM-11 测覆盖
- **没有高度调整**（grep `parse_table_height` 无命中）
- 表的渲染宽高在 Canvas draw 时是固定默认（推测 200×auto），Inspector 可手动改 width

**触及模块**：`editor_panels.rs`（加 parse_table_height）、Canvas 渲染（动态高度计算）、表头/字段行换行布局、reference 连线重算。

**UI/UX 影响**：是（Inspector 加 height 字段；可能需要拖拽 resize 手柄）。

**工作量**：小（与 UT-MM-11 同思路，约 0.5-1 天）。

### 需求 5：样式优化（字体清晰度、交互流畅性）

**现状**：
- 前端 `frontend-rs/src/styles.css` 是全局样式
- 字体来源：`<link>` 引外部或 fallback 到 `system-ui`（推测）
- Canvas 渲染字体走 `ctx.font`（推测，需 grep 确认）
- 交互流畅性：S05 OT 协作 + 自动保存 + 防抖均已有基础设施

**典型范畴**（待 operator 确认边界）：
- 全局字体回退栈 + 字号梯度（标题/正文/小字）
- 字体子像素抗锯齿 (`-webkit-font-smoothing`)
- Canvas 文本渲染走离屏 canvas 缓存
- 关键交互（拖拽/连线/Inspector 切换）< 16ms/帧
- reduced-motion 支持（已有 UT-MM 覆盖）

**触及模块**：`styles.css`（全局变量）、`editor_panels.rs` 的 canvas draw 路径、字体加载策略。

**UI/UX 影响**：是（视觉全局改动）。

**工作量**：中（需设计与回归全套 UI 视觉）。

### 需求 6：提高用户方便性（泛化项）

**现状**：operator 没具象化，需内环提出子项供 operator 圈定。

**常见可改进点**（待 operator 圈定优先级）：
- 快捷键可发现性（⌘K 命令面板已有 ST-KB-CMD-01，可补"?" 弹帮助）
- 错误提示本地化（中英双语 + 错误码映射）
- 撤销栈视觉化（History 面板，列出最近 N 步）
- 表/字段批量操作（批量重命名、批量改类型）
- 自动保存可关闭（高级用户）
- 移动端体验（720px 以下降级已有 ST-PU-26，再做 480px）
- 教程/空状态引导（首次进入引导、模板库）

**触及模块**：取决于具体子项。

**UI/UX 影响**：取决于子项。

## 2. 落点汇总表

| 需求 | 前端模块 | 后端模块 | DB schema | MCP S06 | 部署 | UI/UX | 工作量 |
|---|---|---|---|---|---|---|---|
| 1 列表视图 | ✅ 新组件 | — | — | — | — | ✅ | 中（3-5 天）|
| 2 关系推导 | ✅ relation 流 | — | reference 表 | — | — | ✅ | 小-中（1-3 天）|
| 3 PG/MySQL | ✅ datasource UI | ✅ 新抽象 + 路由 | ✅ 新表 | ✅ 新工具 | ✅ | ✅ | **大（≥ 1 sprint）** |
| 4 表宽高 | ✅ parse + canvas | — | — | — | — | ✅ | 小（0.5-1 天） |
| 5 样式 | ✅ styles.css + canvas | — | — | — | — | ✅ | 中（3-5 天） |
| 6 方便性 | 取决于子项 | — | — | — | — | ✅ | — |

## 3. 关键风险点

- **需求 3 是变更之王**：影响 schema + 后端 + 前端 + MCP + 测试基础设施。任何"6 项一锅端"都不可行。
- **需求 1/4/5 同属前端画布与呈现层**：可合并为 `ux-canvas-batch` 一个提案，但要求 1 与 4/5 复杂度差 5 倍——合并会让提案过大。
- **需求 6 是黑洞**：operator 不具象化 = 无法定边界 = 无法定工作量。
- guard 是全局单活跃：6 项**任何时刻只能开 1 个**，并行必须串行。