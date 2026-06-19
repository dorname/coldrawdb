# 实现任务

## [delta] 规格变更

### Phase 1 — 需求文档（4 项 delta）

- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-02-product-vision.md` — ADDED 新文档（产品背景 / 定位 / 用户画像 / 成功指标）
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-03-pain-points.md` — ADDED 新文档（P01~P04 痛点因果链）
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-04-scenario-detail.md` — ADDED 新文档（S01 / S02 GIVEN/WHEN/THEN 详述）
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-01-requirements.md` — MODIFIED（拆分"约束与边界"为 3 小节 + 新增"场景总览"小节）

### Phase 3-0 — 技术架构（1 项 delta）

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — MODIFIED（Mermaid 系统架构图 + 选型表补理由列 + §11 非功能性约束 + §12 外部依赖与测试策略）

### 元数据 — `logos-project.yaml`（1 项 delta）

- [x] 产出 delta 到 `deltas/logos-project.yaml` — MODIFIED（填 `tech_stack` 11 项 + 新增 `scenarios` 5 条 + 新增 `external_dependencies` + 新增 `resource_index` 18 条索引）

### 自检清单（所有 delta 产出后执行）

- [x] 每个 delta 文件使用 `## ADDED —` / `## MODIFIED —` / `## REMOVED —` 标记
- [x] 每个 ADDED 块对应主文档中的一个完整章节（可独立 merge）
- [x] 每个 MODIFIED 块含修改后完整内容（merge 时整体替换）
- [x] delta 文件路径与目标主文档一一对应（除 `logos-project.yaml` 外，遵循 `deltas/prd/...` 镜像映射）

## [code] 代码实现

（无 — 本次为纯设计级变更，不涉及代码）

## [deploy] 部署任务

（无 — 不需要部署）