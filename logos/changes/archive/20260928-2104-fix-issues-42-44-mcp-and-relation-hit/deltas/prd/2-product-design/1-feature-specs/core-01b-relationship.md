# Delta — core-01b-relationship.md（fix-issues-42-44-mcp-and-relation-hit）

## ADDED — §4.7 关系线命中合同：线型同源 + 最近距离优先（fix-issues-42-44-mcp-and-relation-hit / #44）

**背景**：#44 命中错乱双根因——①`hit_test_reference_tier` 按 refs 数组序先命中即返回（非最近优先）；②命中几何恒为贝塞尔，忽略 `line_type`（orthogonal 折线绘制与命中不同源）。

合同条款：

- **R-HIT-01 线型同源**：命中检测几何必须与绘制几何同口径——`line_type=bezier` 用 `calc_path_tier` 贝塞尔（24 段折线近似）；`line_type=orthogonal` 用 `calc_orthogonal_path_tier` 折线顶点序列；tier（详情/拓扑）锚定策略一致（沿用 #35 R-LOD-08）。
- **R-HIT-02 最近距离优先**：命中带宽 8px（世界坐标，覆盖 7px 光晕 + 2px 主线）内若有多条关系命中，必须返回**点到线距离最小**的那条；禁止「数组序先命中即返回」。
- **R-HIT-03 阈值外不命中**：所有关系距离均超阈值时必须返回 None——点击空白不得误选远处关系（与 §5.15 空白点击清选语义衔接）。
- **R-HIT-04 悬停/点击同口径**：点击选中与悬浮检测（§4.8）共用同一命中函数，保证「所见即所选」。
- **R-HIT-05 命中优先级不变**：关系线命中仍排在表命中之后（连线被表遮住时点击选中表）；便签/区域/字段端口优先级不变。

验收：UT-PB-18 / UT-PB-19；e2e ST-PB-11。

## ADDED — §4.8 悬浮关系线摘要 tooltip（fix-issues-42-44-mcp-and-relation-hit / #43）

**背景**：#43——字段维度下关系线密集、路径较长时，悬浮线段缺少「哪张表哪个字段 ↔ 哪张表哪个字段」的快速确认手段。

交互规格：

- **R-HOV-01 触发**：pointermove 且未处于任何拖拽态（表/关系/便签/区域/框选/pan/端点拖拽）时，对指针世界坐标执行 §4.7 命中；命中关系线即显示 tooltip。
- **R-HOV-02 内容**：单行摘要 `源表名.源字段名 → 目标表名.目标字段名`；表维度（拓扑档）下至少 `源表名 → 目标表名`（字段锚定仍存在时同样展示字段）。
- **R-HOV-03 位置**：tooltip 跟随光标右下偏移（约 12px），不遮挡 Inspector；纯 DOM 浮层，不进画布绘制层。
- **R-HOV-04 无副作用**：悬停不改变选中态、不触发保存、不进撤销栈；点击仍按既有逻辑选中关系。
- **R-HOV-05 消失条件**：指针移出命中带宽、进入任意拖拽态、或命中目标变为非关系图元时消失。
- **R-HOV-06 只读/Viewer**：tooltip 同样可用（只读信息展示不受编辑权限限制）。

验收：UT-PB-20（文案纯函数）；e2e ST-PB-12（悬浮出 tooltip、选中态不变）。

## MODIFIED — 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-R-01 | 创建 one_to_many 关系 |
| UT-R-02 | 改 cardinality 触发 SQL 重生成 |
| UT-R-03 | many_to_many 自动创建中间表 |
| UT-R-04 | 拖拽端点改变起止字段 |
| UT-R-05 | 删除关系不级联删除表 |
| ST-R-01 | 端到端：用户-文章一对多 → SQL 含正确外键 |
| UT-PB-01 | `hit_test_field` 命中字段行 |
| UT-PB-02 | `build_reference` 默认 RESTRICT |
| UT-PB-03 | `flip_reference_endpoints` 互换端点 |
| UT-PB-04 | `toggle_field_primary` 单表唯一 PK |
| UT-PB-05 | 确认条可见，点 create 后 `references.len()+1` |
| UT-PB-06 | 位移 < 4px 判定为点击；≥ 4px 判定为拖线 |
| UT-PB-07 | 橡皮筋路径起点为源字段锚点、终点为指针坐标 |
| ST-PB-01 | e2e：关系工具点击两点 + 确认 → Inspector 可编辑关系 |
| ST-PB-02 | e2e：关系工具从字段拖到另一字段 + 确认 → 新增 1 条 reference |
| UT-PB-09 | `pick_port_sides` 按相对位置选侧（目标在左 → 左出右进；在右 → 右出左进；x 相等 → 默认） |
| UT-PB-10 | `calc_path` 消费选侧结果：锚点与贝塞尔控制点方向随侧变化 |
| UT-PB-11 | 关系 `color` 非空时渲染用色取自 `Reference.color`，为空回退源表色 / `palette.relation` |
| ST-PB-05 | e2e：目标表拖到源表左侧 → 连线改为左出右进，不再绕行 |
| ST-PB-06 | e2e：Inspector 修改关系颜色 → 仅该线变色 → 保存刷新后保留 |
| UT-PB-12 | `relation_stroke_color`：显式色 > 源表色 > palette |
| UT-PB-13 | `calc_path`/`calc_path_orthogonal`：orthogonal 折线锚点消费 `pick_port_sides` |
| UT-PB-14 | 线型 dashed 时 dash 数组非空；solid 为空 |
| UT-PB-15 | 选中表时非相关线 alpha 降低、相关线不降 |
| ST-PB-07 | e2e：切换正交折线 → 保存刷新保留 |
| ST-PB-08 | e2e：表设色且关系未设色 → 出边跟表色；关系显式设色后改表色不影响该线 |
| UT-PB-16 | 力导向：确定性 / 无重叠 / 孤立不动 |
| ST-PB-09 | Command Palette「整理布局」触发后连通表坐标变化 |
| UT-PB-17 | 批量 line_type 命令纯函数与单次 Undo |
| ST-PB-10 | e2e：「应用到全部关系」+ 命令面板 |
| UT-PB-18 | 命中合同：阈值内最近距离优先；全超阈值返回 None（#44 R-HIT-02/03） |
| UT-PB-19 | 命中合同：线型同源——orthogonal 关系按折线几何命中，不按贝塞尔（#44 R-HIT-01） |
| UT-PB-20 | 悬浮 tooltip 文案纯函数：`源表.源字段 → 目标表.目标字段`；拓扑档 `源表 → 目标表`（#43 R-HOV-02） |
| ST-PB-11 | e2e：密集近平行线连续点击不同线段，选中与点击目标一致；两线中间空白不误选（#44） |
| ST-PB-12 | e2e：悬浮关系线出 tooltip 且选中态不变；移出消失（#43） |

> 详细步骤见 `core-PB-relationship-test-cases.md`。
