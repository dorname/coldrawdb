# 合并指令

## 变更提案
- 提案名称：sync-tech-plan-with-product-design
- 提案目录：logos/changes/sync-tech-plan-with-product-design/

## 提案内容

# 变更提案：sync-tech-plan-with-product-design

> module: core | created: 2026-06-21

## 变更原因

Phase 2 产品设计在 `optimize-ui-prototypes`（2026-06-21）与 redesign phases A–E 归档后已更新：V2 布局（AppBar / ToolRail / Inspector）、E4 Command Palette + Code View、`?share=` 分享 URL、S03–S05 交互设计文档等。Phase 3 技术计划（S01/S02 时序图、架构数据流）与 `core-implementation-checklist.md` 仍引用旧 V1 模式（`/editor/{id}` 路由、EditorPanels 单一入口、缺失 E4 辅路径），与 Phase 2 不对齐，影响 Step 5 实现追溯。

## 变更类型

设计级变更（Documentation / Specification Only）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（以 Phase 2 为输入源，不反向修改）
- 影响的业务场景：S01（主）、S02（主）、S03–S05（索引与交叉引用）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：**否**
- 部署原因：仅规格文档对齐
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **S01 时序图**：补充 AppBar / ToolRail / Inspector / ModalRoot / CommandPalette / CodeView 参与者；对齐 debounce 保存状态锚点与 409 模态 testid；新增 E4 辅路径（无 HTTP）；网络重试策略对齐 Phase 2（3s / 6s / 12s，封顶 30s）。
2. **S02 时序图**：分享 URL 从 `/editor/{id}` 改为 `/?share=` / `/editor?share=`；补充 Share 模态生成链接与 Landing 冷启动分支；错误文案对齐 Phase 2。
3. **场景总览**：V1/V2 索引表增加 Phase 2 设计文档列；S03–S05 状态从「deferred」更新为「规格完成」。
4. **架构概要**：§5 数据流补充 Command Palette / Code View 客户端路径；交叉引用 Phase 2 设计文档。
5. **实现清单**：补充 E4/E5/E6 设计系统与 Command Palette / Code View 模块状态；更新文档完成度统计。


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md

- Delta 文件：`logos/changes/sync-tech-plan-with-product-design/deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md`
- 目标目录：`logos/resources/prd/3-technical-plan/1-architecture/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md

- Delta 文件：`logos/changes/sync-tech-plan-with-product-design/deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md

- Delta 文件：`logos/changes/sync-tech-plan-with-product-design/deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md

- Delta 文件：`logos/changes/sync-tech-plan-with-product-design/deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
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
   git add -A && git commit -m "docs(sync-tech-plan-with-product-design): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive sync-tech-plan-with-product-design`。
