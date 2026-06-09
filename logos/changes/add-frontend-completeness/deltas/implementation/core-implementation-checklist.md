# Delta — core-implementation-checklist.md
# 模块：core | 提案：add-frontend-completeness
# 路径：logos/changes/add-frontend-completeness/deltas/implementation/core-implementation-checklist.md
# 对齐参考源：add-frontend-completeness/tasks.md B1~B5 + 实测 editor_panels.rs / editor_render.rs

## MODIFIED — §2.3 editor_panels（修正误标 + 补完未勾选项）

> **替换** 主文档 `logos/resources/implementation/core-implementation-checklist.md` 中
> `### 2.3 editor_panels` 整段。
> 修正理由：原 §2.3 将 Areas/Enums/Notes/Relationships/Types/Issues/DBMLEditor/搜索筛选全标 [x]，
> 但实测 `frontend-rs/src/editor_panels.rs`（379 行）仅含 TopBar/LeftPanel(Tables only)/RightPanel 三个组件，
> 顶部菜单 / 9 模态 / 撤销 UI / 搜索 / 5 个侧栏 Tab 全部缺失。
> merge 时按本节内容覆盖原 §2.3 整段。

### 2.3 editor_panels

**已实现（V1 Phase 4 MVP）**：

- [x] TopBar 基础（建表 + 保存按钮 + revision 显示）
- [x] LeftPanel — Tables Tab + 列表项 + 选中高亮
- [x] RightPanel — 字段增删改（仅 fields 维度的 type / default / check / primary 等属性）
- [x] ConflictDialog 基础（强制覆盖 / 重新加载二选一）
- [x] ErrorToast 基础

**未实现（V1 add-frontend-completeness 补全）**：

- [ ] **样式系统**（B1）：`frontend-rs/src/styles.css` 落地 cdb- 设计 token
- [ ] **TopMenuBar**（B1）：4 个下拉（File / Edit / View / Help）骨架 + 占位菜单项
- [ ] **Toolbar**（B1）：Undo / Redo 按钮（绑定 CommandStack）+ Share / Export 占位
- [ ] **UndoRedoButtons**（B1）：底座在 `editor_core::CommandStack`（line 208-310），UI 未暴露
- [ ] **LeftPanel 改造**（B2）：6+1 Tab 切换器（Tables / Areas / Enums / Notes / Relationships / Types / Issues）
- [ ] **AreasTab**（B2）：Area 列表 + 增删改
- [ ] **EnumsTab**（B2）：Enum 列表 + 增删改（V1 仅前端 state）
- [ ] **NotesTab**（B2）：Note 列表 + 增删改
- [ ] **RelationshipsTab**（B2）：Reference 列表 + 增删改
- [ ] **TypesTab**（B2）：CustomType 列表 + 增删改（V1 仅前端 state）
- [ ] **IssuesTab**（B3）：孤儿关系 / 类型未匹配 / 主键缺失 校验
- [ ] **DBMLEditor 备选视图**（B3）：DBML 文本编辑入口
- [ ] **全局搜索**（B5）：侧栏顶部搜索框接通 store（spec §10）
- [ ] **类型筛选**（B5）：按字段类型 filter（spec §10）
- [ ] **ModalRoot**（B4）：通用遮罩 + ESC 关闭 + 背景点击关闭
- [ ] **NewModal**（B4）：输入名称 + 调 `editor_data_access::create`
- [ ] **OpenModal**（B4）：文件选择 + 上传 .json 调 `/api/v1/diagrams/import`
- [ ] **ShareModal**（B4）：生成 `/editor?share={id}` URL + 复制按钮
- [ ] **RenameModal**（B4）：重命名 diagram（PUT 时一并提交）
- [ ] **ImportModal**（B5）：粘贴 SQL + 调 bridge 端点
- [ ] **ImportSourceModal**（B5）：选 backend 桥接源
- [ ] **LanguageModal**（B5）：zh / en 切换（V1 提示 toast）
- [ ] **SetTableWidthModal**（B5）：输入新宽度
- [ ] **ConfigureCustomTypesModal**（B5）：自定义类型增删
- [ ] **键盘快捷键**（B5）：Ctrl+Z / Ctrl+Shift+Z / Delete / Ctrl+D / Space / Ctrl+S
- [ ] **画布拖拽端点**（B3）：references 端点拖拽改 start/end_field_id
- [ ] **画布框选**（B5）：鼠标拖空白框选多个对象
- [ ] **画布右键菜单**（B5）：编辑 / 删除 / 复制上下文菜单

**V2 待实现（本提案外）**：

- [ ] 房间成员列表（V2 协作 Tab，add-v2-collab-spec 提案）

## MODIFIED — §2.4 editor_render（补充 B3 待办）

> **替换** 主文档 `logos/resources/implementation/core-implementation-checklist.md` 中
> `### 2.4 editor_render` 整段。
> 修正理由：draw_area / draw_note 函数确实在 `editor_render.rs`（line 409/431），但 Canvas 组件
> `create_effect` 传空 `areas: Vec::new>()` / `notes: Vec::new>()`（line 114-116），所以视觉上无 areas/notes。
> 同样 references 贝塞尔线存在但 EditorStore.references 已在 store 中（B3 需要 wire up）。
> merge 时按本节内容覆盖原 §2.4 整段。

### 2.4 editor_render

**已实现**：

- [x] Canvas 容器 + 鼠标事件（pointer/wheel）+ Transform 状态（pan_x / pan_y / zoom）
- [x] draw_table 函数（line 304）
- [x] draw_field_row 渲染逻辑（Table 内部）
- [x] draw_bezier 函数（line 367）— references 贝塞尔连线底层已实现
- [x] draw_arrow_head 函数（line 390）
- [x] draw_area 函数（line 409）— 已实现但未接入 Canvas
- [x] draw_note 函数（line 431）— 已实现但未接入 Canvas
- [x] draw_grid 网格背景（line 282）
- [x] hit_test 函数（line 471）
- [x] round_rect / round_rect_top 工具函数

**未实现（V1 add-frontend-completeness 补全）**：

- [ ] **Canvas 组件接入 areas/notes store**（B3）：`leptos_canvas::Canvas::create_effect` 传空 `Vec::new>()`，需改为 `store.areas.get()` / `store.notes.get()`
- [ ] **EditorStore 新增 areas / notes signals**（B3）：`editor_core::EditorStore`（line 119-126）目前无 `areas` / `notes`，需新增 RwSignal
- [ ] **load / snapshot 同步**（B3）：`EditorStore::load` / `EditorStore::snapshot` 同步处理新信号
- [ ] **选中高亮闪烁**（B3）：Relationships Tab 单击 → 画布对应线闪烁
- [ ] **画布拖拽表**（B3）：拖表移动改 table.x / table.y
- [ ] **缩放中心切换**（B3）：spec §4 切换"鼠标位置 / 画布中心"

**V2 待实现**：

- [ ] 协作者光标渲染（V2）

## ADDED — §13 跨批次代码 / 测试对照表

> **新增** 一节，给 reviewer 跟踪 B1~B5 推进状态。merge 时追加到主文档末尾（在 §12 之后）。

### 13. 跨批次代码 / 测试对照（add-frontend-completeness）

| 批次 | 业务代码位置 | UT 用例 ID | ST 用例 ID | 提交时点 |
|---|---|---|---|---|
| B1 | `editor_panels.rs`（TopBar 拆分）+ 新增 `styles.css` | UT-S01-07、UT-S01-08 | ST-UI-01 | spec merge 后第 1 批 |
| B2 | `editor_panels.rs`（LeftPanel 改造 + 6 Tab 子组件） | UT-S02-03、UT-S02-04 | ST-UI-02 | B1 完成后 |
| B3 | `editor_render.rs::Canvas` + `editor_core::EditorStore` | UT-S01-09、UT-S01-10 | ST-UI-03 | B2 完成后 |
| B4 | `editor_panels.rs::ModalRoot` + 4 模态子组件 | — | ST-S02-01、ST-S02-02、ST-S02-03、ST-UI-04 | B3 完成后 |
| B5 | `editor_panels.rs`（5 模态 + 快捷键）+ 清单修正 | — | ST-S02-04、ST-S02-05、ST-UI-05 | B4 完成后 |

### 14. 关键不变量

- **核心 5 happy path**（建表 / 加字段 / 设关系 / 改类型 / 保存）在 B1~B5 全程必须 PASS
- **OpenLogos reporter** 必须在 `logos/resources/verify/test-results.jsonl` 持续累计
- **wasm 体积** ≤ 5MB（trunk build release 后验证）
- **样式隔离**：仅 1 个 styles.css、0 个内联 style、所有 class 带 cdb- 前缀
- **store 单一来源**：`EditorStore` 是唯一可变状态；panels/render 只通过 RwSignal 读写

### 15. 对齐参考源（add-frontend-completeness）

- `logos/changes/add-frontend-completeness/proposal.md`
- `logos/changes/add-frontend-completeness/tasks.md`
- `logos/changes/add-frontend-completeness/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（§5.2 样式底座）
- `logos/changes/add-frontend-completeness/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`（B1 视觉基线）
- `drawdb` §2.1 `Workspace.jsx` / §2.2 顶部菜单 / §2.3 编辑器画布 / §2.4 侧边栏
