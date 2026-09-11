# 顶部菜单模态测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-UI-modals-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §3 + §5.1/5.2/5.7/5.8

## 1. 范围

主模态：遮罩、Esc、背景点击关闭，关闭后**无残留** overlay。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## MODIFIED / ADDED — 关闭与残留

| ID | 变更 | 合同 |
|---|---|---|
| UT-MM-04 / UT-MM-05 | MODIFIED | 背景点击 / Esc → `modal_kind=None`；`modal-root` 与遮罩从 DOM 移除或 `hidden`；焦点回退合理 |
| ST-MM-ESC-01（ADDED） | ADDED | 连续打开 New/Share/Conflict 再 Esc：无透明拦截层；画布可点击 |
| ST-MM-CONFLICT-OT（ADDED） | ADDED | 协作 OT 成功路径**不**打开 conflict 模态（与 S01/S05 交叉） |
| UT-MM-01/06/07/08 | 保留 | 标题校验、Share URL 格式等 |

## 2. UT 用例

### UT-MM-01 — New 模态创建 diagram（纯函数路径）

- **位置**：`frontend-rs/src/editor_panels.rs::modals::validate_title` + `build_create_url`
- **前置**：编辑器处于打开状态
- **步骤**：
  1. 打开 File → New 模态
  2. 输入 title="My New Diagram"
  3. 点 OK
- **断言**：
  - `validate_title("My New Diagram") == Ok(())`
  - `build_create_url("d-new") == "/editor/d-new"`
  - 提交时调用 `editor_data_access::create` 并跳转

### UT-MM-04 — 模态背景点击关闭

- **位置**：`frontend-rs/src/editor_panels.rs::modals::ModalRoot`
- **前置**：任一模态打开中
- **步骤**：在遮罩背景（非模态体）上点击
- **断言**：
  - `modal_kind` 信号被设为 `None`
  - DOM 中不再有 `data-testid="modal-root"`
  - 模态体（`data-testid="modal-{kind}"`）不再可见

### UT-MM-05 — 模态 ESC 键关闭

- **位置**：同 UT-MM-04
- **步骤**：在模态打开时按 ESC
- **断言**：同 UT-MM-04

### UT-MM-06 — 必填字段失焦红框

- **位置**：`frontend-rs/src/editor_panels.rs::modals::NewModal` / `RenameModal`
- **步骤**：
  1. 打开 New 模态
  2. 不输入直接点 OK 或失焦
- **断言**：
  - `validate_title("") == Err("title 不能为空")`
  - 输入框 class 包含 `cdb-is-invalid`
  - OK 按钮禁用

### UT-MM-07 — New 模态 title 为空 → OK 禁用

- **位置**：`frontend-rs/src/editor_panels.rs::modals::NewModal`
- **步骤**：打开 New 模态，title 输入框为空
- **断言**：
  - OK 按钮 `disabled` 属性存在
  - 点击 OK 不触发 `editor_data_access::create`

### UT-MM-08 — Share 模态 URL 格式正确

- **位置**：`frontend-rs/src/editor_panels.rs::modals::build_share_url`
- **步骤**：传入 `diagram_id = "abc-123"`
- **断言**：
  - `build_share_url("abc-123") == "/editor?share=abc-123"`
  - URL 字段在模态中以 read-only 形式展示
  - 点 Copy → 调用 `navigator.clipboard.write_text` 写入该 URL

### UT-MM-09 — Open 模态 JSON 解析

- **位置**：`frontend-rs/src/editor_panels.rs::modals::parse_diagram_json`
- **步骤**：
  1. 用户上传 `diagram.json`（合法 Diagram JSON 字符串）
  2. 调用 `parse_diagram_json(text)`
- **断言**：
  - 合法 JSON → `Ok(Diagram { ... })`
  - 非法 JSON → `Err("JSON parse error: ...")`，模态显示错误不跳转

## 3. ST 用例

### ST-MM-01 — 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路

- **位置**：`frontend-rs/tests/wasm/ui.rs`（B5 接入）
- **类型**：wasm-pack test --headless --chrome
- **步骤**：
  1. 启动后端 + 前端
  2. 点击 File → New → 输入 "e2e-diagram" → OK
  3. 验证跳转到 `/editor/{new_id}` + URL 含新 id
  4. 点击 File → Share → 验证 URL 显示
  5. 点击 File → Rename → 改为 "renamed" → OK → 验证标题变化
  6. 点击 File → Open → 输入 URL 中提取的 id → 验证加载
- **B4 标记 skip**：完整 e2e 跑在 B5 wasm-pack test 接入后

## 层级

主模态必须高于抽屉/Popover；关闭动画结束（或 reduced-motion 即时）后不得残留 pointer-events 挡板。

## 4. V1 边界

- ❌ Import / ImportSource / Language / SetTableWidth / ConfigureCustomTypes 模态（B4 仅 4 个核心）
- ❌ File 菜单下拉展开动画（B4 简单 show/hide，无 transition）
- ❌ 取消时未保存修改确认弹窗（V1 直接关闭）
- ❌ 模态拖拽移动位置（V1 居中固定）

## 5. 对齐参考源

- `core-05-top-menu-modals.md` §3 / §5.1 / §5.2 / §5.7 / §5.8
- `frontend-rs/src/editor_panels.rs::ModalRoot`（新增）
- `frontend-rs/src/editor_panels.rs::{NewModal, OpenModal, ShareModal, RenameModal}`（新增）
- `frontend-rs/src/editor_data_access.rs::create` / `get`

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-MM-01 | New 模态创建 diagram（纯函数路径） | `editor_panels.rs::modals::validate_title` + `build_create_url` |
| UT-MM-04 | 模态背景点击关闭 | `editor_panels.rs::modals::ModalRoot` |
| UT-MM-05 | 模态 ESC 键关闭 | `editor_panels.rs::modals::ModalRoot` |
| UT-MM-06 | 必填字段失焦红框 | `editor_panels.rs::modals::{NewModal,RenameModal}` |
| UT-MM-07 | New 模态 title 为空 → OK 禁用 | `editor_panels.rs::modals::NewModal` |
| UT-MM-08 | Share 模态 URL 格式正确 | `editor_panels.rs::modals::build_share_url` |
| UT-MM-09 | Open 模态 JSON 解析 | `editor_panels.rs::modals::parse_diagram_json` |
| ST-MM-01 | 端到端：菜单 / 模态 / 工具栏 / 快捷键 全链路 | `frontend-rs/tests/wasm/ui.rs`（B5） |
