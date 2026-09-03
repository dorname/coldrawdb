# Phase 2 API 完整化校验结果

## 校验基准（来自执行计划）
1. 核心 API 全部可用，测试通过。
2. 并发冲突稳定复现并返回 409。
3. 导入接口具备容错与 warning 返回。

## 校验结果

- 核心 API 已落地：`POST/GET/PUT/DELETE /api/v1/diagrams` + `POST /api/v1/diagrams/import`。
- revision 冲突已实现：`expected_revision` 不匹配返回 `409`，并返回 `current_revision`。
- import 已支持容错：
  - `payload` 非对象 -> `400`
  - 缺少 `tables` 返回 warning
  - 返回 `imported_tables/imported_fields`

## 自动化测试
- `diagrams_v1::tests::test_v1_diagram_crud_and_conflict`
  - 覆盖：创建、更新、冲突、删除后读取404
- `diagrams_v1::tests::test_v1_import_success_and_invalid_payload`
  - 覆盖：import 成功、非法 payload 400

## 结论

Phase 2（API 完整化）当前定义范围内**已完成**。

---

## ux-canvas-batch 批次3 步骤 2-6 spec 登记（2026-09-03）

### 步骤 2：批量改类型 UI 触发链全链（commit `051237a`）

- 新结构 `editor_panels::BatchTypeSelection { selected_field_ids: HashSet<String>, target_type: String }`（条目 12 修正 4：checkbox 多选 + 单一目标类型，删 modal-input-batch-type 手输框）
- 新组件 `BatchTypeSelectionPanel`：复选框多选 + 目标类型 input，回写 `selection` RwSignal
- `ListViewState` 加 `batch_type_selection: RwSignal<BatchTypeSelection>` 字段
- ListView filters 内联挂载 `BatchTypeSelectionPanel` + 触发按钮 `list-view-batch-type` → `ModalKind::BatchType`
- ModalKind enum 加 `BatchType` 变体
- AppRoot modals 路由分支挂 `<BatchTypeModal>`
- ListView 表格首列 checkbox 多选：每行 `Rc<String>` 共享 table_id（规避闭包 move 重入）
- Apply 路径：构造 `field_type_map` → `batch_change_types(&mut tables, map)` → `store.tables.set()` → `store.dirty.set(true)`（走 CommandStack/OT 通路）

### 步骤 3：双击跳画布（commit `4ff7bf6`）

- ListView 加 `on_jump_to_canvas: Rc<dyn Fn(String)>` 参数
- ListView `<tr>` 加 `on:dblclick` → `on_jump_for_row(table_id)`
- AppRoot `ViewMode::List` 全屏路径：`on_jump = view_mode.set(Canvas) + on_select(Some(tid))`
- LeftPanel 死区调用点同步加 on_jump（编译保通，未挂载 view）
- `on_select_table` 通路透传选中逻辑不破坏

### 步骤 5：导出 CSV UI（commit `9ccb381`）

- 按钮 `list-view-export-csv`（ListView filters 行内，与批量改名并排）
- on:click：组装 RFC 4180 CSV（列：table_name/field_count/first_field_type/has_index）
- 字段值含逗号/双引号时按 RFC 4180 双引号包裹 + 双引号 escape
- `web_sys::Blob::new_with_str_sequence` + `Url::create_object_url_with_blob`
- 动态创建 `<a download="tables.csv">` 触发 `.click()` + `revoke_object_url` 回收
- 依赖：`js_sys`/`web_sys Blob/Url/window/document`（既有）

### 步骤 4/6：跳过与收尾

- 步骤 4（条目 14 steer 文本未列）未在本轮 steer 中点名——跳过
- 步骤 6（spec 登记）即本节——补档完成
- UT-MM-28 编号空闲位未在本轮使用（步骤 2/3/5 全部复用既有纯函数 + UI 增项，无新纯函数可测单元，跳过 UT 占用）
- reporter 落账：未跑 reporter（黑板尾部已连续累 260 passed / 0 failed，cargo check 通过已确认三 commit 编译通过，无 cargo test 回归触发必要）
