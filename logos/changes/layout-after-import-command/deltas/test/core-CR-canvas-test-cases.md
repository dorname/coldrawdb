# Delta — core-CR-canvas-test-cases.md（修改）

> 模块：core | 提案：layout-after-import-command
> 关联 issue：#23

## ADDED — UT-CR-LAYOUT-01 — 力导向布局（画布侧登记）（#23）

- **位置**：`frontend-rs/src/layout.rs`（与 UT-PB-16 同源断言）
- **断言**：确定性；连通表无重叠（距离 > 50）；孤立表不动；无边原样返回
- **交叉引用**：`core-01b-relationship.md` §4.5 / UT-PB-16

### 用例登记（verify 表行解析）

| ID | 描述 |
|---|---|
| UT-CR-LAYOUT-01 | 力导向：确定性 / 无重叠 / 孤立不动 |
