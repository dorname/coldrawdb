# Delta: remove-legacy-docs — core-02-diagram-persistence.md

> 目标主文档：`logos/resources/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md`
> 提案：remove-legacy-docs | 日期：2026-09-11

## MODIFIED — 10. 对齐参考源

## 10. 对齐参考源

- drawdb §2.5 持久化语义
- drawdb `src/utils/saveToLocal.js`（drawdb 客户端 localStorage 策略）
- `backend/src/diagrams_v1.rs`（5 端点 Rust 路由）
- `backend/src/diagrams/`, `backend/src/fields/`, `backend/src/references/`, `backend/src/areas/`, `backend/src/notes/`（5 个领域子模块）
- `backend/src/tables/` + `backend/src/indices/`（含 table_link / indice_link 关联表）
- `backend/init.sql`（11 张表 DDL）
- `database_design.json`（字段命名对账）
- （历史）原 `docs/drawdb-capability-checklist.md` §2.5 已随提案 remove-legacy-docs 移除，内容可溯 git 历史
