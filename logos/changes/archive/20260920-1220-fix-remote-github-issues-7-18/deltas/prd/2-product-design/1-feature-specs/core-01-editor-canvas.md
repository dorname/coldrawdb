# Delta — core-01-editor-canvas.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#11（尺寸变化失真）、#17（拖拽高亮直角）、#10（画布注释展示，渲染侧口径）

## ADDED — §11.5 画布尺寸变化同步（V1.2，fix-remote-github-issues-7-18）

> 修复 issue #11：现行 canvas 缓冲尺寸与 DPR transform 仅在 `frame_tick` 驱动的渲染 effect 内同步；分隔条拖动只写 CSS 变量（`splitter.rs`），窗口/侧栏尺寸变化无监听，旧位图被拉伸直到下一次交互才校正。

**问题陈述补充**（R-DPR 缺口）：

| 项 | 现状 | 期望（V1.2） |
|---|---|---|
| 容器尺寸变化监听 | 无 `ResizeObserver`、无 `window.resize` 监听 | canvas 父容器挂 `ResizeObserver`；尺寸或 dpr 变化即同步 backing store 并 `schedule_paint` |
| 分隔条拖动 | 只写 CSS 变量，不触发重绘 | 拖动中与松手后画布持续清晰（经 ResizeObserver 统一触发，或 splitter 显式通知） |
| 同步时机 | 下次交互才校正 | 尺寸变化的同一帧（rAF 内）完成 `canvas.width/height` + `set_transform` 同步，无拉伸帧 |

**实现约束**：

| ID | 约束 |
|---|---|
| R-DPR-07 | canvas 父容器必须挂 `ResizeObserver`（或等价机制）：回调内按最新 `client_width/height × 有效 dpr` 更新 backing store 并触发 `schedule_paint`；与 R-PERF-05 共用 rAF 合并，一帧至多一次 `draw_canvas` |
| R-DPR-08 | 分隔条（`splitter.rs`）拖动路径不得绕过上述同步：优先由 ResizeObserver 自然覆盖；若浏览器兼容兜底需要，splitter 可在 `pointermove`/`pointerup` 显式调 `schedule_paint` |
| R-DPR-09 | 同步逻辑与 R-PERF-10 降采样共存：拖拽活跃期有效 dpr 仍为 `capped_drag_dpr` 结果；ResizeObserver 触发的重算必须使用同一 `effective_device_pixel_ratio()` 来源 |
| R-DPR-10 | 组件卸载时断开 ResizeObserver（`Drop` / cleanup），不得泄漏监听 |

**验收口径**：连续拖动 Inspector/ListView 分隔条的过程中与松手后，文字与连线始终清晰；窗口最大化/还原、侧栏展开收起后无需点击画布即恢复清晰；100% 与非 100% 缩放、普通 DPI 与高 DPI 均通过。

## ADDED — §5.7 选中与拖拽高亮外形（V1.2，fix-remote-github-issues-7-18）

> 修复 issue #17：表体圆角 14px，但 R-PERF-11 拖拽幽灵层用 CSS `outline` 画高亮——`outline` 不贴合 `border-radius`，拖动时呈直角。

| ID | 约束 |
|---|---|
| R-HL-01 | 单表选中态与拖拽幽灵层的高亮描边必须与表外形一致：圆角描边（表体半径 14px，外扩高亮环半径 16px，与 `draw_table_selection` 口径一致），可外扩 1–2px |
| R-HL-02 | 幽灵层实现不得使用 CSS `outline` 画高亮；改用 `border` + `border-radius` 或幽灵 canvas 内 `round_rect` 描边 |
| R-HL-03 | 多选包围盒（框选多表）允许直角矩形；单表路径必须圆角，二者不混用 |
| R-HL-04 | 左右中点手柄位置不因圆角描边错位；非 100% 缩放下圆角平滑（走 §11 DPR 同源路径） |

## ADDED — §5.8 表/字段注释与颜色渲染（V1.2，fix-remote-github-issues-7-18）

> 实现 issue #10 的画布渲染侧与 issue #12 的表/关系渲染侧；交互入口与显示开关见 `core-01a-table-and-field.md` §1.5/§1.6，关系侧见 `core-01b-relationship.md` §4.1。

| ID | 约束 |
|---|---|
| R-CMT-01 | `Table.comment` 非空且显示模式含注释时，表头在表名下方（或同行右侧）渲染注释文本，字号小于表名、颜色取 `palette` 次要文本色；超出单行省略，hover tooltip（title）显示完整 comment |
| R-CMT-02 | `Field.comment` 非空且显示模式含注释时，字段行在类型文本旁/后渲染注释（灰色、单行省略 + title 全文）；字段行宽度内按 名称 > 类型 > 注释 的优先级截断 |
| R-CMT-03 | comment 为空时不渲染、不留占位，行高/表高与现状一致；注释参与 R-PERF-07 精灵指纹（内容变化触发重光栅） |
| R-CMT-04 | 显示模式三态：`name`（仅英文名）/ `name+comment`（默认，用户已拍板）/ `comment`（仅注释）；模式为画布级视图设置，切换即重绘，不落库到 diagram 数据 |
| R-COLOR-01 | 表边框色：`Table.color` 非空时表边框/选中外环基色跟随 `table.color`（现状仅表头渐变使用）；为空保持 `palette.table_border` |
| R-COLOR-02 | 关系线按 `Reference.color` 渲染（合同见 `core-01b` §4.1）；默认 `palette.relation` |
| R-COLOR-03 | 自定义色在暗/亮主题下与选中高亮并存时可辨识；选中态外环仍用 `palette.selected` |

**验收口径**：有 comment 的表/字段在默认模式下画布可见中文注释；空 comment 无占位噪音；导入导出（JSON）后 comment 与颜色保留，画布刷新仍可见。
