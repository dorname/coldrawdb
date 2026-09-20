# Delta — core-01-editor-canvas.md（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#19（区域拖动松手误打开邀请模态）、#20（表宽不自适应）

## ADDED — §5.9 拖动后 click 穿透抑制（#19）

> 修复典型 pointer capture 松手后 click 穿透：区域/表/便签有效拖动松手后，合成 `click` 不得打开 AppBar「邀请」等顶栏控件。

| ID | 约束 |
|---|---|
| R-DRAG-CLICK-01 | 当次 pointer 手势被判定为**有效拖动**（位移 ≥ `DRAG_THRESHOLD`，与既有表/关系阈值一致）时，必须抑制紧随其后的合成 `click`（含冒泡到 `btn-invite` / 其他 AppBar 可点击控件），不得打开 `modal-invite` 或切换页面 |
| R-DRAG-CLICK-02 | 未超过阈值的按下-松开（点击选中）**不得**误伤：仍可正常触发选中与顶栏点击 |
| R-DRAG-CLICK-03 | `pointerup` 落账逻辑完成前，不得以「提前 `release_pointer_capture` + 松手坐标命中顶栏」的方式把 click 交给 AppBar；推荐：落账后再释放 capture，或 document 级一次性 click suppress，或拖动期间顶栏 `pointer-events: none` |
| R-DRAG-CLICK-04 | 防护范围：区域拖动、表拖动、便签拖动、多选整组拖动；创建工具拖框（Area create）若位移有效，同样抑制穿透 click |

**验收口径**：协作编辑器中向上拖动区域至「邀请」按钮中心后松开 → `modal-invite` 不出现；区域坐标已更新。

## ADDED — §5.10 表卡内容自适应宽度（#20）

> 补齐 `TABLE_WIDTH` 硬编码缺口；与 R-CMT-01/02「单行省略」并存——自适应先算最小可读宽，再在该宽内省略。

| ID | 约束 |
|---|---|
| R-WIDTH-01 | `table.width == None` 或 `Some(0)` 表示 **auto**：渲染宽按当前注释显示模式下表名/字段名/类型/注释文本测量，取各行需求最大值，夹在 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`（`TABLE_WIDTH=230`，`TABLE_WIDTH_MAX=480`，对齐 ListView 列宽上限） |
| R-WIDTH-02 | `table.width` 为正整数时尊重用户手动宽度（Set Table Width / 拖拽改宽），不因内容自动覆盖 |
| R-WIDTH-03 | `compute_table_render_size`、hit-test、关系锚点、精灵缓存指纹必须共用同一有效宽，禁止绘制宽与命中宽分叉 |
| R-WIDTH-04 | 导入或新建表且宽度为 auto 时，允许一次性计算并可选写回 `table.width`（持久化 fit）；之后用户改名为更长文本时，auto 模式应在下次重绘按内容再算（不强制每次写库） |
| R-WIDTH-05 | Set Table Width 文案「0 = auto」语义必须与 R-WIDTH-01 一致（禁止把 `0` 当成 0px 窄表） |

**验收口径**：长表名（如 `asset_v2.virtualization_cluster_profile`）在 auto 下表头完整可辨（或至少显著宽于 230px 且逼近测量宽）；手动设宽后不再被内容强制撑开。

## MODIFIED — §5.8 R-COLOR-02 交叉引用

将 R-COLOR-02 的「空 color → palette.relation」语义改为：空 `Reference.color` 时的解析合同见 `core-01b-relationship.md` §4.1 / §4.2（可继承源表色）。本文件不重复定义继承优先级。
