# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — 创建房间流程新增引擎前置选择（选「新建空白模型」时露出引擎下拉）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型回写：列表视图、AppBar 引擎下拉、创建房间/新图引擎选择
- [x] 产出 delta 文件到 `deltas/test/core-S04-test-cases.md` — 新增 ST：创建房间选引擎 → 落账 + Inspector 类型过滤
- [x] 产出 delta 文件到 `deltas/test/core-SP-side-panel-test-cases.md` — 新增 ST：列表视图全屏渲染（有表展示清单 / 空表不塌陷）

## [code] 代码实现
- [x] 修复 `frontend-rs/src/styles.css` — 补 `.cdb-list-view-panel` 样式（grid-row: 2、min-height:0、overflow:auto、纵向 flex）
- [x] 修改 `frontend-rs/src/editor_data_access.rs` — `DiagramClient::create` 增加 database 形参并透传
- [x] 修改 `frontend-rs/src/editor_panels.rs` — 房间创建模态选「新建空白模型」时露出引擎下拉并传入创建
- [x] 修改 `frontend-rs/src/modals.rs` — ~~新建图模态加引擎下拉~~（已撤销：新建图模态为死代码路径，ST-S04-UI-10 删除，见 proposal 范围修正）
- [x] 编写对应 UT/ST 测试代码 + OpenLogos reporter（对齐上述 delta 用例 ID）
