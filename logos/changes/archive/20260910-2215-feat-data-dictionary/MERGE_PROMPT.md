# 合并指令

## 变更提案
- 提案名称：feat-data-dictionary
- 提案目录：logos/changes/feat-data-dictionary/

## 提案内容

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
- 影响的 API：无——字典沿用 Enum 同口径：**前端 state，随 diagram JSON 快照持久化**（core-01c §0 边界），不新增端点
- 影响的 DB 表：无——后端不实体化字典表
- 影响的编排测试：无（无新 API）
- 新增测试：`test/core-S07-test-cases.md`（字典 CRUD / 绑定 / 导出 UT+ST）
- 影响的 smoke 测试：否

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端能力（字典存于 diagram JSON，沿用既有 PUT 快照通路），无服务端/Schema/配置变更
- 影响环境：无
- 是否涉及数据迁移：否（diagram JSON 新增可选 `dictionaries` 数组，旧图无此字段时默认为空，向前兼容）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

新增「数据字典」能力，包含四部分：

1. **字典管理**：侧栏新增「字典」Tab，支持字典 CRUD（名称 + 编码）与字典项管理（value → label 映射，如 `1=男 / 2=女`）；同一图表内字典编码唯一；删除被引用字典前需确认级联影响（沿用 Enum 引用一致性口径）。
2. **字段绑定**：字段 Inspector 增加可选「数据字典」属性，绑定后字典随字段展示；字段注释/文档中体现映射关系。绑定为软引用（按字典编码），解绑不影响字段本身。
3. **持久化与协作**：字典作为 diagram JSON 顶层可选 `dictionaries` 数组存储，沿用 S01 PUT 快照通路；S05 OT 场景下字典编辑作为模型 op 自然同步，协议零变更；S06 MCP 读取图表时可一并返回字典（如改动极小，否则列为边界外）。
4. **导出**：导出数据字典文档（首阶段 Markdown：字典清单 + 各字典映射表 + 字段绑定关系表）；PDManer 的 Word 导出与代码生成（Java Bean / MyBatisPlus 枚举常量）列为边界外后续迭代。

**边界（本变更不做）**：字典驱动的代码生成；Word 导出；跨图表/全局字典库（字典作用域 = 单图表）；字典值对字段取值的运行时校验。


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-00-scenario-overview.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/1-product-requirements/core-00-scenario-overview.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/1-product-requirements/core-01-requirements.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/1-product-requirements/core-01-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/3-technical-plan/2-scenario-implementation/core-S07-data-dictionary.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/prd/3-technical-plan/2-scenario-implementation/core-S07-data-dictionary.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/test/core-S07-test-cases.md

- Delta 文件：`logos/changes/feat-data-dictionary/deltas/test/core-S07-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(feat-data-dictionary): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive feat-data-dictionary`。
