# Pending Items

> Extracted from 4 conversation(s) on 2026-03-10

## TODO
| # | Description | Source | Priority |
|---|------------|--------|----------|
| 1 | Phase 4: Rust Web MVP 主流程（技术选型 PoC + 建表/加字段/建关联/保存/加载完整链路） | [Phase 4/5 执行计划](0c609704-251a-4b27-ab37-32a07a1bffca) | High |
| 2 | Phase 5: 灰度到 100% 并满足 SLA（小流量灰度、监控指标、回滚预案与复盘文档） | [Phase 4/5 执行计划](0c609704-251a-4b27-ab37-32a07a1bffca) | Medium |
| 3 | 完善 `backend/src/diagrams/mod.rs` 中 `update_diagram` 事务：删除/重建表关联与引用关联，保证图表及引用更新一致性 | [图表后端逻辑与事务 TODO](0c609704-251a-4b27-ab37-32a07a1bffca) | Medium |
| 4 | 实现 `src/utils/importSQL/oraclesql.js` 中 Oracle SQL 导入 default 值重建逻辑（等 parser 能力就绪后补全） | [SQL 导入/导出 TODO](0c609704-251a-4b27-ab37-32a07a1bffca) | Low |

## Bugs
| # | Description | Source | Severity |
|---|------------|--------|----------|
| 1 | `backend/src/entity/vo/template_vo.rs` 与 `backend/src/templates/mod.rs` 使用 `Template` Entity 而非 `template::Model`，导致 `TemplateVo::from` 签名与调用处不匹配，编译期大量 E0308/E0609 错误 | [前后端打通与模板支持](576a8aeb-c6ac-48ed-b3f1-ef3a3ba0dd98) | High |

## Missing / Unimplemented
| # | Description | Source | Notes |
|---|------------|--------|-------|
| 1 | `init_templates` 目前只是占位函数，尚未真正把前端 6 个模板 seeds 写入后端 `template` 表（`custom=0` 的默认模板初始化逻辑缺失） | [前后端打通与模板支持](576a8aeb-c6ac-48ed-b3f1-ef3a3ba0dd98) | 需要决定 seeds 的最终来源（静态 JSON / 迁移脚本），并实现幂等插入 |

## Summary
- Total pending: 6
- TODO: 4, Bugs: 1, Missing: 1

