# 产品优化批次 · 开放问题清单（条目6 切片 3/3）

> 草稿，**未创建** `logos/changes/` 目录，未运行 `openlogos change`。
> 等待 operator 裁决后，外环下一条 steer 可派发"首个提案的 proposal.md + tasks.md 草案"。

## Q1 · 需求 1 列表视图的功能边界

**问题**：参考 pdmaner 的列表视图具体包括哪些能力？

**候选能力**（请 operator 圈定）：
- [ ] 表名/字段名/类型 表格化展示
- [ ] 排序（按任意列）
- [ ] 过滤（按名称模糊匹配 / 按类型 / 按是否有索引）
- [ ] 批量重命名（多表或多字段一次性改名）
- [ ] 批量改类型（多字段一次性改类型）
- [ ] 双击跳到画布对应表
- [ ] 导出 CSV / Excel
- [ ] 列宽可调
- [ ] 表/字段分组（按 schema / 按 tag）

**影响**：圈定的子项决定 `ux-canvas-batch` 提案的工作量（3 天 vs 8 天差距）。

## Q2 · 需求 2 关系推导的覆盖规则

**问题**：连接多个字段，"自然推导"的具体规则是？

**候选规则**：
- 端 A 字段数 + 端 B 字段数 → cardinality 映射：
  - 1 + 1 → one_to_one
  - 1 + N → one_to_many（或 many_to_one，取决于从哪端看）
  - N + N → many_to_many
- 字段顺序：连接时按用户点击顺序还是表 schema 顺序？
- 是否允许用户**手动覆盖**推导结果？
- DB schema：`reference` 表是 `end_field_id` 单字段（现状）还是数组？

**影响**：决定 relation 创建流 UX 改动幅度 + 数据 schema 是否需变。

## Q3 · 需求 3 PG/MySQL 支持的"程度"

**问题**：operator 说的"开始支持 PG/MySQL"是指？

**候选程度**（请 operator 圈定）：
- **程度 A（最小）**：导出 SQL 时支持 PG/MySQL 方言（当前导出 SQL 是 dialect-agnostic，可声明 target dialect 输出引号/类型映射）—— 工作量 1-2 天
- **程度 B（中）**：A + datasource 连接配置（用户保存 PG/MySQL 连接串，前端展示但不执行）—— 工作量 3-5 天
- **程度 C（完整）**：B + 在线 introspect（连接到真实 PG/MySQL 实例，读 schema 回来生成 diagram）—— 工作量 5-10 天
- **程度 D（+MCP）**：C + MCP `mcp__datasource__*` 工具族（让 AI 客户端能 introspect/执行 DDL）—— 工作量 +2-3 天

**强烈推荐程度 C 或 D**：因为仅程度 A 等于把活推给用户手动复制粘贴 SQL，价值有限；程度 C/D 才是真正"开始支持"。

## Q4 · 需求 4 表宽高的"高度"语义

**问题**：表的高度调整是？

**候选语义**：
- **绝对高度**：用户输入数值（如 `height: 200`），表按该高度渲染，超出部分滚动
- **最小高度**：用户设最小值，字段多时自动撑高（**推荐**）
- **自动高度**：完全由字段数决定，不允许手动调（与当前一致，不算新功能）

**推荐最小高度**：用户控制"默认紧凑/宽松"，但具体像素高度由字段数 + 行高算出。

## Q5 · 需求 5 样式优化的具体维度

**问题**：operator 说的"字体清晰度、交互流畅性"具体期望？

**候选维度**：
- **字体**：
  - 全局回退栈：`system-ui, -apple-system, "Segoe UI", ...`
  - 子像素抗锯齿：`-webkit-font-smoothing: antialiased`
  - Canvas 文本走离屏 cache
  - 中文字体支持（思源黑体 / 苹方）
- **流畅性**：
  - 关键交互帧率 < 16ms（拖拽/连线/Inspector 切换）
  - reduced-motion 媒体查询支持（已有 UT-MM 覆盖）
  - requestAnimationFrame 统一调度
  - 大图（>200 表）虚拟化

**请 operator 圈定子集**。

## Q6 · 需求 6"用户方便性"的具象化

**问题**：operator 没说具体子项，请圈定：

**候选子项**（按工作量排序）：
| 子项 | 工作量 | UX 影响 |
|---|---|---|
| 快捷键可发现性（⌘K + ? 帮助） | 小 | 高 |
| 错误码 → 中文文案映射 | 小 | 中 |
| 撤销栈 History 面板 | 中 | 高 |
| 表/字段批量重命名 | 中 | 中 |
| 自动保存可关闭 | 小 | 中 |
| 移动端 480px 降级 | 中 | 中 |
| 首次进入引导 + 模板库 | 中 | 高 |
| 字段拖拽排序 | 小 | 中 |

**请 operator 圈定 2-4 个优先做**。

## Q7 · 串行 5 案的总工作量是否接受

**问题**：5 案串行总工作量约 15-20 工作日（1.5-2 sprint）。operator 是否接受？

**备选**：
- **接受**：5 案按推荐顺序串行（推荐）
- **拒绝**：合并某些案（如 D 合并到 C）减到 3-4 案
- **加速**：增加并发人手（需 operator 决定）

## Q8 · guard 切换的 metadata 规范

**问题**：openlogos 流程中，guard 从提案 A 切到 B 的 metadata 记录约定？

**当前观察**：`logos/.openlogos-guard` 是单 JSON，切换时直接覆盖。需要 operator 确认：
- 是否要在 commit message 里引用旧 guard 名？
- 是否要在黑板记录 guard 切换事件？

## Q9 · 首案提案草案的"模板"

**问题**：外环下一条 steer 派发"首案 proposal.md + tasks.md 草案"，operator 期望的详细程度？

**候选模板**：
- **精简**：Why/What/范围/影响分析（4 段） + tasks checklist（5-8 项）
- **标准**：精简 + 数据契约变更 + 验收门槛 + 风险点（详尽）
- **完整**：标准 + 替代方案否决理由 + 部署影响 + UI 声明 + 关联场景编号

**推荐完整模板**（与既有 proposal.md 风格一致，如 fix-auth-register-redact）。

## Q10 · 时间线预期

**问题**：operator 对 5 案的时间线预期？

**默认假设**：1.5-2 sprint 完成 A→D→C→B，E 待子项圈定。

**是否接受**：请 operator 明确（用于决定是否拆分批次到多个 sprint）。