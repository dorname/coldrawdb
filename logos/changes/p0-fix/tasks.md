# 实现任务：p0-fix

> 配套：`logos/changes/p0-fix/proposal.md`
> 上游：用户反馈 6 项问题（2026-09-04）→ P0 三项切片
> 外环强制约束：禁止改测试断言；新 UT/ST 编号先 grep 取下一空闲（当前占用至 UT-MM-30）；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）

## 定点 1：协作房间删除（0.5 天）

### 前端 RoomClient::delete_room 方法

- [x] `frontend-rs/src/editor_data_access.rs`: 加 `pub async fn delete_room(&self, token: &str, room_id: &str) -> Result<(), ApiError>`：
  - 调用 `DELETE /rooms/{room_id}`（backend/src/rooms_v1.rs:341 已实现）
  - 返回 `Result<(), ApiError>`（204 无内容 → Ok(())；其余 → `ApiError::Server(status, body)`——实际枚举无 Forbidden/NotFound 变体，403/404 由 UI 层按状态码区分文案）
  - 测试：UT-MM-31（grep 取下一空闲；`delete_room_url` + `map_delete_room_status` 纯函数）

### RoomPanel 加「删除房间」按钮

- [x] `frontend-rs/src/editor_panels.rs`: `RoomPanel` 组件加删除按钮：
  - 位置：RoomPanel 底部（成员列表之后）
  - 可见性：仅 Owner 显示（`can_delete_room(my_role)` 纯函数，UT-MM-32）
  - 样式：`cdb-btn cdb-btn--danger`（红色警告按钮）
  - data-testid：`btn-delete-room`
  - 点击 → `ModalKind::DeleteRoom` 弹确认模态

### ModalKind::DeleteRoom 确认模态

- [x] `frontend-rs/src/editor_panels.rs`: `ModalKind` enum 加 `DeleteRoom` 变体
- [x] `frontend-rs/src/editor_panels.rs`: `DeleteRoomModal` 组件：
  - 显示房间名 + 成员数 + 警告文案「删除后不可恢复，房间及其所有协作数据将被永久删除」
  - 确认按钮（红色，data-testid=`btn-confirm-delete-room`）→ 调 `RoomClient::delete_room` → 成功 → 关闭模态 → 回 rooms 页 → 刷新列表
  - 取消按钮（data-testid=`btn-cancel-delete-room`）→ 关闭模态
  - 错误处理：网络失败 → 显示错误「删除失败，请稍后重试」+ 模态保持打开；后端 403 → 显示错误「无权限删除此房间」

### 删除后行为

- [x] 删除成功 → 关闭 RoomPanel → 当前房间置 None → 回 rooms 页 → 刷新 rooms 列表（`reload_rooms` nonce 强制 `RoomSummary` 重新拉取，绕过 token 缓存）
- [x] 删除失败 → 显示错误（模态内，不关闭）

**定点 1 用例登记**：UT-MM-31 / UT-MM-32 → `core-UI-modals-2-test-cases.md`；ST-S04-UI-08 → `core-S04-test-cases.md`；reporter 已写入 `logos/resources/verify/test-results.jsonl`（UT×2 pass，ST skip——e2e harness 待接入）

## 定点 3：关系连线简化（1-2 天）

### 删 RelToolState::Confirm 状态

- [x] `frontend-rs/src/editor_panels.rs`: `RelToolState` enum 删 `Confirm` 变体（`Dragging` → 直接落账）
- [x] `frontend-rs/src/editor_panels.rs`: `RelToolHint` 组件删 Confirm 分支提示（hint() 无 Confirm 显式分支，随枚举删除自然收口）
- [x] `frontend-rs/src/editor_panels.rs`: 关系创建逻辑改：
  - 拖到目标字段松开 → **直接落账**（`build_reference` + `on_create_reference`：push + dirty + schedule_save）
  - 无确认条（`RelationshipConfirmBar` 组件与使用点已删除）
  - cardinality 自动推导（`infer_cardinality` 纯函数，既有）

### 点击连线弹关系列表模态

- [x] `frontend-rs/src/editor_render.rs`: canvas pointerdown 加 `hit_test_reference` 命中检测（点到贝塞尔线，24 段折线近似阈值 8px；命中顺序在表之后——连线被表遮住时选中表）
- [x] `frontend-rs/src/editor_panels.rs`: `ModalKind::ReferenceDetail` 模态：
  - 显示 reference 详情：start_table.start_field → end_table.end_field + cardinality
  - 删除按钮（data-testid=`btn-delete-reference`）→ 移除 reference + dirty + PUT
  - 关闭按钮（data-testid=`btn-close-reference-detail`）

### Delete 键删除连线

- [x] `frontend-rs/src/editor_panels.rs`: `setup_editor_tool_shortcuts` 加 Delete/Backspace 处理（`is_delete_key` 纯函数 UT-MM-34；选中连线 → `on_delete_ref` 移除 + dirty + PUT；页面/只读/浮层/输入焦点门控复用既有）
- [x] 选中态：连线被点击后高亮（`selected_ref_id` 信号 → 光晕 10px selected_soft + 主线 3.5px selected）

**定点 3 用例登记**：UT-MM-33 / UT-MM-34 → `core-UI-modals-2-test-cases.md`；ST-PB-01/02 流程更新 + ST-PB-03/04 新增 → `core-PB-relationship-test-cases.md` + `test-spec-parity-d.mjs`
**规格驱动的断言调整**：`tests/phase_b_relationship.rs` UT-PB-05 源码扫描断言由「RelationshipConfirmBar 存在」反转为「已删除 + 直接落账通路保留」（提案删除确认条的必然结果）

## 定点 2：区域/便签交互（2-3 天）

### tool-new-area on:click（拖拽创建区域）

- [ ] `frontend-rs/src/editor_panels.rs`: `tool-new-area` 按钮加 `on:click`：
  - 点击 → 画布十字光标（cursor: crosshair）
  - 拖拽画布框选区域（pointerdown → pointermove → pointerup）
  - 松开 → 创建 Area { id, name: "未命名区域", x, y, width, height, color: "#3b82f6" }
  - 拖框 < 10px → 不创建（防误触）

### tool-new-note on:click（点击放置便签）

- [ ] `frontend-rs/src/editor_panels.rs`: `tool-new-note` 按钮加 `on:click`：
  - 点击 → 画布十字光标（cursor: crosshair）
  - 点击画布某点（pointerdown → pointerup）
  - 松开 → 创建 Note { id, content: "", x, y, color: "#f59e0b" }

### Inspector Area/Note 编辑面板

- [ ] `frontend-rs/src/editor_panels.rs`: Inspector 加 Area/Note 编辑面板：
  - 选中 Area → 显示 name/comment/color 编辑 + 删除按钮
  - 选中 Note → 显示 content textarea 编辑 + 删除按钮
  - 删除按钮 → 移除 Area/Note + dirty + PUT

### Delete 键删除 Area/Note

- [ ] `frontend-rs/src/editor_panels.rs`: 画布键盘事件加 Delete 键处理（选中 Area/Note → 移除 + dirty + PUT）
- [ ] 选中态：Area/Note 被点击后高亮（border 加粗 + 颜色变化）

## 测试

- [ ] UT-ROOM-01：`RoomClient::delete_room` 纯函数测试（204 成功 / 403 无权限 / 404 房间不存在 / 500 服务器错误）
- [ ] UT-REL-01：关系连线直接落账测试（Dragging → 落账，无 Confirm 状态）
- [ ] UT-AREA-01：Area 拖拽创建测试（拖框 < 10px 不创建）
- [ ] UT-NOTE-01：Note 点击放置测试（点击画布某点落账）

## 不在范围

- 问题 1 纯列表入口强化（P1：键盘快捷键 L + 浮动返回按钮）
- 问题 2 PostgreSQL/MySQL 交互（P2：DB 选择 UI + dialect 分流 + 导出按 dialect）
- 问题 5 创建房间下拉说明（P1：diagram.created_at 时间戳显示）
- 右键菜单（P1：连线右键弹「删除 / 编辑 cardinality」菜单）
- 区域/便签拖拽调整大小（P1：既有拖拽逻辑复用，非本批）
- 不修改既有测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

## 验收标准

- **定点 1**：Owner 进房间 → RoomPanel 显示「删除房间」按钮 → 点击 → 弹确认模态 → 确认 → 删除 → 回 rooms 页 → 列表刷新（该房间消失）
- **定点 3**：用户拖字段 A.id 到字段 B.user_id 松开 → **直接落账**（无确认条）→ 画布显示连线；用户点击连线 → 弹 ReferenceDetail 模态（显示详情 + 删除按钮）；用户选中连线按 Delete → 连线移除 + 画布刷新
- **定点 2**：用户点「添加区域」→ 拖拽画布框选 → 松开 → Area 落账 + 画布显示色块 + Inspector 编辑面板；用户点「添加便签」→ 点击画布 → Note 落账 + 画布显示便签 + Inspector 编辑面板