# 实现任务

> module: core | 提案：add-frontend-completeness
> 共 5 个批次，每批独立闭环。**B1 优先启动**，B2~B5 严格串行。
> 严禁在 [delta] section 写代码任务，严禁在 [code] section 写规格任务。

---

## [delta] 规格变更（B1~B5 共用前置）

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5 渲染策略补充「样式底座」段落（B1） — 117 行，§5.1/5.2/5.3/5.4 四子节，含 cdb- token 体系 + 验收要点
- [x] 产出 delta 到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型扩充 CSS 视觉（B1） — 312 行，MODIFIED style+body + ADDED 注释
- [x] 产出 delta 到 `deltas/implementation/core-implementation-checklist.md` — §2.3 修正 Areas/Enums 误标，补充未勾选项（B5 收尾时统一更新） — 125 行，36 个未实现项 + §13/14/15 跨批次跟踪

---

## [code] 批次 1 — 样式底座 + 顶部菜单骨架 + 撤销/重做 UI

**前置依赖**：无

**覆盖 skipped 用例**（执行前先列 ID — 取自 `logos/resources/test/core-S01-test-cases.md` §2）：
- **UT-S01-07** — 表创建触发 undo 栈（位置 `frontend-rs/src/editor_core.rs::CommandStack::apply`，验证 AddTable 命令 push 进 undo_stack）
- **UT-S01-08** — debounce 触发 save（位置 `frontend-rs/src/editor_data_access.rs::DebounceTrigger::schedule`，验证 1s 静默期后触发回调）

> 注：`core-S01-test-cases.md` §2 line 107/120 与 §7 附录 A line 273/274 在「UT-S01-07/08」标题描述上不一致（§2 说是 frontend undo/debounce，§7 附录说是 backend GET/revision）。
> 本 B1 严格按 §2 详细定义执行（frontend undo 栈 + debounce）。§7 附录的 backend 描述属于 baseline docs 内部矛盾，**B5 收尾时统一在 `deltas/test/core-S01-test-cases.md` 中修正附录**。

**实施步骤**：
- [x] 在 `frontend-rs/index.html` 引入 `styles.css`（link rel）
- [x] 创建 `frontend-rs/src/styles.css`：设计 token（颜色/间距/字号）+ 栅格 + `.cdb-*` 前缀规则
- [x] 修改 `frontend-rs/src/editor_panels.rs`：
  - 拆分 `TopBar` 为 `TopMenuBar`（4 下拉空壳）+ `Toolbar`（撤销/重做/Share/Export 按钮占位）
  - 新增 `UndoRedoButtons` 子组件，绑定 `EditorStore` + `CommandStack`（底座在 `editor_core.rs:208-310`）
  - 给所有现有组件 class 加 `cdb-` 前缀
- [x] UT-S01-07：编写 `editor_core::tests` 中 CommandStack::apply AddTable 单元测试（在 `frontend-rs/src/editor_core.rs` 末尾 `#[cfg(test)] mod tests`）
- [x] UT-S01-08：编写 `editor_core::tests` 中 DebounceTrigger 单元测试（wasm-bindgen-test 模拟时间）— B1 用 `#[ignore]` 占位，B5 接入 wasm-pack test 后取消 ignore
- [x] 写入 OpenLogos reporter 到 `logos/resources/verify/test-results.jsonl`（UT-S01-07 pass，UT-S01-08 skip）
- [x] `cd frontend-rs && trunk build --release false` 确认 wasm 体积不超 5MB（实测 2.3M ✅）
- [x] 立即 `cd <项目根> && openlogos verify` 确认不破坏现有 PASS 用例（Gate 3.6 PASS ✅，28/28 覆盖）

**B1 暂不引入新 ST 编号**（顶部菜单 4 下拉渲染、撤销按钮触发由 UT-S01-07/08 间接覆盖，verify 阶段如需追加 ST 编号再走规格变更）。

**回滚条件**：B1 完成后 `trunk build` 体积 > 5MB → 拆分样式为按需加载；B1 撤销/重做 UI 渲染但 Command 栈空 → 暂停 B2 排查。

---

## [code] 批次 2 — 5 个剩余侧栏 Tab

**前置依赖**：B1 完成（CSS 底座 + 顶部菜单）
**覆盖 skipped 用例**（修正：原 tasks.md 错误引用 UT-S02-03/04，core-S02-test-cases.md 实际是后端 API 测试；新编号来自 `core-04-side-panel-tabs.md` §11 + 本次 delta 追加的 `core-SP-side-panel-test-cases.md`）：
- **UT-SP-02**（已存在，§11 索引）— Tables Tab 搜索过滤
- **UT-SP-09**（本次 delta 新增）— 6 业务 Tab 切换
- **UT-SP-10**（本次 delta 新增）— 全局搜索跨 Tab 过滤
- **ST-SP-01**（已存在，§11 索引，B2 间接覆盖）— 端到端 5 表 + Issues Tab 0 error

**Delta 配套**（merge 时同步合并）：
- 新增 `logos/resources/test/core-SP-side-panel-test-cases.md`（详细定义 UT-SP-02/09/10 + ST-SP-01）
- 修改 `core-04-side-panel-tabs.md` §11（追加 UT-SP-09/10 索引行）

**实施步骤**：
- [x] 修改 `editor_panels.rs`：
  - 新增 `LeftPanel` 改造：tab 切换器（Tables / Areas / Enums / Notes / Relationships / Types / Issues）
  - 新增 7 个 Tab 子组件：`TablesTab`（从 LeftPanel 抽出）、`AreasTab`、`EnumsTab`、`NotesTab`、`RelationshipsTab`、`TypesTab`、`IssuesTab`
  - 搜索 + 类型筛选输入框（spec §10）
- [x] Areas/Enums/Notes/Types 暂用内存 state（spec 标记 V1 仅前端 state）
- [x] Relationships/Issues 用 store 中已有数据
- [x] 新增对应 UT（纯函数 + cargo test --lib 共 9 个 UT 通过）
- [x] 写入 OpenLogos reporter（UT-SP-02/09/10 pass，ST-SP-01 skip e2e 待 B5）
- [x] `openlogos verify` 确认 Gate 3.6 PASS（32 用例，12 通过，100% 覆盖）

**回滚条件**：B2 后侧栏 tab 切换导致 Tables Tab 原有功能 regression → 暂停 B3 修复。

---

## [code] 批次 3 — 画布渲染补全 + Issues Tab

**前置依赖**：B2 完成
**覆盖 skipped 用例**（修正：原 tasks.md 错误引用 UT-S01-09/10，core-S01-test-cases.md §2 line 134/147 实际是后端 API（自动保存重试 + 字段类型校验），不适合画布渲染；新编号来自 `core-01-editor-canvas.md` §5.3 + 本次 delta 追加的 `core-CR-canvas-test-cases.md`）：
- **UT-CR-01**（本次 delta 新增）— Areas 渲染（store 状态切换）
- **UT-CR-02**（本次 delta 新增）— Notes 渲染（store 状态切换）
- **UT-CR-03**（本次 delta 新增）— 端点 drag 改 start_field_id
- **UT-CR-04**（本次 delta 新增）— 端点 drag 改 end_field_id
- **UT-CR-05**（本次 delta 新增）— 端点 drag 不存在的 reference_id
- **ST-CR-01**（本次 delta 新增，e2e 待 B5）— references 贝塞尔连线在画布可见

**Delta 配套**（merge 时同步合并）：
- 新增 `logos/resources/test/core-CR-canvas-test-cases.md`（详细定义 UT-CR-01~05 + ST-CR-01）
- 修改 `core-01-editor-canvas.md` §5.3（追加 §5.3.1 测试 ID 索引）

**实施步骤**：
- [x] 修改 `editor_render.rs::leptos_canvas::Canvas`：将空 `areas`/`notes` 替换为 `store.areas.get()` / `store.notes.get()`（需在 `EditorStore` 新增这两个 signal）
- [x] `EditorStore` 新增 `areas: RwSignal<Vec<Area>>` / `notes: RwSignal<Vec<Note>>`（`editor_core.rs:119-126`）
- [x] `load` / `snapshot` 方法同步更新
- [x] Issues Tab 数据源：从 `references` 派生孤儿关系 + 字段类型未匹配 + 主键缺失
- [x] 画布交互增强：拖拽端点（references 端点拖拽改 start/end_field_id，pure function `update_reference_endpoint`）
- [x] 新增对应 UT + ST（UT-CR-01~05 Rust unit tests + ST-CR-01 B5 e2e skip）
- [x] 写入 OpenLogos reporter（5 pass + 1 skip → 17/38 通过 100% 覆盖）
- [x] `openlogos verify`（Gate 3.6 PASS，wasm 3.7M ≤ 5MB）

**回滚条件**：B3 后 `references` 渲染空 → 排查 `EditorStore.references` 是否被 `load` 正确注入。

---

## [code] 批次 4 — 4 个核心模态

**前置依赖**：B3 完成
**覆盖 skipped 用例**（frontend UI UT，spec `core-05-top-menu-modals.md` §7 已定义）：
- **UT-MM-01**（本次 delta 复用）— New 模态创建 diagram（validate_title + build_create_url）
- **UT-MM-04**（本次 delta 复用）— 模态背景点击关闭
- **UT-MM-05**（本次 delta 复用）— 模态 ESC 键关闭
- **UT-MM-06**（本次 delta 复用）— 必填字段失焦红框
- **UT-MM-07**（本次 delta 复用）— New 模态 title 为空 → OK 禁用
- **UT-MM-08**（本次 delta 复用）— Share 模态 URL 格式正确
- **UT-MM-09**（本次 delta 复用）— Open 模态 JSON 解析
- **ST-MM-01**（本次 delta 复用，e2e 待 B5）— 端到端菜单/模态/工具栏/快捷键全链路

**注意**：
- `ST-S02-01` / `ST-S02-02` / `ST-S02-03` 是 backend `core-S02-test-cases.md` 中的 API 端到端用例，**不在前端 B4 范围**（需要后端进程 + 真实 HTTP 调用，前端 wasm-pack 测不到）
- B4 用 UT-MM-01~09（前端纯函数 UT 可测）覆盖 4 个模态的输入校验 + URL 生成 + JSON 解析
- ST-MM-01 在 B5 接入 wasm-pack test harness 后跑

**Delta 配套**（merge 时同步合并）：
- 新增 `logos/resources/test/core-UI-modals-test-cases.md`（详细定义 UT-MM-01/04/05/06/07/08/09 + ST-MM-01）
- 修改 `core-05-top-menu-modals.md` 追加 §9.1 B4 测试 ID 索引

**实施步骤**：
- [ ] 在 `editor_panels.rs` 新增 `ModalRoot` 子模块：通用遮罩 + ESC 关闭 + 背景点击关闭
- [ ] 新增 4 个模态组件：`NewModal`（输入名称 + 创建）、`OpenModal`（文件选择 + 上传 .json）、`ShareModal`（生成 `/editor?share=ID` URL + 复制按钮）、`RenameModal`（重命名 diagram）
- [ ] 顶部菜单 File 下拉的对应项接通 `ModalRoot` 的 show/hide signal
- [ ] 调通 `editor_data_access::create` / `get`（已在 120/85 行）
- [ ] 新增对应 UT + ST
- [ ] 写入 OpenLogos reporter
- [ ] `openlogos verify`

**回滚条件**：4 个模态任一导致顶部菜单 regression → 暂停 B5。

---

## [code] 批次 5 — 剩余 5 模态 + 快捷键 + 搜索筛选 + 清单修正

**前置依赖**：B4 完成
**覆盖 skipped 用例**：
- ST-S02-04（Import 模态 SQL 解析）
- ST-S02-05（Language 模态切换 i18n）
- 新增 ST-UI-05（Ctrl+Z / Ctrl+Shift+Z 键盘快捷键）

**实施步骤**：
- [ ] 新增 5 个模态组件：`ImportModal`（粘贴 SQL）、`ImportSourceModal`（选 backend 源）、`LanguageModal`（zh/en 切换，V1 提示 toast）、`SetTableWidthModal`（输入新宽度）、`ConfigureCustomTypesModal`（增删自定义类型）
- [ ] 顶部菜单接通剩下 5 个菜单项
- [ ] 全局键盘事件监听：Ctrl/Cmd+Z、Ctrl/Cmd+Shift+Z、Delete、Ctrl/Cmd+D、Space（pan）、Ctrl/Cmd+S（force save）
- [ ] 侧栏全局搜索框接通 store（spec §10）
- [ ] 修正 `logos/resources/implementation/core-implementation-checklist.md`：§2.3 Areas/Enums 由 `[x]` 改 `[ ]`；§2.4/2.5/2.6 等补勾选
- [ ] 新增对应 UT + ST
- [ ] 写入 OpenLogos reporter
- [ ] `openlogos verify`（目标：skipped 减少到 0，passed 提升到 ≥25/28）

**回滚条件**：5 个模态任一阻塞 UI → 拆 B5 为 B5a（5 模态）+ B5b（快捷键 + 清单修正）顺序执行。

---

## [deploy] 部署任务

> ⚠️ 人类确认点：仅在 B1~B5 全部完成 + `openlogos verify` PASS 后执行。

- [ ] `cd frontend-rs && trunk build --release` 生成生产 dist
- [ ] 用 `dist/` 内容替换 staging 服务器前端静态目录
- [ ] 确认 `nginx` 重新加载生效
- [ ] **人类确认** 后运行 `openlogos smoke`：浏览器 e2e 验证 5 大 happy path
- [ ] smoke 失败 → `git revert` 整个 add-frontend-completeness 提交链，恢复到 `0375339`

---

## 部署决策一致性自检

| 检查项 | 状态 |
|---|---|
| `proposal.md` 声明"是否需要部署：是" | ✅ |
| `tasks.md` 存在 `[deploy]` section | ✅ |
| `proposal.md` 声明"是否需要 smoke：是" | ✅ |
| `[deploy]` section 存在 | ✅ |
| `[code]` section 未混部署命令 | ✅（仅有 build 命令，部署执行在 `[deploy]`） |
