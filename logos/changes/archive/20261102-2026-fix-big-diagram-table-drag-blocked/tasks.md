# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 修订 R-HIT-01/R-HIT-05，明确有效几何同源、编辑态表优先、显式关系工具与只读模式的优先级
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 补充 §5.18 密集关系下直接拖表与工具回落口径，新增 §5.19 悬浮控件指针边界
- [x] 产出 `deltas/test/core-PB-relationship-test-cases.md` — 修订 UT-PB-28/31 的只读适用范围，新增 UT-PB-37 与 ST-PB-21

## [code] 代码实现

- [x] 修改 `frontend-rs/src/editor_render.rs`：可编辑默认/框选路径命中表时优先表；关系命中及 AABB 预过滤接收当前 zoom 与 label_font_scale，点击/悬停统一使用完整参数路径
- [x] 修改 `frontend-rs/src/styles.css`：悬浮条容器非按钮区域指针穿透，实际按钮保持指针可点
- [x] 在 `frontend-rs/src/editor_render.rs` 交付 UT-PB-37，覆盖 bezier/orthogonal/straight 在低缩放与放大字号下的同源命中，并保留既有包装函数兼容测试
- [x] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付 54 表 / 97 关系的 ST-PB-21：默认工具拖表、显式关系选择、悬浮条空白穿透与按钮可点
- [x] 更新 `frontend-rs/tests/openlogos_reporter.rs` 及对应 reporter 登记，确保 UT-PB-37、ST-PB-21 写入 `logos/resources/verify/test-results.jsonl`
