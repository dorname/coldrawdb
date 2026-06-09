# Delta — core-01-editor-canvas.md
# 模块：core | 提案：add-frontend-completeness
# 路径：`logos/changes/add-frontend-completeness/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
# 对齐参考源：tasks.md B3「画布渲染补全」+ 新建 `core-CR-canvas-test-cases.md`

## MODIFIED — §5.3 追加测试 ID 索引段落

> **追加** 到主文档 `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
> §5.3「Areas / Notes / References 渲染」段落末尾。merge 时在 §5.3 后插入 §5.3.1 子节，
> 描述测试 ID 索引 + 引用 `core-CR-canvas-test-cases.md`。

### 5.3.1 测试 ID 索引（B3 范围）

| TC ID | 描述 | 对齐实现 |
|---|---|---|
| UT-CR-01 | Areas 渲染（store 状态切换 + draw_area 接收 &\[Area\]） | `editor_core.rs::EditorStore` |
| UT-CR-02 | Notes 渲染（store 状态切换 + draw_note 接收 &\[Note\]） | `editor_core.rs::EditorStore` |
| UT-CR-03 | 端点 drag 改 start_field_id（pure function） | `editor_render.rs::update_reference_endpoint` |
| UT-CR-04 | 端点 drag 改 end_field_id | `editor_render.rs::update_reference_endpoint` |
| UT-CR-05 | 端点 drag 不存在的 reference_id（no-op） | `editor_render.rs::update_reference_endpoint` |
| ST-CR-01 | references 贝塞尔连线在画布可见（e2e） | `frontend-rs/tests/wasm/cr.rs` |

> **详细定义** 见 `logos/resources/test/core-CR-canvas-test-cases.md`（与本 delta 同步新增）。
> 修正 tasks.md B3 覆盖说明：删去错误引用的 UT-S01-09/10，替换为 UT-CR-01~05 + ST-CR-01。
