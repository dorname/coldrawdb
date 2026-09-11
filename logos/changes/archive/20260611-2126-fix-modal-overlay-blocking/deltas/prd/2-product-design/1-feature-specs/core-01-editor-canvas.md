## ADDED — §5 渲染策略：editor-canvas testid 约束（提案：fix-modal-overlay-blocking）

> 模块：core | 提案：fix-modal-overlay-blocking
> 路径：`deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
> 对齐参考源：`core-01-editor-canvas.md` §5.1 渲染主体

# 画布 testid 约束（fix B1）

## 1. 范围

本 delta 在 §5.1 渲染主体下追加 testid 约束，对应 `fix-modal-overlay-blocking` 提案 B1 批次：

- 加：`cdb-canvas-container` 必须带 `data-testid="editor-canvas"`
- 补：§5.5 测试 ID 索引补 `UT-FIX-02`（编译期 grep 断言）

## 2. 修改点（§5.1 渲染主体追加）

**原 §5.1**（line 53-57）：
```
### 5.1 渲染主体

- **render 层**：`frontend-rs/editor_render` 使用 `<canvas>`（HTML5）+ 贝塞尔连线（自渲染，无 vDOM diff）
- **响应式**：基于 Leptos signals 细粒度更新（仅重绘变更部分）
- **性能预算**：100 张表 / 200 条关系 / 60fps（来源：Phase 4 W4 perf）
```

**追加一条**：
```
- **画布容器 testid**：`<div class="cdb-canvas-container">` 必须带 `data-testid="editor-canvas"`,
  用于 e2e 定位画布区域（HP-01 / HP-05 锚点）
```

## 3. §5.5 测试 ID 索引（追加）

| TC ID | 描述 | 对齐实现 | B1 状态 |
|---|---|---|---|
| UT-FIX-02 | `cdb-canvas-container` 含 `data-testid="editor-canvas"`（编译期 grep 断言） | `frontend-rs/src/editor_panels.rs` | ✅ B1 实现 |

## 4. 验收要点

- HP-01：page.goto 之后 `await page.locator('[data-testid="editor-canvas"]').waitFor()` 必须可见
- HP-05：键盘快捷键测试的 focus 锚点用 `editor-canvas` testid
- 行为等价性：testid 不影响 CSS 样式、不影响事件冒泡

## 5. 对齐参考源

- `logos/changes/archive/20260610-2122-add-frontend-completeness/` — 前置提案
- `logos/spec/smoke-report.md` — HP-01/05 失败原因
- `frontend-rs/src/editor_panels.rs` line 1214（画布 div）— 改动点
