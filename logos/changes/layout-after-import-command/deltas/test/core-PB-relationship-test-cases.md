# Delta — core-PB-relationship-test-cases.md（修改）

> 模块：core | 提案：layout-after-import-command
> 关联 issue：#23 | 规格：`core-01b-relationship.md` §4.5

## ADDED — UT-PB-16 — 力导向布局确定性 / 无重叠 / 孤立不动（#23）

- **位置**：`frontend-rs/src/layout.rs::force_directed_layout`
- **参数**：iterations=100，spacing=180，seed=42
- **断言**：
  1. 相同输入两次结果完全一致（确定性）
  2. 连通表两两欧氏距离 > 50（无重叠）
  3. 无边表坐标不变（允许 jitter 扰动 < 5）
  4. 无边 / 空表：原样返回

## ADDED — ST-PB-09 — Command Palette「整理布局」触发（#23）

- **GIVEN**：画布含 ≥2 张连通表（有关系），坐标故意重叠或过近
- **WHEN**：打开 Command Palette → 选中 `palette-action-layout` / id=`action:layout`
- **THEN**：连通表 x/y 相对触发前发生变化；store dirty；孤立表（若有）坐标不变

> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。

### 用例登记追加

| ID | 描述 |
|---|---|
| UT-PB-16 | 力导向确定性 / 无重叠 / 孤立不动 |
| ST-PB-09 | palette 整理布局触发 |
