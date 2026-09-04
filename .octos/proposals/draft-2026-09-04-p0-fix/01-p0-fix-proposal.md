# 变更提案：p0-fix（协作房间删除 + 区域/便签交互 + 关系连线简化）

> module: core | created: 2026-09-04
> 状态：**草案**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 上游：用户反馈 6 项问题（列表视图/DB UI/区域便签/关系连线/房间删除/创建房间下拉）→ P0 三项切片
> 上游判词：用户 2026-09-04 P0 优先级指派（问题 6 + 3 + 4）
> 工作量档位：4-5 天档（P0 三项合计：0.5 + 2-3 + 1-2 = 3.5-5.5 天）

## 变更原因

**UX 缺口**（用户反馈 6 项问题中 P0 三项）：

1. **问题 6：协作房间无法删除**
   - 现状：`backend/src/rooms_v1.rs:341 delete_room_handler` 已实现（archive_room，返回 204）
   - 前端缺口：**无 UI 调用入口**——RoomPanel 仅成员管理（`change_role`/`remove_member`），无「删除房间」按钮
   - 用户路径：rooms 页 → 进房间 → 想删但无入口 → 只能联系管理员

2. **问题 3：区域（Areas）/ 便签（Notes）不可用**
   - 现状：`tool-new-area`/`tool-new-note` 按钮存在（`editor_panels.rs:3662/3670`），但 **on:click handler 缺失**
   - Areas/Notes 渲染已有（`editor_render.rs:1454 draw_area` / `:1427-1436` draw_note）
   - AreasTab/NotesTab 组件存在（`editor_panels.rs`），store.areas / store.notes 信号已有（`editor_core.rs:148-149`）
   - 用户路径：工具栏点按钮 → 无反应 → 功能不可用

3. **问题 4：关系连线交互冗余**
   - 现状：创建关系后弹确认条（`RelToolState::Confirm`，`editor_panels.rs:388-402`），须点「创建」才落账
   - 用户反馈：「没有必要再多加一次创建 one to one 的交互」——即松开即落账（无需确认）
   - 缺失：画布点击连线无反应（应弹关系列表）；画布不能直接删除连线（应 Delete 键）

## 变更内容（3 项定点）

### 定点 1：协作房间删除（问题 6）

**现状事实层**：
- 后端 `DELETE /rooms/{room_id}` 已实现（`backend/src/rooms_v1.rs:341`，archive_room 返回 204）
- 前端 `RoomClient::delete_room` 未实现（`frontend-rs/src/editor_data_access.rs` grep 无命中）
- RoomPanel 仅成员管理（`editor_panels.rs:2132 RoomPanel`），无删除按钮
- 权限：仅 Owner 可删除（`backend/src/rooms_v1.rs` archive_room 内部权限检查）

**变更**：
- **前端 `RoomClient::delete_room` 方法**：`DELETE /rooms/{room_id}` 调用（`RoomClient` 在 `editor_data_access.rs`）
- **RoomPanel 加「删除房间」按钮**（Owner 可见，角色 = owner 时显示）
- **删除前确认模态**（`ModalKind::DeleteRoom`，显示房间名 + 成员数 + 警告文案）
- **删除后行为**：关闭 RoomPanel → 刷新 rooms 列表 → 当前房间置 None → 回 rooms 页

**真值表**：

| 角色 | 按钮可见 | 点击行为 |
|---|---|---|
| Owner | ✅ | 弹确认模态 → 确认 → 删除 → 回 rooms 页 |
| Editor | ❌ | 无按钮（不渲染） |
| Viewer | ❌ | 无按钮（不渲染） |
| 未登录 | ❌ | 无按钮（auth guard） |

**实例推演**：
- **happy 1**：Owner 进房间 → RoomPanel 显示「删除房间」按钮 → 点击 → 弹确认模态（显示房间名「数据模型评审」+ 成员数「3 位成员」+ 警告「删除后不可恢复」）→ 确认 → DELETE /rooms/{id} 204 → 回 rooms 页 → 列表刷新（该房间消失）
- **edge 1**：Editor 进房间 → RoomPanel 无「删除房间」按钮（不渲染）
- **edge 2**：Owner 点击删除但网络失败 → 显示错误「删除失败，请稍后重试」+ 模态保持打开
- **edge 3**：Owner 点击删除但后端 403 → 显示错误「无权限删除此房间」（后端权限检查）

### 定点 2：区域/便签交互（问题 3）

**现状事实层**：
- `tool-new-area`/`tool-new-note` 按钮存在（`editor_panels.rs:3662/3670`），**on:click 缺失**
- Areas 渲染：`editor_render.rs:1454 draw_area`（带 name/comment/color 渲染）
- Notes 渲染：`editor_render.rs:1427-1436 draw_note`（带 content 渲染）
- store.areas / store.notes 信号已有（`editor_core.rs:148-149`）
- AreasTab/NotesTab 组件存在（`editor_panels.rs`），含 `new_default_area`/`new_default_note` 构造

**变更**：
- **`tool-new-area` on:click**：画布拖拽创建区域（拖框选画布区域 → 松开即落账 Area { id, name, x, y, width, height, color }）
- **`tool-new-note` on:click**：画布点击放置便签（点击画布某点 → 落账 Note { id, content, x, y, color }）
- **Inspector 加 Area/Note 编辑面板**（选中 Area/Note 时显示 name/comment/color 编辑 + 删除按钮）
- **Delete 键删除**：选中 Area/Note 后按 Delete → 移除 + store.dirty 置位 + PUT

**真值表（创建交互）**：

| 按钮 | 交互 | 结果 |
|---|---|---|
| tool-new-area | 点击 → 拖拽画布框选区域 → 松开 | 创建 Area { name: "未命名区域", x, y, width, height, color: "#3b82f6" } |
| tool-new-note | 点击 → 点击画布某点 | 创建 Note { content: "", x, y, color: "#f59e0b" } |
| Delete 键（选中 Area/Note） | 按 Delete | 移除 Area/Note + store.dirty = true + PUT |
| Inspector Area/Note 编辑 | 改 name/comment/color | 更新 store + dirty + PUT |

**实例推演**：
- **happy 1**：用户点「添加区域」按钮 → 画布十字光标 → 拖拽框选（x=100, y=100 到 x=300, y=200）→ 松开 → Area 落账（name=未命名区域，width=200, height=100）→ 画布显示半透明色块 + 名称 → Inspector 显示 Area 编辑面板
- **happy 2**：用户点「添加便签」按钮 → 画布十字光标 → 点击（x=150, y=250）→ 松开 → Note 落账（content="", x=150, y=250）→ 画布显示便签图标 + 内容 → Inspector 显示 Note 编辑面板（content textarea 可编辑）
- **edge 1**：用户点「添加区域」但拖框 < 10px → 不创建（防误触）
- **edge 2**：用户选中 Area 后按 Delete → Area 移除 + 画布刷新 + Inspector 清空
- **edge 3**：用户拖拽 Area 边框调整大小 → 更新 width/height + dirty + PUT（既有拖拽逻辑复用）

### 定点 3：关系连线简化（问题 4）

**现状事实层**：
- `RelToolState::Confirm` 状态存在（`editor_panels.rs:388-402`），含 start_table_id/start_field_id/end_table_id/end_field_id/cardinality
- 确认条渲染：`rel-confirm-bar`（`editor_panels.rs`），含「创建」按钮 → 调用 `create_reference` 落账
- 删除连线：Inspector 有删除（`delete_reference` 调用），但画布**无直接删除**
- 点击连线：**无命中检测**（canvas click 事件未消费 reference hit）

**变更**：
- **删 Confirm 状态**：拖到目标字段松开即落账（`RelToolState::Dragging` → `store.references.push(new_reference)` + dirty + PUT），**无确认条**
- **点击连线 → 弹关系列表模态**（`ModalKind::ReferenceDetail`，显示 reference 详情：start_table.start_field → end_table.end_field + cardinality + 删除按钮）
- **画布直接删除连线**：选中连线后按 Delete → 移除 reference + dirty + PUT
- **右键菜单**：连线右键弹「删除 / 编辑 cardinality」菜单（可选，P1）

**真值表（连线交互）**：

| 交互 | 现状 | 变更后 |
|---|---|---|
| 拖拽创建关系 | 拖到目标字段松开 → 弹确认条 → 点「创建」落账 | 拖到目标字段松开 → **直接落账**（无确认条） |
| 点击连线 | 无反应 | 弹 `ReferenceDetail` 模态（显示详情 + 删除按钮） |
| 选中连线按 Delete | 无反应 | 移除 reference + dirty + PUT |
| 右键连线 | 无反应 | 弹菜单「删除 / 编辑 cardinality」（可选 P1） |

**实例推演**：
- **happy 1**：用户拖字段 A.id 到字段 B.user_id 松开 → 直接落账 reference（cardinality 自动推导 1:N）→ 画布显示连线 → **无确认条**
- **happy 2**：用户点击画布上某条连线 → 弹 ReferenceDetail 模态（显示「users.id → posts.user_id (1:N)」+ 「删除关系」按钮）
- **happy 3**：用户选中连线后按 Delete → 连线移除 → 画布刷新 → Inspector 清空
- **edge 1**：用户拖字段 A.id 到同一字段 A.id → 不创建（防自引用）
- **edge 2**：用户点击画布空白处（非连线）→ 不弹模态（既有行为不变）
- **edge 3**：用户删除连线但网络失败 → 显示错误「删除失败，请稍后重试」+ 连线保留（store 不变）

## 实现顺序建议（P0）

1. **定点 1 协作房间删除**（0.5 天）：
   - `RoomClient::delete_room` 方法（DELETE /rooms/{id}）
   - RoomPanel 加「删除房间」按钮（Owner 可见）
   - `ModalKind::DeleteRoom` 确认模态
   - 删除后回 rooms 页 + 列表刷新

2. **定点 3 关系连线简化**（1-2 天）：
   - 删 `RelToolState::Confirm` 状态（Dragging → 直接落账）
   - 点击连线命中检测（canvas click 消费 reference hit）
   - `ModalKind::ReferenceDetail` 模态（显示详情 + 删除按钮）
   - Delete 键删除连线（store.references 移除 + dirty + PUT）

3. **定点 2 区域/便签交互**（2-3 天）：
   - `tool-new-area` on:click（拖拽创建区域）
   - `tool-new-note` on:click（点击放置便签）
   - Inspector Area/Note 编辑面板（name/comment/color + 删除）
   - Delete 键删除 Area/Note
   - 画布渲染联动（draw_area / draw_note 已有）

**每步独立 commit**，commit message 格式 `fix(<module>): ...` 或 `feat(<module>): ...`。

## 不在范围（明确排除）

- 问题 1 纯列表入口强化（P1：键盘快捷键 L + 浮动返回按钮）
- 问题 2 PostgreSQL/MySQL 交互（P2：DB 选择 UI + dialect 分流 + 导出按 dialect）
- 问题 5 创建房间下拉说明（P1：diagram.created_at 时间戳显示）
- 右键菜单（P1：连线右键弹「删除 / 编辑 cardinality」菜单）
- 区域/便签拖拽调整大小（P1：既有拖拽逻辑复用，非本批）
- 关系连线右键菜单（P1：右键弹菜单，非本批）
- 不修改既有测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

## 验收标准

- **定点 1**：Owner 进房间 → RoomPanel 显示「删除房间」按钮 → 点击 → 弹确认模态 → 确认 → 删除 → 回 rooms 页 → 列表刷新（该房间消失）
- **定点 2**：用户点「添加区域」→ 拖拽画布框选 → 松开 → Area 落账 + 画布显示色块 + Inspector 编辑面板；用户点「添加便签」→ 点击画布 → Note 落账 + 画布显示便签 + Inspector 编辑面板
- **定点 3**：用户拖字段 A.id 到字段 B.user_id 松开 → **直接落账**（无确认条）→ 画布显示连线；用户点击连线 → 弹 ReferenceDetail 模态（显示详情 + 删除按钮）；用户选中连线按 Delete → 连线移除 + 画布刷新