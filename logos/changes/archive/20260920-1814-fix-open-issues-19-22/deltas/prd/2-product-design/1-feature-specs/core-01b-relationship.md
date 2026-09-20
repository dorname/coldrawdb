# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#21（线型 + 密度降噪）、#22（出边跟随源表色）

## MODIFIED — §3.4「V1.2 不做」收口

将原句：

> **V1.2 不做**：自动绕障 routing、正交折线、手动指定出入侧的 UI（后续增强）。

替换为：

> **本版本仍不做**：自动绕障 routing、边捆绑、导入后自动整理布局、手动指定出入侧 UI。  
> **本提案（fix-open-issues-19-22 / #21）新增**：正交折线 / 直线路径类型与虚线线型（§4.3）；选中态密度降噪（§4.4）。出入侧仍由 §3.4 `pick_port_sides` 自动决定。

## MODIFIED — §4.1 关系线颜色（#22 继承语义）

将「为空时使用 `palette.relation`」替换为下列优先级（其余条款保留）：

| 优先级 | 条件 | 用色 |
|---|---|---|
| 1 | `Reference.color` 非空 | 关系显式色 |
| 2 | `Reference.color` 为空且源表（`start_table_id`）`Table.color` 非空 | **跟随源表色** |
| 3 | 二者皆空 | `palette.relation` 默认主题色 |

- 清除关系色（回 `''`）后恢复跟随源表；**不得**因改表色而静默覆盖已保存的非空 `Reference.color`
- Inspector 关系色入口标明「默认 = 跟随源表」；选中高亮规则不变

## ADDED — §4.2 出边跟随源表色（#22）

> 与 §4.1 优先级表一致；供表色变更与关系渲染共用。

| 规则 | 规格 |
|---|---|
| 定义 | 「出边」= `start_table_id` 等于该表的 `Reference` |
| 即时性 | 源表 `color` 变更后，所有 `Reference.color==''` 的出边下一帧重绘即用新表色 |
| 兼容 | 存量图行为：无表色时仍为主题默认色，与 #12 交付观感一致 |

## ADDED — §4.3 线条类型与线型（#21 C1）

| 规则 | 规格 |
|---|---|
| 数据 | `Reference.line_type`: `bezier`（默认）\| `orthogonal` \| `straight`；`Reference.stroke_style`: `solid`（默认）\| `dashed` |
| 持久化 | 前端模型 + 后端 `reference` 表（迁移 `0010_reference_line_style`）+ `diagrams.yaml` 同步；JSON 导入导出保留；SQL/DBML 降级忽略 |
| 路径 | `bezier` 保持现状三次贝塞尔；`straight` 为两端锚点直线；`orthogonal` 为水平/垂直折线（至少一折），锚点与出入侧由 §3.4 `pick_port_sides` 决定 |
| 线型 | `dashed` 使用 canvas `set_line_dash`；`solid` 实线 |
| Inspector | 选中关系后可切换线条类型与线型（`data-testid="inspector-rel-line-type"` / `inspector-rel-stroke-style`），走既有落账链路 |
| 兼容 | 缺字段反序列化为 `bezier` + `solid` |

## ADDED — §4.4 密度降噪（#21 C2）

| 规则 | 规格 |
|---|---|
| 触发 | 画布选中 ≥1 张表或 ≥1 条关系时启用 |
| 相关定义 | 与选中表相连的关系、或选中关系本身，视为相关 |
| 视觉 | 非相关关系线不透明度降低（建议 ≤ 0.25），相关线保持正常或略加粗；不删除、不改数据 |
| 无选中 | 全部关系正常不透明度 |

## MODIFIED — 7. 与后端实体的对账（追加行）

| 前端 | 后端 | 说明 |
|---|---|---|
| line_type | `reference.line_type` | `bezier`/`orthogonal`/`straight`；迁移 `0010` |
| stroke_style | `reference.stroke_style` | `solid`/`dashed`；迁移 `0010` |
| color（空） | `reference.color=''` | 渲染继承源表色（§4.1/§4.2），非直接等于主题色 |

## MODIFIED — 8. 测试用例 ID 索引（追加）

| TC ID | 描述 |
|---|---|
| UT-PB-12 | `relation_stroke_color`：显式色 > 源表色 > palette |
| UT-PB-13 | `calc_path`/`calc_path_orthogonal`：orthogonal 折线锚点消费 `pick_port_sides` |
| UT-PB-14 | 线型 dashed 时 dash 数组非空；solid 为空 |
| UT-PB-15 | 选中表时非相关线 alpha 降低、相关线不降 |
| ST-PB-07 | e2e：切换正交折线 → 保存刷新保留 |
| ST-PB-08 | e2e：表设色且关系未设色 → 出边跟表色；关系显式设色后改表色不影响该线 |
