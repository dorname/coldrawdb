# Delta — core-CR-canvas-test-cases.md（fix-issues-38-41-canvas-interaction）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#39 / #40 / #41

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-CR-PAN-01 | click/drag 阈值判定纯函数（#39 §5.15） | 位移 < 4px → 判定 click（清选）；≥ 4px → 判定 drag-pan（保留选中）；边界 4px 恰为 drag；Shift/框选工具激活时仍走框选口径 |
| UT-CR-FONT-01 | 标签字号倍率与钳制纯函数（#40 §5.16） | 档位循环 0.8→1.0→1.25→1.5→0.8；localStorage 读写/非法值回落 1.0；`max(世界字号 × zoom × 倍率, 10px)` 钳制：zoom 0.2 时触发下限、zoom 2.0 不封顶 |
| UT-AN-LOCK-01 | Area.locked 序列化兼容（#41 §5.17） | JSON 缺 `locked` 键 → 反序列化 locked=false（旧图兼容）；locked=true 往返序列化保持；锁定区域拖拽/resize 门控纯函数返回 no-op |

## ADDED — ST 用例（追加）

| ID | 前置与步骤 | 预期 |
|---|---|---|
| ST-CR-PAN-02 | e2e：选中表 → 空白按下拖动 ≥50px 平移 → 断言选中保留（Inspector 仍显示该表 + hl 探针 sel_table=true）→ 空白单击（无位移）→ 断言选中清空；pan 后 Ctrl+Z 对选中集语义不变 | 拖动平移保留选中（R-PAN-SEL-03）；单击清选（R-PAN-SEL-02）；框选（Shift+拖）不受影响（R-PAN-SEL-05） |
| ST-CR-FONT-01 | e2e：默认倍率基线字号 → 点 `canvas-font-scale` 切到 1.25 → 探针 label_font_scale=1.25 且表名字号放大 → reload 后倍率保持（localStorage）→ 缩放到 zoom ≤0.35 → min_font_clamped=true 且屏幕字号 ≥10px | R-FONT-01/02/04/05 全覆盖；工具栏 UI 字号不受影响（R-FONT-03 负断言） |
| ST-AN-03 | e2e：建区域 → 右键区域 → 「锁定区域」→ 拖动区域无位移、resize 手柄无响应 → 锁图标可见 → 保存 reload 仍锁定 → Inspector 复选框解锁 → 拖动恢复 | R-AREALOCK-02/03/05/06 全覆盖；锁定区域仍可选中与删除（R-AREALOCK-04） |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-CR-PAN-01 | UT | click/drag 4px 阈值判定（#39） |
| UT-CR-FONT-01 | UT | 字号倍率档位 + 10px 屏幕下限钳制（#40） |
| UT-AN-LOCK-01 | UT | Area.locked 缺省兼容 + 拖拽门控（#41） |
| ST-CR-PAN-02 | ST | 空白拖动平移保留选中 / 单击清选（#39） |
| ST-CR-FONT-01 | ST | 字号倍率循环 + 持久化 + 低 zoom 钳制（#40） |
| ST-AN-03 | ST | 区域锁定全链路 e2e + 持久化（#41） |
