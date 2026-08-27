# 变更提案：fix-canvas-interaction-hotfixes

> module: core | created: 2026-08-27
> 前置：core 模块已 launched；fix-canvas-hidpi-rendering (20260826-1330) 已归档
> 性质：hotfix 集合（用户主动报告 3 个交互问题）

## 变更原因

Change 1 (`fix-canvas-hidpi-rendering`) 归档后用户主动报告 3 个独立交互 bug，
均与画布渲染 / 交互态相关，根因各异。逐个 fix 后合并归档。

### 子问题 1：选中关系后画布无法拖动
- 症状：点击关系 endpoint 选中后，鼠标在画布空白处无法 pan，也无法拖动表
- 根因：
  - `on_pointerdown` rel_tool_active 未命中字段时直接 `return`（关系工具下点空白无法 pan）
  - `on_pointerup` 缺 endpoint_drag 显式处理分支
  - 无 `pointercancel` 监听（Alt+Tab 切窗口后 drag_state 残留）
- 修复 commit：`a17b710`

### 子问题 2：刷新后登录丢失 + 表重叠在原点
- 症状 1：登录后刷新页面要求重新登录
- 症状 2：刷新后原本分布的表全部 (0,0) 重叠
- 根因：
  - S03 AuthSession 是纯内存 RwSignal，刷新即丢，无 localStorage 持久化
  - `backend diagram_persistence::row_f64` 用 `try_get::<String>` 取 INTEGER/REAL 列，
    SQLite `NUMERIC` affinity 存 `60.0` 为 INTEGER，`try_get::<String>` 返回 Err，
    fallback `0.0` → 4 张表全部重叠
- 修复 commit：`e0413fb`

### 子问题 3：滚轮缩放内容跑出界面 + 鼠标无法正确移动画布
- 症状：滚轮缩放后画布"漂"出 viewport；pan 时鼠标方向与画布方向偏差
- 根因：
  - `on_wheel` 反向计算 `new_pan = mouse - diag*zoom` 缺少 `rect.left` 偏移
  - `screen_to_diagram` 内部 `mouse - rect.left` 减了 canvas 边界偏移，
    反向计算必须同步减 `rect.left`，否则每次缩放累积偏移，多次后画布漂出
  - 副作用：累积偏量下 on_pointermove 用屏幕 delta 看似正确，但 transform.pan
    本身错位，pan 方向偏差
- 修复 commit：`3b41cd4`

## 变更范围

### 子问题 1
- `frontend-rs/src/editor_render.rs`
  - `on_pointerdown` rel_tool_active 未命中字段 → 回落 pan 模式
  - `on_pointerup` 新增 endpoint_drag 显式分支 + return
  - 新增 `on_pointercancel` 闭包 + `on:pointercancel` 监听器
  - `on_pointerdown` 开头防御性清 stale drag_state

### 子问题 2
- `frontend-rs/src/editor_data_access.rs`
  - `persist_auth_session / restore_auth_session / clear_auth_session`
  - localStorage key: `coldrawdb.auth_session.v1`
- `frontend-rs/src/editor_panels.rs`
  - login 成功 → persist
  - refresh_session 成功 → persist
  - logout / 401 → clear
  - AppRoot 启动 spawn_local 异步验证 token 有效性，恢复 session
- `backend/src/diagram_persistence.rs`
  - `row_f64` 依次 `try_get::<f64> → <i64> → <String>`，覆盖 INTEGER/REAL/TEXT

### 子问题 3
- `frontend-rs/src/editor_render.rs::on_wheel`
  - 显式算 `anchor = mouse - rect.left`，`pan = anchor - diag*zoom`

## 部署影响

- 是否需要部署：是（frontend wasm 替换 + backend 二进制替换）
- 部署原因：前端行为修复 + 后端读取修复
- 影响环境：本地 + staging
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（保留上一版 dist + backend 二进制）
- 是否需要 smoke：是（HP-01~HP-05 仍 PASS；新 SMOKE 用例可选）

## 合并的提交

```
a17b710 fix(editor-render): 修复选中关系后画布无法拖动的 bug
e0413fb fix(auth+canvas): 登录持久化 + 表 x/y 读取修复
3b41cd4 fix(editor-render): 修复滚轮缩放 pan 累积漂移
```

## 验收（本地端到端）

| 验证项 | 预期 |
|---|---|
| `cargo check --target wasm32-unknown-unknown` | ✅ pass |
| `cargo build --release` (backend) | ✅ pass |
| `trunk build --release` (frontend) | ✅ pass |
| `openlogos verify` | Gate 3.6 PASS（244/266 pass, 0 fail, 22 skip） |
| `openlogos smoke` | Gate 3.8 PASS（6/6 PASS） |
| **行为验收 1**：选中关系后画布可拖动 | 复测 a17b710 路径 |
| **行为验收 2**：登录刷新保持登录 + 表位置正确 | 复测 e0413fb 路径 |
| **行为验收 3**：滚轮缩放内容不漂 + pan 方向正确 | 复测 3b41cd4 路径 |

## 不在本变更范围

- Monaco wasm 完整挂载
- `indices` 接收 frontend 写入
- 22 个剩余 skip（spec-defined / 视觉回归 / 杂项 e2e）