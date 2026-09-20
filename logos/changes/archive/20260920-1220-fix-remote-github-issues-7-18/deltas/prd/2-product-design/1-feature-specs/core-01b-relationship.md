# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#9（关系连线固定右出左进）、#12（关系连线颜色配置）

## ADDED — §3.4 连线路径自动选侧（fix-remote-github-issues-7-18）

> 修复 issue #9：现行 `calc_path` 写死「源字段右点出、目标字段左点进」（`editor_render.rs` 中 `x1 = from.x + TABLE_WIDTH`、`x2 = to.x`），与双侧 port 的视觉暗示不一致。

**合同**：

| 规则 | 规格 |
|---|---|
| 出入侧选择 | 按两表相对几何位置自动选择：目标表中心 x < 源表中心 x → **左出右进**；否则 → **右出左进**。选择逻辑抽为纯函数（如 `pick_port_sides(from, to) -> (FieldPortSide, FieldPortSide)`），可单测 |
| 锚点 | 出/入锚点按所选侧取字段行左缘或右缘中点（复用 `field_anchor_for_side` 口径） |
| 贝塞尔控制点 | 控制点偏移方向跟随出入侧（左出时控制点向左伸展，右出向右），不得仍按固定 +x/-x 方向 |
| 拖拽建关系 | 字段行左右两侧 port 均可作为拖拽起点与放置终点（`hit_test_field_port` 命中的 side 必须被路径计算消费） |
| 动态更新 | 已有关系在任一端表拖动后按最新相对位置重算出入侧，无需用户重建关系 |
| 稳定性 | 两侧距离相等（x 差为 0）时保持现状默认（右出左进），避免边界抖动 |

**V1.2 不做**：自动绕障 routing、正交折线、手动指定出入侧的 UI（后续增强）。

## ADDED — §4.1 关系线颜色（fix-remote-github-issues-7-18）

> 实现 issue #12 的关系侧：`Reference` 增加可选 `color` 字段。

| 规则 | 规格 |
|---|---|
| 数据 | `Reference.color: string`（默认 `''` = 未配置）；前端模型、后端 `reference` 表（迁移 `0006_reference_color`）、`diagrams.yaml`、`mcp-tools.yaml` `UpdateReferenceInput` 同步扩展 |
| 渲染 | `color` 非空时关系线（含拖拽橡皮筋预览与端点）使用该色；为空时使用 `palette.relation` 默认主题色 |
| 选中态 | 选中高亮优先级高于自定义色（选中态仍用 `palette.selected` 外环/加粗，自定义色作为基线色保留），二者不得互相覆盖导致不可辨识 |
| Inspector | 关系面板新增颜色入口（预设色板 + 清除回默认）；变更走既有落账链路（store → dirty → schedule_save） |
| 导入导出 | JSON 导出保留 `color`、导入回填；SQL / DBML 导出**降级忽略**（不改变 DDL 语义），导入 SQL/DBML 时 `color=''` |
| 兼容 | 存量图无 `color` 字段 → 反序列化默认 `''`，视觉与现状完全一致 |

## MODIFIED — 7. 与后端实体的对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Relationship` | `reference` 表 | 1:1 映射 |
| cardinality | `reference.cardinality` 字段 | 枚举值 |
| onUpdate | `reference.on_update` | snake_case 存库 |
| onDelete | `reference.on_delete` | snake_case 存库 |
| startTableId + startFieldId | `reference.start_table_id` + `reference.start_field_id` | UUID |
| endTableId + endFieldId | `reference.end_table_id` + `reference.end_field_id` | UUID |
| color | `reference.color` | 关系线颜色（`''` = 默认主题色）；迁移 `0006_reference_color` 新增 |
| many_to_many 中间表 | `table_link` 表 | 自动生成 |

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
| UT-PB-11 | 关系 `color` 非空时渲染用色取自 `Reference.color`，为空回退 `palette.relation` |
| ST-PB-05 | e2e：目标表拖到源表左侧 → 连线改为左出右进，不再绕行 |
| ST-PB-06 | e2e：Inspector 修改关系颜色 → 仅该线变色 → 保存刷新后保留 |

> 详细步骤见 `core-PB-relationship-test-cases.md`。
