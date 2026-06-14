## ADDED — 顶部菜单 + 模态规格

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
> 对齐参考源：drawdb §2.2 顶部菜单 + 9 个模态（New/Open/Import/ImportSource/Language/SetTableWidth/Share/Rename/ConfigureCustomTypes）

# 顶部菜单 + 模态规格（V1）

## 1. 顶部菜单布局

```
+------------------------------------------------------------+
| [Logo] [File▼] [Edit▼] [View▼] [Help▼]      [SaveState] [⚙]|
+------------------------------------------------------------+
| [↶][↷] [Title editor]    [revision: 5]  [Share] [Export▼] |
+------------------------------------------------------------+
```

- **Logo**：左上角 drawdb → coldrawdb 重命名（V1）
- **菜单**：4 个下拉（File / Edit / View / Help）
- **工具栏**：撤销 / 重做 / 标题编辑 / revision 状态 / Share / Export
- **右侧**：SaveState 指示器 + 设置图标

## 2. 菜单项

### 2.1 File

| 项 | 快捷键 | 行为 |
|---|---|---|
| New | Ctrl/Cmd + N | 打开 New 模态 |
| Open | Ctrl/Cmd + O | 触发文件选择器，导入 `.json` |
| Save | Ctrl/Cmd + S | 立即保存（绕过 debounce） |
| Import | — | 打开 Import 模态 |
| Export | — | 打开 Export 模态 |
| Share | — | 打开 Share 模态 |
| Rename | — | 打开 Rename 模态 |
| Delete | — | 删除当前 diagram（确认） |

### 2.2 Edit

| 项 | 快捷键 | 行为 |
|---|---|---|
| Undo | Ctrl/Cmd + Z | 撤销栈弹一步 |
| Redo | Ctrl/Cmd + Shift + Z | 重做栈弹一步 |
| Cut | Ctrl/Cmd + X | 剪切选中对象 |
| Copy | Ctrl/Cmd + C | 复制选中对象 |
| Paste | Ctrl/Cmd + V | 粘贴 |
| Duplicate | Ctrl/Cmd + D | 复制选中 |
| Select All | Ctrl/Cmd + A | 全选画布对象 |
| Find | Ctrl/Cmd + F | 聚焦搜索框 |

### 2.3 View

| 项 | 行为 |
|---|---|
| Zoom In | 画布放大（步进 0.25x） |
| Zoom Out | 画布缩小 |
| Zoom Reset | 重置为 1x |
| Show Grid | 切换网格显示（drawdb 行为；V1 可选） |
| DBML Editor | 切换到 DBML 视图（详见 core-04 §9） |
| Settings | 打开设置页 |

### 2.4 Help

| 项 | 行为 |
|---|---|
| About | 打开 About 模态 |
| Shortcuts | 快捷键速查 |
| Report Bug | 跳转 drawdb issue 页面（V1 复用 drawdb 链接） |

## 3. 9 个模态清单

| 模态 | 用途 | 字段 |
|---|---|---|
| New | 新建 diagram | title（必填） |
| Open | 打开已有 diagram | diagram id（URL 输入） |
| Import | 导入文件 | file（拖拽 / 选择） + format（SQL/DBML/JSON） |
| ImportSource | 选择导入源 | local / remote（V1 仅 local） |
| Language | 切换 UI 语言 | en / zh（V1 双语） |
| SetTableWidth | 批量设置表宽 | target width（0 = auto） |
| Share | 生成分享链接 | visibility（public/private，V1 实际无差别） |
| Rename | 重命名 diagram | title |
| ConfigureCustomTypes | 管理自定义类型 | 列表 + 增删改 |

## 4. 模态通用模式

### 4.1 打开 / 关闭

- 打开：从菜单 / 工具栏触发
- 关闭：右上角 × / ESC / 背景点击
- **遮罩生命周期**：`<div class="cdb-modal-overlay">`（即"背景"）**仅在 `kind.get().is_some()` 时存在**。模态关闭（`kind` 回到 `None`）时遮罩必须从 DOM 移除，否则遮罩会持续拦截全屏 pointer events，阻挡非模态 UI 的所有点击（HP-01~HP-05 回归验收点）
- 取消前若有未保存修改 → 弹确认

### 4.2 布局

```
+--------------------------------------+
| [Title]                    [×]       |
+--------------------------------------+
|                                      |
|         Form fields                   |
|                                      |
+--------------------------------------+
|                          [Cancel][OK]|
+--------------------------------------+
```

### 4.3 校验

- 必填字段失焦时红框 + 提示
- OK 按钮在表单未通过校验时禁用
- 校验规则：与对应实体对象一致（如 diagram.title 非空 + 长度 ≤ 64）

## 5. 模态详细规格

### 5.1 New 模态

- 字段：`title`（text）
- OK：POST `/api/v1/diagrams` → 跳转到 `/editor/{id}`

### 5.2 Open 模态

- 字段：`diagram_id`（text，UUID 格式）
- OK：跳转 `/editor/{id}`

### 5.3 Import 模态

- 字段：`file`（file input）+ `format`（radio: SQL/DBML/JSON）
- 拖拽支持：拖文件到模态区域
- 大小限制：5 MB（来自 bridge config）
- OK：调用 `POST /api/v1/bridge/import/local` → 完成后跳转到新 diagram

### 5.4 ImportSource 模态

- 字段：`source`（radio: local / remote）
- V1 仅 local 实际生效；remote 选项**预留 UI**，后端待 V2

### 5.5 Language 模态

- 字段：`language`（radio: en / zh）
- 立即生效（不需 OK 按钮）
- 持久化到 `localStorage`

### 5.6 SetTableWidth 模态

- 字段：`width`（number，0 = auto）
- 应用：遍历所有 table，更新 `width`
- 立即生效，触发自动保存

### 5.7 Share 模态

- 字段：`share_link`（text，read-only）+ Copy 按钮
- 链接格式：`/editor/{id}`（V1 无权限控制，所有人可访问）
- 复制后按钮文案变 "Copied!" 2 秒

### 5.8 Rename 模态

- 字段：`title`（text）
- OK：PUT `/api/v1/diagrams/{id}`（仅 title 字段）

### 5.9 ConfigureCustomTypes 模态

- 列表：所有自定义类型
- 操作：增 / 删 / 改（详见 core-01c §3.2）
- 关闭：自动保存（V1 仅前端 state；reload 后丢失）— ⚠️ V1 限制

## 6. 工具栏组件

### 6.1 撤销 / 重做栈深度指示

- 撤销栈：`[撤销步数 / 总步数]`
- 例：撤销 3 步后显示 `3/50`

### 6.2 标题编辑器

- 双击 diagram 标题 → 文本输入框
- 失焦或回车 → 触发保存

### 6.3 revision 状态

- 显示 `rev: 5` 标签
- 鼠标悬停 → tooltip 显示时间

### 6.4 SaveState 指示器

- `Saved`（绿）/ `Saving...`（黄）/ `Error`（红）/ `Idle`（灰）
- 鼠标悬停 → 显示最后保存时间

## 7. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-MM-01 | File → New → 填写 title → OK → 创建 diagram |
| UT-MM-02 | Edit → Undo → 撤销栈 -1 |
| UT-MM-03 | View → Zoom In → 画布放大 0.25x |
| UT-MM-04 | 模态背景点击 → 关闭 |
| UT-MM-05 | 模态 ESC → 关闭 |
| UT-MM-06 | 必填字段失焦 → 红框 |
| UT-MM-07 | New 模态 title 为空 → OK 禁用 |
| UT-MM-08 | Share 模态 Copy 按钮 → 剪贴板内容正确 |
| UT-MM-09 | ConfigureCustomTypes 关闭 → 自定义类型保留（仅当前 session） |
| ST-MM-01 | 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路操作 |

## 8. V1 边界

- ❌ Remote Import（V1 仅 local）
- ❌ 自定义快捷键（V1 硬编码 drawdb 默认）
- ❌ 自定义 UI 主题（V1 仅 light）
- ❌ ConfigureCustomTypes 跨刷新保留（V1 仅前端 session state）
- ❌ Share 链接权限控制（V1 公开访问，V2 计划私有房间）
- ❌ 多语言扩展（V1 仅 en / zh）

## 9. 对齐参考源

- drawdb `src/components/EditorHeader/`
- drawdb `src/components/Modals/`
- drawdb `src/components/Modals/Share/`
- drawdb `src/components/Modals/ConfigureCustomTypes/`
- drawdb `src/components/Modals/Import/`
- coldrawdb `frontend-rs/src/editor_panels.rs`（标题编辑器等）
- `docs/drawdb-capability-checklist.md` §2.2
## ADDED — §9.1 B4 测试 ID 索引（提案：add-frontend-completeness）

> 模块：core | 提案：add-frontend-completeness
> 路径：deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md
> 对齐参考源：`core-05-top-menu-modals.md` §7 + `test/core-UI-modals-test-cases.md`

# B4 模态补全 — 测试 ID 索引

## 1. 范围

B4 在 §3 的 9 个模态清单中，**仅实现 4 个核心模态**：
- New（§5.1）
- Open（§5.2）
- Share（§5.7）
- Rename（§5.8）

其余 5 个（Import / ImportSource / Language / SetTableWidth / ConfigureCustomTypes）放 B5。

## 2. 测试 ID 索引

| TC ID | 描述 | 对齐实现 | B4 状态 |
|---|---|---|---|
| UT-MM-01 | New 模态创建 diagram（validate_title + build_create_url） | `editor_panels.rs::modals::validate_title` | ✅ B4 实现 |
| UT-MM-04 | 模态背景点击关闭 | `editor_panels.rs::modals::ModalRoot` | ✅ B4 实现 |
| UT-MM-05 | 模态 ESC 键关闭 | `editor_panels.rs::modals::ModalRoot` | ✅ B4 实现 |
| UT-MM-06 | 必填字段失焦红框 | `editor_panels.rs::modals::{NewModal,RenameModal}` | ✅ B4 实现 |
| UT-MM-07 | New 模态 title 为空 → OK 禁用 | `editor_panels.rs::modals::NewModal` | ✅ B4 实现 |
| UT-MM-08 | Share 模态 URL 格式正确（build_share_url） | `editor_panels.rs::modals::build_share_url` | ✅ B4 实现 |
| UT-MM-09 | Open 模态 JSON 解析（parse_diagram_json） | `editor_panels.rs::modals::parse_diagram_json` | ✅ B4 实现 |
| ST-MM-01 | 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e |

未在本索引中的 §7 编号（UT-MM-02/03 + UT-MM-09 ConfigureCustomTypes 部分）属于 B5 范围（撤销/重做、缩放、ConfigureCustomTypes）。

## 3. B4 spec 修正

- 原 §7 编号 `UT-MM-09 ConfigureCustomTypes 关闭 → 自定义类型保留` 是 ConfigureCustomTypes 模态的测试，不在本 B4 范围。本 B4 delta 将 `Open 模态 JSON 解析` 也归为 `UT-MM-09`（spec 第 9 项的复用，详见 `core-UI-modals-test-cases.md` §2）。
- `ST-S02-01` / `ST-S02-02` / `ST-S02-03` 是 backend `core-S02-test-cases.md` 中的 API 端到端用例，**不在前端 B4 范围**。前端 B4 仅覆盖 UT-MM-01~09 + ST-MM-01。

## 4. 对齐参考源

- `core-05-top-menu-modals.md` §3 / §4 / §5.1 / §5.2 / §5.7 / §5.8
- `core-UI-modals-test-cases.md`（详细 UT 步骤）
- `frontend-rs/src/editor_panels.rs::modals`（新增子模块）
## ADDED — §9.2 B5 测试 ID 索引（提案：add-frontend-completeness）

> 模块：core | 提案：add-frontend-completeness
> 路径：deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md
> 对齐参考源：`core-05-top-menu-modals.md` §5.3/5.4/5.5/5.6/5.9 + §2.2 Edit 菜单

# B5 模态 + 快捷键 — 测试 ID 索引

## 1. 范围

B5 在 §3 的 9 个模态清单中，补齐最后 **5 个模态** + **键盘快捷键**：
- Import（§5.3）
- ImportSource（§5.4）
- Language（§5.5）
- SetTableWidth（§5.6）
- ConfigureCustomTypes（§5.9）
- 全局键盘：Ctrl+Z / Ctrl+Shift+Z（§2.2 Edit 菜单）

## 2. 测试 ID 索引

| TC ID | 描述 | 对齐实现 | B5 状态 |
|---|---|---|---|
| UT-MM-10 | Import 模态 SQL 解析（parse_sql_statements） | `editor_panels.rs::modals::parse_sql_statements` | ✅ B5 实现 |
| UT-MM-11 | SetTableWidth 模态宽度解析（parse_table_width） | `editor_panels.rs::modals::parse_table_width` | ✅ B5 实现 |
| UT-MM-12 | Language 模态验证（validate_language） | `editor_panels.rs::modals::validate_language` | ✅ B5 实现 |
| UT-MM-13 | ConfigureCustomTypes 增删（add/remove_custom_type） | `editor_panels.rs::modals::{add,remove}_custom_type` | ✅ B5 实现 |
| UT-MM-14 | ImportSource 模态选择解析（resolve_import_source） | `editor_panels.rs::modals::resolve_import_source` | ✅ B5 实现 |
| UT-MM-15 | CommandStack::undo 弹出最近命令 | `editor_core.rs::CommandStack::undo` | ✅ B5 实现 |
| UT-MM-16 | CommandStack::redo 弹出最近 undo | `editor_core.rs::CommandStack::redo` | ✅ B5 实现 |
| UT-KB-01 | 键盘事件 Ctrl+Z 触发 undo（is_undo_shortcut） | `editor_panels.rs::modals::is_undo_shortcut` | ✅ B5 实现 |
| ST-MM-02 | 端到端 Import 模态 SQL 解析 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e |
| ST-MM-03 | ConfigureCustomTypes 关闭后跨刷新保留 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e（V1 限制） |
| ST-UI-05 | Ctrl+Z / Ctrl+Shift+Z 键盘快捷键 e2e | `frontend-rs/tests/wasm/kb.rs` | ⏭️ B5 e2e |

## 3. B5 spec 修正

- 原 §7 编号 UT-MM-02（Edit → Undo → 撤销栈 -1）+ UT-MM-03（View → Zoom In → 画布放大 0.25x）也属于本 B5 范围。UT-MM-02 由 `CommandStack::undo` + 键盘 Ctrl+Z 覆盖；UT-MM-03（Zoom In）属画布交互，本 B5 不实现（V1 边界）。
- `ST-S02-04`（Import 模态 SQL 解析）+ `ST-S02-05`（Language 模态切换 i18n）是 backend `core-S02-test-cases.md` 中的端到端用例，**不在前端 B5 范围**。前端 B5 用 UT-MM-10~14 覆盖解析/校验纯函数。
- `ST-UI-05` 键盘快捷键 e2e 留 B5 wasm-pack test 接入后跑（UT-KB-01 已用纯函数覆盖 `is_undo_shortcut` 逻辑）。

## 4. B5 实施分解（避免单批过大）

按 rollback 条件，B5 可拆为 B5a + B5b 顺序执行：
- **B5a**：5 个剩余模态（UT-MM-10~14）+ 模态组件（Import/ImportSource/Language/SetTableWidth/ConfigureCustomTypes）
- **B5b**：键盘快捷键（UT-MM-15/16 + UT-KB-01）+ 修正 `core-implementation-checklist.md`

实际可按单批闭环（每批 5~6 UT）执行，避免无谓拆分。

## 5. 对齐参考源

- `core-05-top-menu-modals.md` §5.3 / §5.4 / §5.5 / §5.6 / §5.9 / §2.2
- `core-UI-modals-2-test-cases.md`（5 模态 UT 详细步骤）
- `core-KB-shortcut-test-cases.md`（键盘快捷键 UT 详细步骤）
- `frontend-rs/src/editor_panels.rs::modals`（B4 子模块扩展）
- `frontend-rs/src/editor_core.rs::CommandStack`（扩展 undo/redo）

## ADDED — §9.3 B1 测试 ID 索引（提案：fix-modal-overlay-blocking）

> 模块：core | 提案：fix-modal-overlay-blocking
> 路径：deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md
> 对齐参考源：`core-05-top-menu-modals.md` §4.1（遮罩生命周期）+ §5.7（Share）+ `test/fix-modal-blocking-test-cases.md`

# B1 Modal 遮罩修复 — 测试 ID 索引

## 1. 范围

B1 修一处真实 UI bug（ModalRoot 遮罩无条件渲染）+ 补 1 个 testid 锚点（画布）+ 加 1 项 backend middleware（CORS），单批次闭环：

- 修：`<div class="cdb-modal-overlay">` 受 `kind.get().is_some()` 控制（详见 §4.1 遮罩生命周期新约束）
- 补：`<div class="cdb-canvas-container">` 加 `data-testid="editor-canvas"`（详见 `core-01-editor-canvas.md` §5.1）
- 加：backend `actix-cors` middleware（dev 模式允许 `http://localhost:8080` 跨源 PUT）

## 2. 测试 ID 索引

| TC ID | 描述 | 对齐实现 | B1 状态 |
|---|---|---|---|
| UT-FIX-01 | ModalRoot 在 `kind=None` 时不渲染遮罩 div | `editor_panels.rs::modals::ModalRoot` | ✅ B1 实现 |
| ST-FIX-01 | Playwright e2e 5/5 HP 全 PASS（HP-01~HP-05） | `frontend-rs/scripts/e2e-smoke.mjs` | ✅ B1 实现 |

## 3. B1 spec 修正

- 原 §7 编号 UT-MM-04（模态背景点击关闭）已通过 B4 实现，本 B1 不重复。UT-FIX-01 与 UT-MM-04 行为正交：
  - UT-MM-04 验证「模态打开时点击背景能关闭」（B4 已 PASS）
  - UT-FIX-01 验证「模态关闭后遮罩 div 必须从 DOM 移除」（B1 新约束）
- ST-FIX-01 是 `add-frontend-completeness` 提案 [deploy] section 列出的 smoke 验证的**重新执行**（前次 0/5 FAIL 因 ModalRoot 遮罩 + editor-canvas testid 缺失 + CORS 缺失三重原因，本 B1 修复后重跑）

## 4. 验收要点

- HP-01（Load blank editor）：页面初始无 `kind=Some(_)`，DOM 中**不应**存在 `cdb-modal-overlay`
- HP-02~HP-05：所有非模态操作（点击 button / 点击 file menu 项）必须可达，不再被 `intercepts pointer events` 拦截
- 行为等价性：模态打开时仍能背景点击关闭（UT-MM-04 已覆盖）

## 5. 不在本 B1 范围

- add-frontend-completeness 留下的其他 stub（share URL 加载 / import submit handler / undo-redo 实际 effect / set_ref 实际 effect）— 后续专门提案
- actix-cors 配置文件化（V1 用 `Cors::permissive()`，生产由 config.toml 切换的留 V2）

## 6. 对齐参考源

- `core-05-top-menu-modals.md` §4.1（遮罩生命周期新约束，本 B1 加）
- `core-01-editor-canvas.md` §5.1（testid 新约束）
- `logos/resources/test/fix-modal-blocking-test-cases.md`（详细 UT-FIX-01 + ST-FIX-01 步骤）
- `logos/spec/smoke-report.md`（前次 0/5 FAIL 证据）
- `logos/changes/archive/20260610-2122-add-frontend-completeness/`（前置提案）

## ADDED — §12 AppBar IO 按钮与 IO 抽屉（Phase C）

> 模块：core | 提案：redesign-phase-c-import-export

### 12.1 AppBar 导入 / 导出（Phase C 生效）

| 按钮 | testid | Phase C 行为 |
|------|--------|--------------|
| 导入 | `btn-import` | **启用**；点击 → `io_drawer = Import` |
| 导出 ▾ | `btn-export` | 点击 → `io_drawer = Export`（V1 无下拉子项，单按钮开抽屉） |

- 移除 Phase A 占位：`disabled` + tooltip「导入功能即将推出」
- 保存状态、撤销/重做、分享行为不变

### 12.2 File 菜单「导入」改线

| 菜单项 | V1 行为 | Phase C 行为 |
|--------|---------|--------------|
| 导入 | 打开 `ImportModal` | 打开 `ImportDrawer`（`io_drawer = Import`） |

New / Open / Rename / Share 仍走模态。

### 12.3 Import 模态降级

- `ModalKind::Import` 保留组件与 `parse_sql_statements` UT，**默认 UI 路径不再触发**
- e2e HP-04（SQL 模态 parse）迁移为 ST-PC-01（ImportDrawer parse summary）或保留模态仅测试路径

### 12.4 Phase C 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-AB-04 | **更新**：`btn-import` Phase C 为 **enabled** |
| ST-PC-01 | e2e：AppBar 导入 → 抽屉 → 解析摘要 |

## MODIFIED — §5.3 Import 模态（补充说明）

**Phase C 备注**：主交互迁移至 `core-01d-import-export.md` ImportDrawer；本节模态规格保留供回归 UT-MM-10，不作为默认用户路径。
