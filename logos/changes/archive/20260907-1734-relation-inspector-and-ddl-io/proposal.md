# 变更提案：relation-inspector-and-ddl-io

> module: core | created: 2026-09-07

## 变更原因

用户真机反馈三项优化：

1. **关系连线弹窗冗余**：点击关系连线会弹「关系详情」模态，但右侧 Inspector 已展示同样的详情（table_1.id → table_2.id、cardinality），弹窗是重复信息通路。期望：点击连线 = 选中 + Inspector 展示；删除走 Delete/Backspace 快捷键（`is_delete_key` 已双键覆盖，ST-PB-04 已落账验证）。
2. **DDL 导入形同虚设**：导入抽屉 SQL 格式的解析是最小桩（`parse_sql_import_tables` 只提表名，每张表硬造一个 `id INT` 字段），真实 DDL 的列/类型/约束全部丢失。
3. **建表语句导出不完整**：`export_diagram_sql` 只输出 CREATE TABLE + 类型 + PK/NOT NULL，UNIQUE/自增/DEFAULT/COMMENT 丢失，外键 `references` 参数被忽略（规格 core-01d §5.2 本就声明 V1 含 FK 引用，实现未兑现）。

## 变更类型

设计级变更（功能规格 + 原型 + 测试用例 + 代码）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `core-01b-relationship.md`（§3 关系操作：点击连线交互从「弹详情模态」改为「选中 + Inspector 展示」；删除口径收敛为 Delete/Backspace + Inspector 删除按钮）
  - `core-01d-import-export.md`（§4.4 导入提交行为：SQL 导入升级为列级 DDL 解析；§5.2 SQL 导出预览口径补全）
- 影响的原型：`core-01-editor-prototype.html`（连线点击不再弹模态；importModel SQL 解析与 exportContent SQL 导出口径对齐——先行）
- 影响的业务场景：PB（关系）、PC（导入导出）
- 影响的部署方案：无
- 影响的 API：无（bridge import 只持久化 payload，解析在客户端；API 契约不变）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 ST/UT 用例：
  - `core-PB-relationship-test-cases.md`：ST-PB-03（MODIFIED：点击连线 → 选中 + Inspector 详情，不再弹模态；删除走 Inspector 按钮/键盘）、ST-PB-04（保持，补 Backspace 断言）
  - `core-PC-import-export-test-cases.md`：UT-PC-02（MODIFIED：SQL 导出断言补全 UNIQUE/自增/DEFAULT/FK）；新增 UT-PC-07（DDL 列级导入解析纯函数）、UT-PC-08（DDL 导入外键 REFERENCES → references 生成）；ST-PC-01 断言同步（摘要口径不变，导入结果含真实列）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端变更（交互简化 + 客户端解析/生成纯函数增强），bridge API 契约与后端行为不变
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **关系连线交互收敛**：`on_reference_pick` 不再弹 `ModalKind::ReferenceDetail` 模态，只做选中（`SelectionKind::Reference`）+ 开 Inspector；关系详情模态组件及其 testid（`modal-reference-detail` 等）移除。删除路径 = Delete/Backspace 快捷键（已支持）+ Inspector 关系面板删除按钮（已存在）。
2. **DDL 导入列级解析**（`parse_sql_import_tables` 重写）：
   - 列定义：列名、类型整串（含 `VARCHAR(32)`/`DECIMAL(10,2)` 参数化保留）、列级 PRIMARY KEY / NOT NULL / UNIQUE / AUTO_INCREMENT（PG `SERIAL` 系类型 → 自增）/ DEFAULT 值 / COMMENT '...'
   - 表级约束：`PRIMARY KEY(a[,b])`（联合主键）、`FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `REFERENCES t(c2)` → 生成 references 连线（类型按 core-01b 推导口径）
   - 引号/括号规范化：反引号、双引号、方括号去除；未知类型原样保留整串
   - V1 不做：CHECK、INDEX/KEY 索引定义、ALTER TABLE 序列、视图/触发器、引擎方言全量映射
3. **DDL 导出补全**（`export_diagram_sql`）：列级追加 UNIQUE / DEFAULT / 自增（引擎映射：mysql → `AUTO_INCREMENT`；postgresql → 由 `SERIAL` 类型承担不追加；generic → `AUTO_INCREMENT`）/ COMMENT（mysql 行内 `COMMENT 'x'`；其余引擎输出 `COMMENT ON COLUMN t.c IS 'x'` 后置语句）；表末或表后追加 `FOREIGN KEY (c) REFERENCES t(c2)` 约束（启用 `_references` 参数，兑现 §5.2 既定口径）。

执行顺序：先原型后代码（沿用惯例）。
