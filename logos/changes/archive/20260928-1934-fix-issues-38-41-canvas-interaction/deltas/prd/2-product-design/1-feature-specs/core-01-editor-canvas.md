# Delta — core-01-editor-canvas.md（fix-issues-38-41-canvas-interaction）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#38 / #39 / #40 / #41
> 目标：新增 §5.14～§5.17 四节交互合同。

## ADDED — §5.14 视口操作与撤销栈解耦（#38）

| 合同 ID | 条款 |
|---|---|
| R-PAN-UNDO-01 | pan（空白拖动平移）与 zoom（滚轮/按钮缩放）为**纯视图操作**：不产生 Command、不进撤销栈、不清空撤销/重做栈。 |
| R-PAN-UNDO-02 | pan/zoom 完成后，Ctrl+Z / Ctrl+Y（及 Ctrl+Shift+Z）必须按历史栈正常撤销/重做最近一次图元操作；快捷键可用性与视口交互次数无关。 |
| R-PAN-UNDO-03 | pan 手势的 pointerdown/move/up 全链路不得 panic / 不得击穿 document 级 keydown 监听器；pointer capture 必须在 pointerup/pointercancel 释放（含非常规退出兜底）。 |
| R-PAN-UNDO-04 | 回归锚点：ST-KB-UNDO-01（建表 → 空白拖动 pan → Ctrl+Z 撤销建表 → Ctrl+Y 重做）必须常绿。 |

## ADDED — §5.15 空白拖动平移保留选中（#39，方案 A）

| 合同 ID | 条款 |
|---|---|
| R-PAN-SEL-01 | 空白 pointerdown **不再立即清空选中**；选中清空延迟到 pointerup 判定。 |
| R-PAN-SEL-02 | pointerup 时指针位移 < 4px（纯 click）→ 清空单选与多选集（既有「点空白取消选中」语义不变）。 |
| R-PAN-SEL-03 | 位移 ≥ 4px（拖动）→ 视口平移（既有 pan 行为）且**保留**全部选中（单选 + 三多选集）；Inspector 内容持续对应选中对象。 |
| R-PAN-SEL-04 | 命中图元（表/字段/关系/便签/区域）的 pointerdown 分支不变：按图元拖动/选中语义，不触发 pan。 |
| R-PAN-SEL-05 | 框选手势不变：Shift+空白拖动 / 框选工具激活时空白拖动 = 框选（起点仍清选），不与 pan 混淆。 |
| R-PAN-SEL-06 | 只读（Viewer/分享只读）下空白拖动同样平移且保留选中（纯视图操作）。 |
| R-PAN-SEL-07 | 平移不破坏多选集合：pan 后 Delete / Ctrl+Z 等对保留的选中集语义与 pan 前一致（与 §5.13 R-KBSEL 系列正交）。 |

## ADDED — §5.16 画布标签字号（#40）

| 合同 ID | 条款 |
|---|---|
| R-FONT-01 | 新增「标签字号倍率」本机偏好 `label_font_scale ∈ {0.8, 1.0, 1.25, 1.5}`，默认 1.0；localStorage `cdb.label-font-scale` 持久化；**不写入图表文档**（协作不同步）。 |
| R-FONT-02 | 入口：FloatingControls 循环按钮 `data-testid="canvas-font-scale"`（文案「字号：{档位}」），点击循环 0.8→1.0→1.25→1.5→0.8；只读可用（纯视图操作）。 |
| R-FONT-03 | 生效范围：画布内表名、字段名、注释文本（`draw_table` / 拓扑档表头 / 注释副文本）；生效方式 = 既有字号 × 倍率，与 zoom 正交相乘；不影响工具栏 / Inspector 等非画布 UI。 |
| R-FONT-04 | 屏幕空间最小字号钳制：任意 zoom 下，表名屏幕像素字号 ≥ 10px（`max(世界字号 × zoom × 倍率, 10)`）；字段名/注释同下限。钳制仅在缩小时生效，放大方向不封顶。 |
| R-FONT-05 | 探针：`__cdb_lod_probe` 增补 `label_font_scale`（当前倍率）与 `min_font_clamped`（本帧是否有文本触发下限钳制）。 |
| R-FONT-06 | 与 §5.12 显式维度正交：倍率/钳制在两个维度档位下都生效；表维度表名可读性合同（R-LOD-02 系列）继续满足。 |

## ADDED — §5.17 区域锁定（#41）

| 合同 ID | 条款 |
|---|---|
| R-AREALOCK-01 | `Area` 新增 `locked: bool`（diagram JSON 缺省 false，旧图兼容）；前端 struct、后端 `AreaDto`（serde default）、DB `area.locked` 列（migration 0012_area_lock，INTEGER NOT NULL DEFAULT 0）三层一致。 |
| R-AREALOCK-02 | 入口：a) 区域右键菜单项「锁定区域 / 解锁区域」（画布右键菜单在命中区域时追加该项）；b) Inspector 区域表单「锁定」复选框 `data-testid="inspector-area-locked"`。两入口状态一致。 |
| R-AREALOCK-03 | 锁定态行为：区域拖动 no-op；resize 手柄不响应（不显示或显示禁用态）；多选整组拖动跳过锁定区域（其余选中图元正常移动）。 |
| R-AREALOCK-04 | 锁定不阻止：选中、Inspector 查看/改名/改色、删除（Delete 键与按钮均允许，与「防误拖」目标一致）。 |
| R-AREALOCK-05 | 视觉提示：锁定区域边框右上角渲染锁形标记（canvas 绘制，选中态与非选中态均可见）。 |
| R-AREALOCK-06 | 持久化：locked 随 PUT 快照通路落库，reload 后保持；协作房间经既有 area op / doc_json 透出，协议零变更。 |
| R-AREALOCK-07 | 只读（Viewer）下锁定/解锁入口禁用（锁定是文档属性变更，非视图操作）。 |
