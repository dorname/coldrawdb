# Delta — core-frontend-alignment-acceptance.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 验收边界

唯一现行主原型：`core-01-editor-prototype.html`。**演示 ≠ 生产**：主原型模拟登录/WS/OT/远端光标不得标为生产已同步。

历史 `core-03/04/05-*-prototype.html` 不作为验收入口。

状态：本提案 `implement-unified-prototype-spec-parity` 执行生产前端逐项对齐。代码完成前仍为「后端已实现；生产前端部分接入」。§7 勾选仅表示对应区域已对照主原型验证，不得因演示器通过而提前勾选。

## MODIFIED — 4. verify 前检查

运行 `openlogos verify implement-unified-prototype-spec-parity` 前，至少应完成：

- A～D 批次业务代码与对应 UT/ST/e2e。
- `SPEC_PARITY_SKIP_IDS` 清空，或仅余带缺口说明的显式 skip。
- 后端 auth/rooms/collab Rust 测试回归。
- 前端 Rust 单元测试回归。
- 统一原型 ST-PU 回归，确认视觉交互基线未破坏。
- reporter 中本提案落地用例 ID 与测试文档一一对应。
- §7 区域 checklist 已按已验证批次勾选。

## MODIFIED — 7. 按页面区域 checklist（生产逐项）

本提案按 A～D 批次逐项勾选。合并本 delta 时保持未勾选；仅在对应代码批次验证后勾选。7.6 为非验收约束，不因演示通过而勾选完成。

### 7.1 auth（A 批）
- [ ] 默认未登录入口为 auth（非空白 editor）
- [ ] `login-form` / `register-form` 双 tab、字段错误、loading
- [ ] 错误不枚举用户；无 token 原文
- [ ] 成功进入 rooms

### 7.2 rooms（B 批）
- [ ] `rooms-list-page` + 列表/空状态 + 创建入口 + 用户菜单
- [ ] 创建/打开进入 `room-editor-page`
- [ ] `room-badge` 可回 rooms

### 7.3 invite（B 批）
- [ ] `invite-accept-page` preview；过期无加入
- [ ] 未登录接受→提示登录；登录后可续接
- [ ] 接受后进入同一 room

### 7.4 room-editor · 壳层（C/D 批）
- [ ] `app-bar` / `tool-rail` / `editor-canvas` / `inspector` / `status-bar`
- [ ] 保存态 `save-state` + revision（S01）
- [ ] 协作 `ws-status` / `ot-rev` / `room-presence` / `reconnect-banner` / Activity
- [ ] Viewer 只读
- [ ] 更多菜单 → IO；⌘K 命令面板；代码视图
- [ ] 主题切换；720px 关键操作可达

### 7.5 画布 / 关系 / IO（D 批）
- [ ] 表拖动 pointer capture；生产松手 `GRID_SIZE=20`；关系跟手
- [ ] 关系：4px 阈值、rubber-band、点击两点、确认条（生产）
- [ ] IO 抽屉格式预览

### 7.6 明确非验收（演示）
- [ ] 主原型「模拟远端/断线/诊断」控件不要求生产原样提供
- [ ] 不得因演示通过而勾选本清单完成项

## MODIFIED — 8. 既有 FEALIGN / FEUX 的第二阶段解读

保留 FEALIGN-AC-* / FEUX-AC-* 作为能力维度；本提案以**页面区域 checklist** 为验收入口并执行实现。Reporter：本提案写入 `test-results.jsonl`；skip 必须说明 harness 缺口，不得静默缺失。
