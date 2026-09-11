# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/api/` — rooms.yaml 新增回收站三端点（archived 列表 / restore / permanent）
- [x] 产出 delta 文件到 `deltas/api/` — bridge.yaml 新增 `POST /api/v1/bridge/export/execute`
- [x] **验证 API YAML** — delta 中所有 YAML 片段符合 OpenAPI 3.x 规范（含 `:` 或特殊字符的 description/summary 值用双引号包裹）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-S04-room-lifecycle-design.md 扩展生命周期状态机 + 回收站交互 + 删除文案修正
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-01d-import-export.md 新增 ExportDrawer「导出到数据库」区块 + 下拉样式条款
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-03-bridge-io.md 新增 §14 导出执行端点
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-09-core-components.md 新增 select 控件视觉条款
- [x] 产出 delta 文件到 `deltas/test/` — core-S04-test-cases.md 新增回收站 UT/e2e 用例
- [x] 产出 delta 文件到 `deltas/test/` — core-PC-import-export-test-cases.md 新增导出执行 UT/e2e 用例

## [code] 代码实现

批次一：房间回收站（覆盖 UT-S04-11~14 + e2e 锚点）
- [x] 后端：`list_archived_rooms` / `restore_room`（409 冲突）/ `permanent_delete_room` + 路由与 handler
- [x] 后端：回收站 UT（恢复 / 硬删除 / 权限 / 冲突）
- [x] 前端：房间列表页回收站入口 + 回收站视图（恢复 / 彻底删除二次确认）+ DeleteRoomModal 文案修正
- [x] 前端：回收站锚点 UT + e2e 用例

批次二：下拉列表 UI 修复（覆盖样式锚点用例）
- [x] styles.css：`.cdb-form-select` / `.cdb-select` 重做（appearance、chevron、暗色适配、防裁切）

批次三：连接数据库导出执行（覆盖 UT-PC-16~19 + ST-PC-05）
- [x] 后端：`POST /api/v1/bridge/export/execute`（SQLite/PG 执行 DDL + 错误映射）
- [x] 后端：导出执行 UT（SQLite 真实库 + 嵌入式 PG + 错误映射）
- [x] 前端：ExportDrawer「导出到数据库」区块 + 结果反馈
- [x] 前端：锚点 UT + e2e 用例（ST-PC-05）

批次四：verify 回归修复（首轮 verify pre-run 翻出 2 个由本变更时序/级联偏移引发的既有隐患）
- [x] ListView 双击跳画布竞态（ST-SP-LIST-01）：`cdb-list-view-body` 移出 `store.tables` 动态块，改静态节点 + display 绑定，行节点跨保存写回稳定（dblclick 不再因 DOM 重建丢失）
- [x] 溢出菜单右锚定失效（ST-PC-03）：`.cdb-app-bar__overflow-menu` 的 `left: auto` 被同优先级靠后的 `.cdb-menu-dropdown { left: 0 }` 覆盖，菜单误左锚定；提升选择器优先级修复

每批均含：业务代码 + UT/ST 测试代码 + 写入 `logos/resources/verify/test-results.jsonl` 的 OpenLogos reporter
