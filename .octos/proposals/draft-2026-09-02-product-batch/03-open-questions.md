# 产品优化批次 · 开放问题清单（条目6 切片 3/3，已回写 operator 裁决）

> 草稿，**未创建** `logos/changes/` 目录，未运行 `openlogos change`。
> 裁决来源：黑板 `.octos/OUTER_LOOP_REVIEW.md` 条目6 尾部 operator 批注（2026-09-02）。

## Q1 · 需求 1 列表视图的功能边界 — ✅ **operator 已裁决**

**裁决结论**：**全部 9 项候选**都做（全量列表视图能力）。

**采纳的子项**（operator 圈定 ALL）：
- [x] 表名/字段名/类型 表格化展示
- [x] 排序（按任意列）
- [x] 过滤（按名称模糊匹配 / 按类型 / 按是否有索引）
- [x] 批量重命名（多表或多字段一次性改名）
- [x] 批量改类型（多字段一次性改类型）
- [x] 双击跳到画布对应表
- [x] 导出 CSV / Excel
- [x] 列宽可调
- [x] 表/字段分组（按 schema / 按 tag）

**对 `ux-canvas-batch` 提案的影响**：工作量按 **8 天档位** 估（从原 5-8 天上调）。

---

## Q2 · 关系推导的覆盖规则 — ✅ **operator 已裁决**

**裁决结论**：
- **推导规则**：1+1→1:1、1+N→1:N、N+N→N:N（与内环建议一致）
- **字段顺序**：**按用户点击顺序**（operator 明确）
- **是否允许手动覆盖**：**允许**（operator 明确"允许手动覆盖"）

**对 `feat-relation-inference` 提案的影响**：
- Inspector reference 面板需保留 cardinality 编辑器（用户可改）
- 字段在 relation 创建时记录顺序（`reference.field_order: Vec<String>` 或追加元数据）
- 老数据 backward-compat：现有 `reference.cardinality` 字段保留，新规则只影响新建

---

## Q3 · PG/MySQL 支持的"程度" — ✅ **operator 已裁决**

**裁决结论**：**程度 D**（最完整）

**采纳的子能力**：
- ✅ A：导出 SQL 时支持 PG/MySQL 方言
- ✅ B：datasource 连接配置（保存 PG/MySQL 连接串）
- ✅ C：在线 introspect（连接到真实实例，读 schema 回来生成 diagram）
- ✅ D + MCP：`mcp__datasource__*` 工具族（让 AI 客户端能 introspect/执行 DDL）

**对 `feat-multiple-datasources` 提案的影响**：
- 工作量上调至 **1.5-2 sprint**（D 比 C 多 +2-3 天 MCP）
- 必须包含：datasource CRUD + 加密 secret + 连接池 + introspect + MCP 工具族 + docker-compose 测试设施

---

## Q4 · 表宽高的"高度"语义 — ✅ **operator 已裁决**

**裁决结论**：**最小高度语义**（与内环推荐一致）

- 用户控制"最小高度"（数值输入）
- 字段多时按字段数 + 行高**自动撑高**
- 渲染时实际高度 = max(最小高度, 字段数 × 行高)

**对 `feat-table-resize` 提案的影响**：
- Inspector 暴露"min height"字段（输入框，仿 `parse_table_width`）
- Canvas 渲染：实际高度 = max(min_height, fields × row_height)
- 新增 `parse_table_height` 纯函数 + UT-MM-12 测试

---

## Q5 · 样式优化的具体维度 — ✅ **外环代决**

**裁决结论**（外环代决，明确子集）：
- ✅ 字体回退栈（`system-ui, -apple-system, "Segoe UI", ...`）
- ✅ 子像素抗锯齿（`-webkit-font-smoothing: antialiased`）
- ✅ 中文字体支持（思源黑体 / 苹方）
- ✅ Canvas 文本离屏缓存
- ✅ 关键交互 < 16ms/帧（拖拽/连线/Inspector 切换）
- ✅ requestAnimationFrame 统一调度
- ❌ 大图（>200 表）虚拟化 — **暂缓**（性能专项另立提案）

**对 `ux-canvas-batch` 提案的影响**：性能子项排除后，工作量从 5-8 天下调到 **4-6 天**（视觉为主，性能子项保留最小 rAF + 16ms 目标）。

---

## Q6 · 方便性子项圈定 — ✅ **外环代决**

**裁决结论**（外环代决，圈定 4 项）：
- ✅ 快捷键可发现性（`?` 弹帮助，与现有 ST-KB-CMD-01 ⌘K 命令面板配套）
- ✅ 错误码 → 中文文案映射
- ✅ 撤销栈 History 面板
- ✅ 字段拖拽排序

**未采纳**（operator 已说明理由）：
- 批量重命名：Q1 全选覆盖，归 C 案不重复
- 自动保存可关闭、移动端 480px、首次引导+模板库：暂缓（待后续批次）

**对 `ux-ergonomics-subset` 提案的影响**：工作量约 **3-4 天**（4 子项各 0.5-1 天）。

---

## Q7 · 串行 5 案总工作量是否接受 — ✅ **operator 已裁决**

**裁决结论**：**接受 5 案串行 15-20 工作日**（含 operator 调档后的工作量）

---

## Q8 · guard 切换 metadata 规范 — ✅ **外环已裁决**

**裁决结论**：**无需新增规范**。guard 切换由 `openlogos archive` + `openlogos change` 两个 CLI 动作天然留痕（commit + 黑板批注），维持现状。

---

## Q9 · 首案 proposal.md 模板 — ✅ **外环已裁决**

**裁决结论**：采用**完整模板**（与 `fix-auth-register-redact/proposal.md` 风格一致）：
- Why / What / 范围 / 数据契约变更 / 验收门槛 / 风险 / 替代方案否决理由 / 部署影响 / UI 声明 / 关联场景

**本切片已应用**：见 `04-feat-table-resize-proposal.md` 与 `05-feat-table-resize-tasks.md`。

---

## Q10 · 时间线预期 — ✅ **operator 已裁决**

**裁决结论**：**接受 5 案串行 15-20 工作日**（与 Q7 一致）。

**更新后的工作量矩阵**（应用 Q1/Q3/Q5/Q6 裁决）：

| 序 | 提案 | 工作量（裁决后） | 原估 |
|---|---|---|---|
| A | `feat-table-resize` | 0.5-1 天 | 0.5-1 天 |
| D | `feat-relation-inference` | 1-3 天 | 1-3 天 |
| C | `ux-canvas-batch` | 7-10 天 | 5-8 天（Q1 上调）+（Q5 下调）= 净上调 |
| B | `feat-multiple-datasources` | 1.5-2 sprint | ≥ 1 sprint（Q3 上调）|
| E | `ux-ergonomics-subset` | 3-4 天 | 待定（Q6 圈 4 子项）|
| **总计** | — | **约 20-25 工作日** | 15-20 工作日 |

**说明**：总盘上调 5 天，主要因 Q1 列表视图全量能力 + Q3 PG/MySQL 升至 D 档。operator 接受。

---

## 待执行（切片 2 任务）

- ✅ Q1-Q10 已逐题标注裁决
- ⏳ 起草首案 A `feat-table-resize` proposal.md + tasks.md（Q4 最小高度语义）
- ⏳ 修正切片 1 三处文档问题（§5 active guard + 切分原则4 + 工作量统一）