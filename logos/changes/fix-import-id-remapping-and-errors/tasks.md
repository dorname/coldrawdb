# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/1-product-requirements/core-01-requirements.md` — 更新 FR-12 的独立导入与错误诊断验收
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 定义实体 ID 重映射、端点回填与反序列化错误口径
- [x] 产出 `deltas/api/diagrams.yaml` — 更新 import 的 200/400/500 契约描述
- [x] 产出 `deltas/test/core-S01-test-cases.md` — 新增 UT-S01-14、UT-S01-15、ST-S01-06 及 reporter 登记

## [code] 代码实现

- [x] 修改 `backend/src/diagram_persistence.rs`，实现全局实体 ID 重映射、关系端点回填和嵌套反序列化错误返回
- [x] 修改 `backend/src/diagrams_v1.rs`，实现 400/500 可诊断响应，并补齐 UT-S01-14、UT-S01-15、ST-S01-06 与 OpenLogos reporter
- [x] 运行后端导入定向测试、全量测试与格式检查，保留重复导入验证结果（97/97 通过；全仓 rustfmt 仍有既有格式债，本提案未扩散格式化）
