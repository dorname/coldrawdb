# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — **原型先行**：renderListView 重设计为 PDManer 式字段明细编辑表格（行内编辑：字段代码/显示名称/PK/不为空/唯一/自增/数据类型/长度/小数位/说明；增删字段、上移/下移、按表分组头）+ 类型/长度/小数位分离配置
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — MODIFIED ListView 章节：只读分组清单 → 字段明细行内编辑网格（列定义、编辑落账、增删/排序、V1 边界：不做主题域树/多表 Tab 条/数据域列）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — MODIFIED §2.2：参数化类型口径（基类型 + 长度/小数位；type_ 存合成整串如 `VARCHAR(32)`；parse/compose 规则与默认值；向后兼容老整串）
- [x] 产出 delta 文件到 `deltas/test/core-SP-side-panel-test-cases.md` — MODIFIED ST-SP-LIST-01（网格定位+层叠断言保留，预期升级为编辑网格）+ ADDED ST-SP-LIST-02（行内编辑落账：改字段名/类型/长度/约束 → PUT 快照正确；增删字段；上移/下移）
- [x] 产出 delta 文件到 `deltas/test/core-UI-modals-2-test-cases.md` — MODIFIED ST-LV-01（分组编辑断言对齐新网格）+ ADDED UT-MM-37（parse_field_type/compose_field_type 纯函数：基类型/长度/小数位解析合成、老整串兼容、缺省值）
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — ADDED ST-CR-NOTE-01（便签拖动落账 + 文本输入不丢字）+ ST-CR-AREA-01（区域拖动落账）+ ST-CR-TAG-01（Inspector tag 受控输入连续键入不丢字）

## [code] 代码实现
- [x] 修改 `frontend-rs/src/editor_core.rs` — 新增 `parse_field_type` / `compose_field_type` 纯函数 + UT-MM-37 单测
- [x] 重写 `frontend-rs/src/editor_panels.rs::ListView` — PDManer 式字段明细编辑网格（行内编辑落账、增删字段、上移/下移、类型/长度/小数位列，沿用现有落账通路）
- [x] 修改 `frontend-rs/src/editor_panels.rs` Inspector 字段表单 — 类型下拉改基类型 + 长度/小数位输入（合成整串落账）；tag 输入改受控模式（本地 draft signal + 响应式 prop:value）
- [x] 修改 `frontend-rs/src/editor_render.rs` — DragState 补 note/area 变体 + pointermove/up 拖拽链路（仿表拖动三段式）；`on_update_area` 补 x/y 写回通道（`editor_panels.rs:8506`）
- [x] 高频写回治理 — 输入防抖写回、拖拽期只写视觉层（mouseup 落账）、paint rAF 合并、snapshot 移入 debounce 回调（消除卡死）
- [x] 改写 `frontend-rs/scripts/test-spec-parity-d.mjs` — ST-SP-LIST-01/02、ST-LV-01 对齐新网格；新增 ST-CR-NOTE-01/ST-CR-AREA-01/ST-CR-TAG-01 脚本；OpenLogos reporter 对齐用例 ID
