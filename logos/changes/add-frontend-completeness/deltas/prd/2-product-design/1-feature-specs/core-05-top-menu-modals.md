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
