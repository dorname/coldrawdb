# Delta: remove-legacy-docs — core-02-product-vision.md

> 目标主文档：`logos/resources/prd/1-product-requirements/core-02-product-vision.md`
> 提案：remove-legacy-docs | 日期：2026-09-11

## MODIFIED — 1.2 核心目标

### 1.2 核心目标

| 目标 | 衡量指标 | 验收方式 |
|---|---|---|
| G1 | drawdb 主分支能力对齐 | V1 收官时已核验（原能力清单已随提案 remove-legacy-docs 移除，内容可溯 git 历史） |
| G2 | 编辑响应 P95 < 200ms | W4 perf 实测：100 张表 / 200 条关系 / 60fps |
| G3 | 自动保存 1s debounce | PUT 触发；失败重试；409 弹冲突对话框 |
| G4 | 7 引擎 SQL 导入导出 | MySQL / PostgreSQL / SQLite / MariaDB / MSSQL / OracleSQL / Generic |
| G5 | 部署零运维 | Docker 单文件 + GitHub Actions CI green |
| G6 | 11 张表可读写无错 | init.sql + database_design.json 双轨对齐 |

## MODIFIED — 1.4 成功指标（Success Metrics）

### 1.4 成功指标（Success Metrics）

#### V1 launch gate（必须全部满足）

- [ ] Phase 4 CI 全绿
- [x] drawdb 能力清单 ✅ 行 ≥ 95% 可演示（V1 收官时已核验；原清单已随提案 remove-legacy-docs 移除，内容可溯 git 历史）
- [ ] P95 编辑响应 < 200ms（W4 perf 实测）
- [ ] 11 张表可读写无错
- [ ] 7 引擎 SQL 导入导出可演示
- [ ] 409 revision 冲突可演示

#### V1 launch 后的扩展指标（V2 候选，非 gate）

- DAU/MAU 留存率 > 30%
- 单实例承载 100 并发用户
- Docker 部署安装 < 5 分钟
- 部署后 7 天内崩溃率 < 0.1%
