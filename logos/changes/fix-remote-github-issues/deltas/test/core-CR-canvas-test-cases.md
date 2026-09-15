# Delta — core-CR-canvas-test-cases.md（#2 聚焦 / #5 多表拖动）

> 变更：fix-remote-github-issues / GitHub #2 #5

## ADDED — UT-CR-FOCUS-01 — focus_transform 使表进入视口

### UT-CR-FOCUS-01 — `focus_transform` 纯函数

- **位置**：`frontend-rs/src/editor_render.rs`（或 `editor_panels.rs`）纯函数
- **步骤**：
  1. 表 AABB 完全在视口外 → 返回的 pan 使表中心落入视口（含 padding）
  2. 表已完全在视口内 → pan 不变或仅允许文档声明的居中微调（断言稳定）
- **断言**：zoom 默认不变；极大表可缩小（若实现）须有单测

## ADDED — UT-CR-FOCUS-02 — 新建表触发聚焦

### UT-CR-FOCUS-02 — on_create_table 后调用 focus

- **位置**：`editor_panels.rs` 锚点 / 行为测试
- **断言**：创建通路在写入 store 后调用聚焦（`include_str!` 含 `focus_table` 或等价符号）

## ADDED — UT-CR-MULTI-01 — 多表同步位移

### UT-CR-MULTI-01 — `translate_tables(ids, dx, dy)`

- **位置**：纯函数或 Command 辅助
- **前置**：3 张表坐标已知；选中其中 2 张
- **步骤**：应用 `Δ=(10,20)`
- **断言**：两张选中表坐标各 +10/+20；未选中表不变

## ADDED — ST-CR-MULTI-01 — 框选后拖动多表

### ST-CR-MULTI-01

- 框选 ≥2 表 → 拖动其中一张 → 全部选中表同步移动 → 松手落账

## MODIFIED — 附录用例 ID 清单（若存在则追加）

| ID | 标题 |
|---|---|
| UT-CR-FOCUS-01 | focus_transform 视口聚焦 |
| UT-CR-FOCUS-02 | 新建表触发 focus |
| UT-CR-MULTI-01 | 多表同步位移 |
| ST-CR-MULTI-01 | 框选后多表拖动 |
