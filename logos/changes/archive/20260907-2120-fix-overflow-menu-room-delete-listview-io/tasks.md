# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 更多菜单视口边界保护（max-height + 滚动 + 右缘夹取）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — 房间列表页卡片删除入口（owner 可见 + 确认模态 + 删除后列表刷新）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — IO 抽屉入口扩展至列表视图（复用本地合并管道）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — ListView 工具栏导入/导出按钮
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — IA 同步（房间卡片删除操作 + ListView IO 入口）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型对齐：菜单边界保护 + 房间卡片删除 + ListView IO 按钮
- [x] 产出 delta 文件到 `deltas/test/core-S04-test-cases.md` — 新增房间列表页删除 e2e/UT 用例（ST-S04-UI-13 / UT-S04-UI-13）
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — 新增 ListView IO 入口 + 菜单边界保护用例（ST-PC-02 / ST-PC-03 / UT-PC-10）
- [x] 产出 delta 文件到 `deltas/test/core-PV-verify-pre-run-test-cases.md` — UT-S04-UI-13 标题式用例登记进兼容索引表（防 verify「未登记 reporter ID」）

## [code] 代码实现
- [x] `frontend-rs/src/styles.css`：`.cdb-app-bar__overflow-menu` 加 max-height + overflow-y + 视口右缘保护
- [x] `frontend-rs/src/editor_panels.rs`：rooms-list-page 房间卡片删除入口（owner 可见）+ 确认模态 + 删除后列表刷新
- [x] `frontend-rs/src/editor_panels.rs`：ListView 工具栏加导入/导出按钮，接线 IoDrawer（复用 on_merge_import 合并管道）；IoDrawer 上移为 .cdb-app 直接子级以支持 ListView 态叠加渲染
- [x] 编写/更新 UT 与 e2e 对齐代码，含 OpenLogos reporter 写入（UT-PC-10 / UT-S04-UI-13 锚点测试 + reporter 登记；ST-PC-02 / ST-PC-03 / ST-S04-UI-13 e2e 落地 D 批）
