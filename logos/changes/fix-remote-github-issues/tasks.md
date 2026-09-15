# 实现任务

## [delta] 规格变更

### 批次 A — BUG
- [x] 产出 delta 文件到 `deltas/test/core-KB-shortcut-test-cases.md` — 登记关系创建进 undo 栈、Ctrl+Z 撤关系不删表、Ctrl+Y redo（对齐 #4）
- [x] 产出 delta 文件到 `deltas/test/core-PE-design-system-test-cases.md` — 登记主题 localStorage 持久化 / 刷新恢复（对齐 #6）
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §12 / compose：`PUBLIC_BASE_URL` 可配置注入说明（对齐 #1）
- [x] 产出 delta 文件到 `deltas/test/core-S04-test-cases.md` — 补充 compose/env 下 inviteUrl 基址约定断言说明（对齐 #1）

### 批次 B — FEATURE
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 画布聚焦 / 多表框选拖动（对齐 #2/#5）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 字段直连手势（对齐 #3）
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — UT-CR-FOCUS-* / UT-CR-MULTI-* / ST-CR-MULTI-01
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — UT-PB-08 / ST-PB-05

## [code] 代码实现

### 批次 A — BUG
- [x] #4：`on_create_reference`（及对称删除路径若缺）接入 `CommandStack::record/apply(AddReference|DeleteReference)`；Undo/Redo 按钮与 `KeyboardShortcuts` 支持 Ctrl+Y；`revert`/`execute` 失败时恢复栈，避免「空弹」
- [x] #4：补充 UT（AddReference 进栈；undo 只删关系不删表；Ctrl+Y redo）+ OpenLogos reporter
- [x] #6：主题切换写入 `localStorage["cdb-mode"]`；启动时优先读持久化值并写回 `data-mode`；补充 UT + reporter
- [x] #1：`docker-compose.yml` 支持/默认注入 `PUBLIC_BASE_URL`；必要时同步 `scripts/` 与部署文档已合并内容；补充脚本或后端回归断言 + reporter

### 批次 B — FEATURE
- [x] #2：实现 `focus_table`（新增表 / 搜索选中 / 列表点击）+ UT/ST + reporter
- [x] #3：字段→字段拖放建关系（无需先点关系工具）+ UT/ST + reporter
- [x] #5：框选多表后整体拖动位移 + UT/ST + reporter
