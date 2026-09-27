# Delta — core-01-editor-canvas.md（fix-open-issues-26-33 / #27 + #30）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：
> - #27：区域创建后可调整宽高——画布边/角 resize 手柄 + Inspector 宽高编辑，落账/撤销与区域拖动同路径。
> - #30：小缩放 LOD 渲染分档——低 zoom 隐藏字段行、表名保持可读、关系线加粗，拓扑可辨。

## ADDED — §5.11 区域创建后可调整大小（fix-open-issues-26-33 / #27）

> 现状缺口：区域仅 `area_rect_from_drag` 创建一次定尺寸 + `area_drag` 整框平移；选中后无 resize 手柄，Inspector 仅 name/color/删除。本节补齐 resize 交互合同，与表侧 `feat-table-resize` 能力对齐。

| ID | 约束 |
|---|---|
| R-ARESZ-01 | 选中区域后，画布在区域矩形**四角与四边中点**渲染 resize 命中面（小手柄或热区），光标按方位显示对应 resize 光标（`nwse-resize` / `nesw-resize` / `ew-resize` / `ns-resize`）；未选中不渲染 |
| R-ARESZ-02 | 拖拽手柄期间仅更新视觉层（与 `area_drag` 同一拖拽状态机分支），`pointermove` 不落账；松手（`pointerup`）一次性写回 `area.width` / `area.height`（角拖拽同时可能写回 `x`/`y`，当拖拽西/北侧手柄时锚定对侧角/边）并进入 CommandStack（单次 Undo 可还原整次 resize），随后走既有保存/协作 OT 通道 |
| R-ARESZ-03 | 最小尺寸约束：resize 后 `width` / `height` 不得小于创建拖框阈值口径的下限（`AREA_MIN_SIZE = 10` 的防误触下限仅防创建误触；resize 下限取 `AREA_MIN_WIDTH = 40` / `AREA_MIN_HEIGHT = 30`，保证区域标题行可读）；拖拽越界时夹紧，不产生负尺寸/翻转 |
| R-ARESZ-04 | Inspector 区域表单（`inspector-area-form`）新增**宽度 / 高度**数值输入（testid `inspector-area-width` / `inspector-area-height`），与画布手柄双向一致：画布 resize 后表单显示新值；表单改值 blur 落账同样进 CommandStack 并即时反映画布；非法输入（非数 / 小于下限）按 R-ARESZ-03 夹紧 |
| R-ARESZ-05 | 只读模式（含分享只读链接、房间只读权限）不渲染 resize 手柄、Inspector 宽高输入禁用；拖动/resize 命中判定在只读下整体失效（与既有只读禁用编辑口径一致） |
| R-ARESZ-06 | resize 手势同样受 §5.9 拖动后 click 穿透抑制（R-DRAG-CLICK-04 防护范围扩展至区域 resize 手柄拖拽）；与 `hit_test_area` 的命中优先级：手柄命中面 > 区域内部拖动 > 区域外 |

**验收口径**：选中区域后拖角改变宽高，松手 PUT 落账且刷新后尺寸保持；Inspector 改宽高画布即时一致；单次 Undo 还原整次 resize；只读模式无任何 resize 入口。

## ADDED — §5.12 小缩放 LOD 渲染分档（fix-open-issues-26-33 / #30）

> 现状缺口：表多时只能缩小画布（35%～56%）做全局浏览，此时表名字发糊、字段行不可读、关系线极细，无法辨认拓扑。本节定义 LOD（level-of-detail）分档渲染合同，复用 R-PERF-07 精灵缓存的 zoom 分档机制，目标是「小缩放下仍能看清表与表的关系」。

| ID | 约束 |
|---|---|
| R-LOD-01 | LOD 档位判定纯函数（如 `lod_tier(zoom)`）：`zoom ≤ LOD_TIER1_MAX`（默认 0.5，token 登记）进入**拓扑档**；其上为**详情档**（现状渲染）。档位边界滞回 ±0.03，避免临界抖动反复重光栅 |
| R-LOD-02 | 拓扑档表卡渲染：隐藏字段行，仅渲染表头（表名 + 可选注释一行，对齐当前注释显示模式）；表头块高度可压缩（去掉字段区），卡体配色/圆角/选中态与详情档一致 |
| R-LOD-03 | 拓扑档表名字号：按世界字号提升使**屏幕有效字号 ≥ 11px**（`世界字号 = max(默认字号, 11 / zoom)`，夹紧上限避免 35% 以下过度放大）；字重 ≥ 500；不再依赖单纯位图缩放（位图缩放发糊是本 issue 主因之一） |
| R-LOD-04 | 拓扑档关系线：线宽按 `max(默认线宽, LOD_LINE_MIN_SCREEN_PX / zoom)` 补偿（`LOD_LINE_MIN_SCREEN_PX` 默认 1.5，夹紧 ≤ 4× 默认线宽），保证 35%～50% zoom 下线宽可读；选中高亮倍率（§4.4）与 LOD 补偿叠乘 |
| R-LOD-05 | 精灵缓存兼容：LOD 档位纳入 R-PERF-07 卡体指纹（跨档切换触发重光栅，同档内拖动/平移仍位块传输）；拓扑档卡体与详情档卡体缓存互不污染 |
| R-LOD-06 | 拓扑档下命中/交互不变：表/关系选中、拖动、resize（#27）与详情档一致；hover 表卡 tooltip 可见字段摘要属可选增强，不在本期 |
| R-LOD-07 | 阈值与样式参数登记到 `core-07-design-tokens.md` §15.4（`canvas.lod.*`），亮/暗主题成对 |

**验收口径**：约 24 表 / 20+ 关系的图缩放到 ≤50%，随机抽 5 对有 FK 的表，目视 3 秒内可指出连线两端；表名清晰可读；缩放回 100% 后详情档渲染与现状一致（无回归）。
