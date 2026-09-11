# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/1-product-requirements/core-00-scenario-overview.md` — 场景索引/映射表新增 S07「管理数据字典并绑定字段」
- [x] 产出 delta 文件到 `deltas/prd/1-product-requirements/core-01-requirements.md` — 新增数据字典 US/FR 与验收条件
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01e-data-dictionary.md` — 新增字典管理/绑定/导出功能规格（新文件）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — §10 移除「数据字典」排除项，侧栏新增字典 Tab 规格
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — 字段 Inspector 新增「绑定字典」属性
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 主原型补充字典 Tab 与字段绑定交互
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S07-data-dictionary.md` — S07 场景时序图（前端派生 + 快照持久化 + OT 同步口径）
- [x] 产出 delta 文件到 `deltas/test/core-S07-test-cases.md` — S07 UT/ST 用例（字典 CRUD / 绑定 / 导出 / 引用删除）
- [x] 更新 `logos/logos-project.yaml`（merge 时）— scenarios 新增 S07、`scenario_counter.next_id` 7→8、resource_index 登记新文档

## [code] 代码实现
- [x] 后端 migration `0007_data_dictionary.up/down.sql`：`diagram.dictionaries TEXT` + `field.dict_code VARCHAR DEFAULT ''`
- [x] 后端 DTO 接线：`DiagramFull.dictionaries`、`FieldDto.dict_code`，save/load 读写两列（含 JSON 解析容错）
- [x] 前端模型层新增 Dictionary/DictionaryItem 类型与 diagram JSON `dictionaries` 序列化（向后兼容缺省为空）；Field 新增 `dict_code`
- [x] 侧栏新增字典 Tab：字典列表 CRUD + 字典项 value/label 编辑（落地形态：ToolRail `toolrail-dicts` + DictPanel 抽屉，对齐主原型——生产前端 LeftPanel 为死区，侧栏 Tab 体系未在线上）
- [x] 字段 Inspector 新增「绑定字典」下拉（软引用，按字典编码）
- [x] 删除被引用字典的确认与级联口径实现（confirm-dict 模态 + 单 undo 单元级联置空）
- [x] 数据字典 Markdown 导出（DictPanel `dict-export` + ExportDrawer「字典」格式页签）
- [x] 编写对应 UT/ST 测试代码 + OpenLogos reporter（写入 logos/resources/verify/test-results.jsonl）
