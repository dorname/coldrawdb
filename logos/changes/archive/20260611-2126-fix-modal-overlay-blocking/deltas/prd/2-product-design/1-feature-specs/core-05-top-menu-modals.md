## MODIFIED — §4.1 打开 / 关闭 + §9 测试 ID 索引（提案：fix-modal-overlay-blocking）

> 模块：core | 提案：fix-modal-overlay-blocking
> 路径：`deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
> 对齐参考源：`core-05-top-menu-modals.md` §4.1 模态通用模式

# 模态遮罩行为约束（fix B1）

## 1. 范围

本 delta 修一处描述 + 补 B1 测试 ID 索引，对应 `fix-modal-overlay-blocking` 提案 B1 批次：

- 修：`core-05-top-menu-modals.md` §4.1 中"背景点击"描述，**显式约束**遮罩 div 仅在模态打开时存在
- 补：§9 索引补 `UT-FIX-01`（ModalRoot 条件渲染）

## 2. 修改点（§4.1 第 90 行）

**原描述**：
```
- 关闭：右上角 × / ESC / 背景点击
```

**修改后**：
```
- 关闭：右上角 × / ESC / 背景点击
- **遮罩生命周期**：`<div class="cdb-modal-overlay">`（即"背景"）**仅在 `kind.get().is_some()` 时存在**。
  模态关闭（`kind` 回到 `None`）时遮罩必须从 DOM 移除，否则遮罩会持续拦截全屏 pointer events，
  阻挡非模态 UI 的所有点击（HP-01~HP-05 回归验收点）
```

## 3. §9 测试 ID 索引（追加）

| TC ID | 描述 | 对齐实现 | B1 状态 |
|---|---|---|---|
| UT-FIX-01 | ModalRoot 在 `kind=None` 时不渲染遮罩 div | `editor_panels.rs::modals::ModalRoot` | ✅ B1 实现 |
| ST-FIX-01 | Playwright e2e 5/5 HP 全 PASS（HP-01~HP-05） | `frontend-rs/scripts/e2e-smoke.mjs` | ✅ B1 实现 |

## 4. 验收要点

- HP-01（Load blank editor）：页面初始无 `kind=Some(_)`，DOM 中**不应**存在 `cdb-modal-overlay`
- HP-02~HP-05：所有非模态操作（点击 button / 点击 file menu 项）必须可达，不再被 `intercepts pointer events` 拦截
- 行为等价性：模态打开时仍能背景点击关闭（原有 on:click handler 保留）

## 5. 对齐参考源

- `logos/changes/archive/20260610-2122-add-frontend-completeness/` — 前置提案
- `logos/spec/smoke-report.md` — smoke 0/5 FAIL 失败证据
- `frontend-rs/src/editor_panels.rs::modals::ModalRoot`（line 1456）— Bug 现场
