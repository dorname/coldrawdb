# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/1-product-requirements/core-01-requirements.md` — 更新 FR-12 的 camelCase/snake_case 兼容验收
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 定义关系端点与 notNull 规范化规则
- [x] 产出 `deltas/api/diagrams.yaml` — 更新 import payload 兼容性描述
- [x] 产出 `deltas/test/core-S01-test-cases.md` — 新增 UT-S01-17、ST-S01-07

## [code] 代码实现

- [x] 修改 `backend/src/diagram_persistence.rs`，实现四个关系端点别名规范化和 DTO serde alias
- [x] 交付 UT-S01-17、ST-S01-07 与 OpenLogos reporter，验证两种命名风格的关系和 NOT NULL 持久化
- [x] 运行后端导入定向测试与全量测试，确认 #62 错误诊断和 ID 隔离无回归
