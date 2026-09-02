# 产品优化批次 · 提案切分与推荐顺序（条目6 切片 2/3）

> 草稿，**未创建** `logos/changes/` 目录，未运行 `openlogos change`。
> 与 `01-current-state.md` 配套；提案草案与 `openlogos change` 留待外环下一条 steer。

## 1. 切分原则

1. **guard 是全局单活跃**（来自 openlogos 工具设计）—— 6 项任何时刻只能 1 个提案 in-flight
2. **影响面相近的合并**：纯前端呈现层（1/4/5）的代码同区域（`editor_panels.rs` + `styles.css` + `editor_core.rs`），可合并以减少 guard 切换
3. **跨后端/DB/MCP 的独立**：3 影响 schema+后端+前端+MCP，必须独立成案
4. **依赖序**：后端基础（3）→ 前端 UX 层（1/4/5）→ 关系推导（2）→ 方便性（6 拆子项）
5. **可独立交付的最小切片优先**：4（表宽高）最小、可独立、可作"热身"提案先做

## 2. 推荐切分（5 个提案 + 1 个待拆分）

### 提案 A: `feat-table-resize`（需求 4 独立）
- **范围**：表宽 + 表高（高度是新增，宽度的 parse 函数已存在 UT-MM-11）
- **涉及文件**：`editor_panels.rs`（加 `parse_table_height`、Inspector height 字段、Canvas 动态高度 + 多行字段布局）、`tests/tokens.rs`（新增 UT-MM-12 高度解析）
- **数据契约变更**：表 record 加可选 `height: Option<u32>` 字段（向后兼容）
- **工作量**：小，0.5-1 天
- **验收门槛**：UT-MM-11/12 pass + Canvas 视觉回归 + reference 连线不重叠
- **可独立交付**：是
- **风险**：与 reference 连线布局耦合，需小心端点重算

### 提案 B: `feat-multiple-datasources`（需求 3 独立，最大）
- **范围**：PG/MySQL datasource 抽象 + 在线 introspect
- **涉及文件**：
  - 后端：`datasources_v1.rs`（新模块）、`introspect_v1.rs`（新模块）、`diagrams_v1.rs`（dialect 字段运行时分支）
  - DB：`datasource` / `datasource_secret` 新表 + 加密
  - 前端：datasource 管理页、连接配置弹窗、introspect 进度
  - MCP：`mcp__datasource__*` 工具族（≥4 个新工具）
  - 测试基础设施：`docker-compose.test.yml`（pg + mysql 容器）+ CI 接入
- **数据契约变更**：`diagram.database: String → DatasourceRef { id, kind: Pg|Mysql|Sqlite }`
- **工作量**：**≥ 1 sprint**（5-10 天）
- **验收门槛**：docker-compose 起 PG/MySQL 容器 → 在线 introspect 真实 PG/MySQL → 导出 SQL 在 PG/MySQL CLI 可执行
- **可独立交付**：是，但需要先有"导出到 SQL"基础（已具备）
- **风险**：schema migration、secret 加密、连接池生命周期、PG/MySQL dialect 差异

### 提案 C: `ux-canvas-batch`（需求 1+4+5 合并）
- **范围**：表结构列表视图 + 表宽高（如 A 未单独做） + 样式优化
- **涉及文件**：`editor_panels.rs`（list view 组件 + resize + canvas draw 路径优化）、`styles.css`（字体回退栈 + 抗锯齿 + 字号梯度）、`editor_core.rs`（store 派生 selectors）
- **数据契约变更**：表 record 加 `height: Option<u32>`（若 A 没做）
- **工作量**：中-大，5-8 天（list view 主导）
- **验收门槛**：列表/画布/Inspector 三视图切换 + 字体视觉回归 + Canvas 帧率 < 16ms
- **可独立交付**：是
- **风险**：list view 与 Inspector 数据同步、Canvas 字体子像素、字号梯度设计

> **变体 C'：若 operator 接受 "4 单独做、5 跟着 C 走"**：把 A 提前做完后，C 只含 1+5。降低单提案复杂度。

### 提案 D: `feat-relation-inference`（需求 2 独立）
- **范围**：连接时不强制选 cardinality，连接多个字段自然推导
- **涉及文件**：`editor_panels.rs` 的 relation 创建/编辑流（确认条去掉 cardinality 下拉）、Inspector reference 面板、cardinality 推导函数（基于两端字段数）、UT-MM-13/14
- **数据契约变更**：可能 `reference.cardinality` 从 String 改为 Enum，或加 `auto_inferred: bool`
- **工作量**：小-中，1-3 天
- **验收门槛**：连接 1 字段 + 1 字段 → 1:1；连接 1 + N → 1:N；连接 M + N → N:N；可手动覆盖
- **可独立交付**：是
- **风险**：老数据的 cardinality 字段如何兼容、Inspector UI 简化的同时保证可读性

### 提案 E: `ux-ergonomics-subset`（需求 6 拆分后挑子集）
- **范围**：operator 圈定方便性子项（候选见 `01-current-state.md` 第 6 节）
- **涉及文件**：取决于子项
- **工作量**：取决于子项
- **验收门槛**：operator 圈定
- **可独立交付**：是
- **风险**：operator 不圈定 = 提案无法启动

## 3. 推荐执行顺序

```
Step 1: A (feat-table-resize)        ← 热身，0.5-1 天，建立节奏
Step 2: D (feat-relation-inference)  ← 小但用户体验提升明显，1-3 天
Step 3: C (ux-canvas-batch)          ← 主力，5-8 天，含 1+5（+4 若 A 未做）
Step 4: B (feat-multiple-datasources) ← 收尾，≥ 1 sprint，最大影响面
Step 5: E (ux-ergonomics-subset)     ← 待 operator 圈定子项
```

**依据**：
- A 最小、最快、风险最低 → 先做积累 guard 切换经验
- D 与画布/列表视图无强耦合，独立可做
- C 是前端呈现层的大头，放在 A 之后可复用 A 的 resize 基础设施
- B 跨最多模块（schema+后端+前端+MCP+测试设施），放最后是因为前 3 个提案完成时团队对前端 store/canvas 已有共识，再扩展到后端数据源更顺
- E 需 operator 圈定子项才能启动

**总工作量估算**（不含 E）：约 1.5-2 sprint（15-20 工作日）。

## 4. 切分对比矩阵

| 方案 | 提案数 | 单提案最大工作量 | guard 切换次数 | 总工作量 |
|---|---|---|---|---|
| 现状 6 项独立 | 6 | 1 sprint (B) | 6 | ~15-20 天 |
| **本推荐 5 案 (A→D→C→B→E)** | **5** | **1 sprint (B)** | **5** | **~15-20 天** |
| 激进合并 1 案 | 1 | ≥ 2 sprint | 1 | 不可控（太大） |
| 三案 (UX+D+数据源) | 3 | 1 sprint (B) | 3 | ~15-20 天 |

**推荐方案**：5 案版本。理由：guard 切换 5 次成本可控，单案最大 1 sprint 不超 proposal 文档上限；激进合并风险太高。

## 5. 与既有 active guard 的关系

- 当前 active guard：`fix-auth-register-redact`（已在 9b89af4 验证 PASS，待 archive）
- operator 排序：5 个提案应**串行**——前一个 archive 后再开下一个
- 不允许 6 项并行提案（openlogos 工具设计）

## 6. 同步约束

- guard 单活跃 ⇒ 任何时候 `logos/.openlogos-guard` 指向**且仅指向**一个 in-flight 提案
- 提案之间不共享 feature branch（openlogos 流程设计）
- 每个提案需独立 verify PASS 才能 archive