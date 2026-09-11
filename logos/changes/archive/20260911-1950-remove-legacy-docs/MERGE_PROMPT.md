# 合并指令

## 变更提案
- 提案名称：remove-legacy-docs
- 提案目录：logos/changes/remove-legacy-docs/

## 提案内容

# 变更提案：remove-legacy-docs

> module: core | created: 2026-09-11

## 变更原因
`docs/` 目录（phase0～phase4 过程文档、MILESTONE_V1_INITIAL、OUTER_LOOP_PROTOCOL、drawdb-capability-checklist 等共 39 个文件约 1.6MB）是「React → Rust Web 重构」期间的历史过程产物，内容已过期且与项目现状不符（如 PHASE4_DONE 描述的重构过程细节、capability-checklist 的 drawdb 对账清单均已随 V1 收官失效）。用户确认整目录删除；其中 README 头图 `docs/assets/app-editor.png` 为现行资产需先迁移保留。删除后须同步修订 logos 规格中的全部 docs/ 引用（经全仓 grep 清点共 11 个规格文件、19 行），避免悬空引用。

## 变更类型
设计级（文档资产清理：历史过程文档整目录删除 + 规格文档「对齐参考源 / 验收条件」章节修订；不涉及运行时行为与 API / DB 契约）

## 变更范围
- 影响的需求文档：`core-01-requirements.md`（§6 验收条件 1 条历史化）、`core-02-product-vision.md`（§1.2 G1 行 + §1.4 launch gate 1 条历史化）
- 影响的功能规格：`core-00-information-architecture.md` §7、`core-01-editor-canvas.md` §10、`core-01c-index-enum-custom-type.md` §8、`core-02-diagram-persistence.md` §10、`core-03-bridge-io.md` §12、`core-04-side-panel-tabs.md` §13、`core-05-top-menu-modals.md` §9（均为「对齐参考源」章节去 docs/ 引用）
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 另涉及（非规格）：`core-implementation-checklist.md` §12、`core-01-deployment-plan.md` §11「对齐参考源」；根 `README.md`、`RUST_WEB_REFACTOR_PLAN.md`、`.github/workflows/build.yml`、`frontend-rs/tests/soak/4h.sh`、`frontend-rs/scripts/capture-readme-screenshot.mjs`

## 部署影响
- 是否需要部署：否
- 部署原因：纯文档与 CI / 辅助脚本路径修正，无运行时行为变更
- 影响环境：无（仅 CI workflow 的 perf 产物输出路径调整）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（git 历史即回滚手段，被删文档均可溯）
- 是否需要 smoke：否

## 变更概述
1. **[delta]** 修订 11 个规格文档中的全部 docs/ 引用：9 处「对齐参考源」章节移除 docs/ 引用行并追加统一墓碑注记（原 docs/ 过程文档已随本提案移除、内容可溯 git 历史）；需求与愿景文档中 3 处已完成的 V1 验收条目改为「已核验 + 墓碑注记」的历史表述。
2. **[code]** 删除 `docs/` 整目录（头图 `app-editor.png` 先迁移至仓库根）；修正全部悬空引用：README（头图路径、架构行、文档索引 2 行）、`RUST_WEB_REFACTOR_PLAN.md`（顶部历史声明）、`build.yml`（删除 mmdc 渲染步骤、perf 产物改写 `perf-results/`）、`soak/4h.sh`（RESULTS 默认值）、`capture-readme-screenshot.mjs`（OUT_DIR 改指仓库根）。


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-01-requirements.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/1-product-requirements/core-01-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/1-product-requirements/core-02-product-vision.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/1-product-requirements/core-02-product-vision.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/remove-legacy-docs/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
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
   git add -A && git commit -m "docs(remove-legacy-docs): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive remove-legacy-docs`。
