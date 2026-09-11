# Delta — core-frontend-alignment-acceptance.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 验收边界

唯一现行主原型：`core-01-editor-prototype.html`。**演示 ≠ 生产**：主原型模拟登录/WS/OT/远端光标不得标为生产已同步。

历史 `core-03/04/05-*-prototype.html` 不作为验收入口。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现与联调验收由 `implement-unified-prototype-spec-parity` 执行。

## ADDED — 7. 按页面区域 checklist（生产逐项）

### 7.1 auth
- [ ] 默认未登录入口为 auth（非空白 editor）
- [ ] `login-form` / `register-form` 双 tab、字段错误、loading
- [ ] 错误不枚举用户；无 token 原文
- [ ] 成功进入 rooms

### 7.2 rooms
- [ ] `rooms-list-page` + 列表/空状态 + 创建入口 + 用户菜单
- [ ] 创建/打开进入 `room-editor-page`
- [ ] `room-badge` 可回 rooms

### 7.3 invite
- [ ] `invite-accept-page` preview；过期无加入
- [ ] 未登录接受→提示登录；登录后可续接
- [ ] 接受后进入同一 room

### 7.4 room-editor · 壳层
- [ ] `app-bar` / `tool-rail` / `editor-canvas` / `inspector` / `status-bar`
- [ ] 保存态 `save-state` + revision（S01）
- [ ] 协作 `ws-status` / `ot-rev` / `room-presence` / `reconnect-banner` / Activity
- [ ] Viewer 只读
- [ ] 更多菜单 → IO；⌘K 命令面板；代码视图
- [ ] 主题切换；720px 关键操作可达

### 7.5 画布 / 关系 / IO
- [ ] 表拖动 pointer capture；生产松手 `GRID_SIZE=20`；关系跟手
- [ ] 关系：4px 阈值、rubber-band、点击两点、确认条（生产）
- [ ] IO 抽屉格式预览

### 7.6 明确非验收（演示）
- [ ] 主原型「模拟远端/断线/诊断」控件不要求生产原样提供
- [ ] 不得因演示通过而勾选本清单完成项

## ADDED — 8. 既有 FEALIGN / FEUX 的第二阶段解读

保留 FEALIGN-AC-* / FEUX-AC-* 作为能力维度；本提案以**页面区域 checklist** 为第二阶段主验收入口。Reporter：实现阶段写入 `test-results.jsonl`。
