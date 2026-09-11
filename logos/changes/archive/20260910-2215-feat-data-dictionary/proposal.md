# 变更提案：feat-data-dictionary

> module: core | created: 2026-09-10

## 变更原因

新增需求：支持 PDManer 式「数据字典」能力——**代码映射表管理**：集中定义字段的可枚举取值及其含义（如 `0=否 / 1=是`、`1=男 / 2=女`），供各表字段绑定复用，避免重复维护同义映射（如「是否启用」「是否有效」复用同一条 Yes/No 字典）。

现行规格 `core-04-side-panel-tabs.md` §10 曾将「数据字典」列为排除项（当时排除的是 PDManer 左树体系整体）；本变更以独立形态引入字典管理，与既有 Enum（字段类型级枚举，core-01c）互补：**Enum 决定字段的存储类型，数据字典描述字段取值的业务语义映射**，二者不冲突。

## 变更类型

需求级（新增场景 S07「管理数据字典并绑定字段」，需全链路：需求 → 设计 → 时序图 → 测试）

## 变更范围

- 影响的需求文档：
  - `prd/1-product-requirements/core-00-scenario-overview.md` — 场景索引新增 S07
  - `prd/1-product-requirements/core-01-requirements.md` — 新增数据字典 US/FR 与验收条件
  - `logos/logos-project.yaml` — scenarios 新增 S07、`scenario_counter.next_id` 7→8（merge 时同步）
- 影响的功能规格：
  - 新增 `prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md` — 字典 CRUD / 字典项 value-label 映射 / 字段绑定 / 导出规格
  - `prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — §10 边界更新（移除「数据字典」排除表述），侧栏新增字典 Tab
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — 字段 Inspector 增加「绑定字典」属性
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 主原型补充字典 Tab 与字段绑定交互
- 影响的业务场景：新增 S07；字典内容随 S01 保存通路持久化，随 S05 OT 模型 op 自然同步（协议零变更）
- 影响的部署方案：无
- 影响的 API：无新端点——复用 S01 `PUT/GET /diagrams/{id}`；后端 `DiagramFull`/`FieldDto` DTO 扩展 `dictionaries`/`dict_code` 字段（实现期修正，见下）
- 影响的 DB 表：**新增 migration `0007_data_dictionary`**：`diagram` 表加 `dictionaries TEXT` 列（JSON blob）、`field` 表加 `dict_code VARCHAR NOT NULL DEFAULT ''` 列（实现期修正——原假设「零 DB 变更」不成立：非房间图为规范化表存储，未知 JSON 字段会在 PUT 往返中丢弃；房间图 `doc_json` 不受影响）
- 影响的编排测试：无（无新 API）
- 新增测试：`test/core-S07-test-cases.md`（字典 CRUD / 绑定 / 导出 UT+ST）
- 影响的 smoke 测试：否

## 部署影响

- 是否需要部署：否
- 部署原因：migration 由后端启动时自动执行（`init.rs` + `schema_migrations`，同 0006 先例）；自托管发版即生效，无独立部署动作
- 影响环境：无
- 是否涉及数据迁移：是（自动）——`0007` 加两列，默认值空/NULL；旧数据零改动，向后兼容
- 是否需要回滚预案：否（`0007_data_dictionary.down.sql` 提供列回滚）
- 是否需要 smoke：否

## 变更概述

新增「数据字典」能力，包含四部分：

1. **字典管理**：侧栏新增「字典」Tab，支持字典 CRUD（名称 + 编码）与字典项管理（value → label 映射，如 `1=男 / 2=女`）；同一图表内字典编码唯一；删除被引用字典前需确认级联影响（沿用 Enum 引用一致性口径）。
2. **字段绑定**：字段 Inspector 增加可选「数据字典」属性，绑定后字典随字段展示；字段注释/文档中体现映射关系。绑定为软引用（按字典编码），解绑不影响字段本身。
3. **持久化与协作**：字典作为 diagram JSON 顶层可选 `dictionaries` 数组随 S01 快照通路传输；后端以 `diagram.dictionaries`（JSON blob 列）+ `field.dict_code` 列承载持久化（migration 0007，启动自动执行）；S05 房间图经 `doc_json` 自然透出，OT 协议零变更；S06 MCP 读取图表时可一并返回字典（自然透出，不扩展工具）。
4. **导出**：导出数据字典文档（首阶段 Markdown：字典清单 + 各字典映射表 + 字段绑定关系表）；PDManer 的 Word 导出与代码生成（Java Bean / MyBatisPlus 枚举常量）列为边界外后续迭代。

**边界（本变更不做）**：字典驱动的代码生成；Word 导出；跨图表/全局字典库（字典作用域 = 单图表）；字典值对字段取值的运行时校验。
