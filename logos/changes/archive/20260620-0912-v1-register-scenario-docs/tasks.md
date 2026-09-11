# 实现任务

## [delta] 规格变更

### Phase 3-1 — 场景建模（1 项 delta）

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md` — ADDED 新文档（Phase 3 技术实现状态总览：场景地图 / 依赖关系 / 场景索引 / 与 Phase 1 总览的引用关系）

### 元数据 — `logos-project.yaml`（1 项 delta）

- [x] 产出 delta 到 `deltas/logos-project.yaml` — MODIFIED（在 `resource_index` 末尾追加 5 条：Phase 3 场景实现概览 + S01/S02 时序图 + S01/S02 编排 JSON）

### 自检清单（所有 delta 产出后执行）

- [x] 每个 delta 文件使用 `## ADDED —` / `## MODIFIED —` 标记
- [x] 每个 ADDED 块对应主文档中的一个完整章节（可独立 merge）
- [x] 每个 MODIFIED 块含修改后完整内容（merge 时整体替换）
- [x] delta 文件路径与目标主文档一一对应（除 `logos-project.yaml` 外，遵循 `deltas/prd/...` 镜像映射）
- [x] Phase 3 概览中明确标注 S03–S05 为「V2 deferred」，与 `logos-project.yaml` 的 `scenarios` 字段保持一致

## [code] 代码实现

（无 — 本次为纯设计级变更，不涉及代码）

## [deploy] 部署任务

（无 — 不需要部署）