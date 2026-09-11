## ADDED — 7. redesign-listview-type-length-canvas-fix：便签/区域拖动 + tag 受控输入（2026-09-07）

> 修复三个交互 bug：note/area 拖拽状态机缺分支（命中只选中）、tag 输入静态快照 + blur 重建丢字、高频 signal set × 同步全量重绘卡死。

### ST-CR-NOTE-01 — 便签拖动落账 + 编辑不卡死（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-note` → 点击画布放置 → mousedown 便签中心 → 拖动 ≥60px → mouseup → Inspector 编辑内容连续键入一段文本
- **预期**：拖动中便签跟随（视觉层）；mouseup 后 PUT 落账 `notes[0].x/y` 更新为拖后坐标；连续键入不丢字（`inspector-note-content` 最终值与键入一致）；全程无长时间帧阻塞（拖动 mousemove 不触发 PUT）

### ST-CR-AREA-01 — 区域拖动落账（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-area` → 拖框创建区域 → mousedown 区域内部 → 拖动 ≥60px → mouseup
- **预期**：mouseup 后 PUT 落账 `areas[0].x/y` 更新为拖后坐标（`on_update_area` x/y 写回通道）；宽高不变；Inspector 区域表单仍选中

### ST-CR-TAG-01 — Inspector 字段 tag 受控输入连续键入不丢字（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：选中表字段 → Inspector `inspector-field-tag` 连续键入 `finance-core`（逐字 input 事件）→ blur
- **预期**：输入过程中值不被重建打断（每击键后输入框值单调增长）；blur 落账 PUT 快照 `fields[i].tag=="finance-core"`；再次聚焦时值保持

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| ST-CR-NOTE-01 | 便签拖动落账 + 编辑不丢字 | e2e test-spec-parity-d |
| ST-CR-AREA-01 | 区域拖动落账（on_update_area x/y 通道） | e2e test-spec-parity-d |
| ST-CR-TAG-01 | Inspector tag 受控输入不丢字 | e2e test-spec-parity-d |
