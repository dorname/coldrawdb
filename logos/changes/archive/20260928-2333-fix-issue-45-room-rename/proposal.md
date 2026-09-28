# 变更提案：fix-issue-45-room-rename

> module: core | created: 2026-09-28

## 变更原因

GitHub issue #45「[BUG] 协作空间名称更改不生效」：

- AppBar 可编辑标题是 **diagram 标题**（`on_title_blur` 仅标 dirty 存 diagram），不写 `room.name`
- 协作空间名展示在只读 `room-badge`，**无内联改名入口**
- 后端 `GET/DELETE /api/v1/rooms/{roomId}` 存在，**无 PATCH/PUT 更新房间名**；`room.name` 仅在 `POST /rooms` 创建时写入
- 创建房间默认名「数据模型评审」与 diagram 标题易混淆 → 用户「改了不生效」

## 变更类型

接口级变更（新增 `PATCH /api/v1/rooms/{roomId}` 改名端点 + 前端改名入口 + 编排/用例覆盖）

## 变更范围

- 影响的需求文档：无（issue 即需求来源，验收条件进功能规格）
- 影响的功能规格：`core-S04-room-lifecycle-design.md`（房间生命周期补「重命名」权限/校验/一致性合同）
- 影响的业务场景：S04 房间生命周期（时序图补 rename 片段）
- 影响的 API：`logos/resources/api/rooms.yaml` —— `/api/v1/rooms/{roomId}` 新增 `patch`（owner only，name 1–64 校验）
- 影响的 DB 表：无（`room.name` 既有列，无 migration）
- 影响的编排测试：`core-S04-room-lifecycle.json`（补 rename 步骤：owner 改名 200 / viewer 403 / 非法名 422 / GET 回读一致）
- 影响的测试文档：`core-S04-test-cases.md`（新 UT/ST：后端校验与权限、前端入口与即时一致）
- 影响的前端：房间列表卡片菜单（重命名入口）、room-badge 展示同步
- 影响的部署方案：`core-01-deployment-plan.md`（后端镜像随 compose 重建注记；无 migration）
- 影响的 smoke 测试：无新增用例（部署后既有套件回归）

## 部署影响

- 是否需要部署：是
- 部署原因：新增后端端点 + 前端入口，需重建 compose 镜像生效
- 影响环境：本地 / 生产（compose）
- 是否涉及数据迁移：否（复用既有 `room.name` 列）
- 是否需要回滚预案：否（纯增量端点，后向兼容；回滚即回退镜像）
- 是否需要 smoke：是（部署后既有 smoke 套件回归，不新增用例）

## 变更概述

1. **后端**：新增 `PATCH /api/v1/rooms/{roomId}`（operationId `renameRoom`）——owner 专属；body `{name}` 校验 trim 后 1–64 字符（沿用 create_room 同一校验口径）；成功 200 返回更新后 RoomDetail；非 owner 403 `FORBIDDEN`；非成员 403 `NOT_A_MEMBER`；非法名 422 `VALIDATION_ERROR`；房间不存在/已归档 404。
2. **前端**：房间列表卡片菜单新增「重命名」（仅 owner 可见），弹窗输入新名 → PATCH → 列表即时刷新；编辑器内 `room-badge` 名称来源于 room detail，重进/刷新即为新名（AppBar diagram 标题语义不变，不与 room.name 关联）。
3. **验收对齐 issue 建议**：明确重命名入口；改名后列表/徽章即时一致且刷新保持；改 diagram 标题不影响 room.name；viewer 不可改；非法长度有校验提示。
