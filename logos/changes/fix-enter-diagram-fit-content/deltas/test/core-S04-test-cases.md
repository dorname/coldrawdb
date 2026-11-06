# Delta — core-S04-test-cases.md（fix-enter-diagram-fit-content）

> 模块：core | 场景：S04/S04.1A | 关联：进入已有图视口不定位到内容（空画布反馈）

## ADDED — ST-S04-UI-19 — 进房加载后表在视口内可见（自动 fit）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **前置**：mock `GET /diagrams/{id}` 返回表坐标远离原点的图（如 (4000, 3000) 附近多表），rooms/协作 mock 同 ST-S04-UI-14 口径
- **步骤**：`/rooms` 点开该图所在房间进入 room-editor
- **断言**：进房后无需任何缩放/平移操作，首张表节点在画布视口内可见（bounding box 与画布容器相交）；缩放显示 ≤100%；空图（0 表）回归场景保持原点不报错
- **reporter**：登记 `ST-S04-UI-19`

## ADDED — ST-S04-UI-20 — 悬浮控件「适应画布」按钮

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：进入远坐标图房间 → 手动平移使画布呈空 → 点悬浮控件 `btn-fit-canvas`
- **断言**：按钮存在且文案「适应画布」（对齐主原型 fit-canvas）；点击后表重新出现在视口内；不影响保存链路（无 PUT/WS op 副作用）
- **reporter**：登记 `ST-S04-UI-20`

## 变更记录追加

| TC ID | 变更 | 说明 |
|-------|------|------|
| ST-S04-UI-19/20 | ADDED（fix-enter-diagram-fit-content） | 进房自动适应内容 + 适应画布按钮 e2e |
