# Delta: core-S04-room-lifecycle-design.md

> 提案：fix-canvas-zoom-invite-comment-resize（问题 2：协作邀请链接无效）

## MODIFIED — ### S04.2 邀请成员

### S04.2 邀请成员

**入口**：编辑器 AppBar `[data-testid="btn-invite"]` 或房间设置页

**Invite 模态** `[data-testid="modal-invite"]`：

1. 选择角色：editor / viewer（owner 不可通过邀请创建）
2. 生成链接：`{public_base_url}/invite/{token}` `[data-testid="invite-url"]`。`public_base_url` 取后端 `PUBLIC_BASE_URL` 配置，缺省由请求 Host header（含 scheme 与非默认端口）推导；链接必须是**可公开访问的完整 URL**，**禁止硬编码 `localhost` / `127.0.0.1` / 无端口占位 host**——被邀请人跨机器打开该链接必须可直接访问
3. 可选：输入邮箱发送（依赖 S03 用户存在或 pending 注册）
4. 「复制链接」→ Toast；链接 7 天有效（过期后 UI 灰显 + 重新生成）

**成员列表面板** `[data-testid="room-members-panel"]`（SideSheet 或 Inspector 附加 Tab）：

- 每行：头像 + display_name + role Tag + 在线点（S05 实装 presence）
- owner 行：role 下拉 disabled
- owner 对其他成员：role 下拉（editor ↔ viewer）+ 「移除」

## MODIFIED — ##### 正常：复制邀请链接

##### 正常：复制邀请链接

- **GIVEN** 用户在 room 内且 role=owner
- **WHEN** 用户点击 `[data-testid="btn-invite"]`，选择 role=editor，点击复制
- **THEN**
  - `[data-testid="invite-url"]` 为完整 URL，匹配 `^{scheme}://{host}(:{port})?/invite/{token}$`，host/port 与当前部署实际对外地址一致（含非默认端口）
  - 链接不含硬编码 `localhost` / `127.0.0.1`；被邀请人在另一台机器上打开该链接可直接访问邀请页
  - Toast「链接已复制」
