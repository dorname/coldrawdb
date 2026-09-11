## ADDED — 10.5 列表视图（ListView 字段明细编辑网格）

### 10.5 列表视图（ListView 字段明细编辑网格，PDManer 式）

**定位**：从只读分组清单升级为**字段明细编辑网格**（redesign-listview-type-length-canvas-fix），对标 PDManer 字段明细表格——列表视图即字段编辑主场，与画布/Inspector 同源同账。

**入口**：AppBar「列表」`[data-testid="btn-view-list"]`，全屏面板 `[data-testid="list-view-panel"]` 占据主区行。

**布局**：

- 工具条：增字段 `[data-testid="list-add-field"]`、删字段、上移、下移（后三者作用于选中字段行，未选中禁用）；右侧「返回画布」
- 网格按表分组：组头行 `表名（N 字段）`（点击选中该表，Inspector 同步）；组内字段行
- 列定义（对齐 PDManer，数据域列不做）：

| 列 | 绑定 | 编辑方式 |
|---|---|---|
| 字段代码 | `field.name` | 文本输入（blur 落账） |
| 显示名称 | `field.comment` | 文本输入（blur 落账） |
| 主键 | `field.primary` | checkbox（即改即账） |
| 不为空 | `field.not_null` | checkbox |
| 唯一 | `field.unique` | checkbox |
| 自增 | `field.increment` | checkbox |
| 数据类型 | `field.type_` 基类型 | 下拉（按引擎过滤，core-01a §2.2 基类型清单） |
| 长度 | `type_` 参数 | 数字输入，仅 VARCHAR 系可编辑，缺省 255 |
| 小数位 | `type_` 参数 | 数字输入，仅 DECIMAL/NUMERIC 系可编辑，缺省 2（精度列并入长度列口径：DECIMAL(长度,小数位)） |
| 说明 | `field.tag` | 文本输入（受控防抖，blur 落账） |

**行为**：

- 行内编辑直接落账（沿用既有 PUT 快照通路；文本类 blur/防抖落账，选择与勾选即改即账）
- 类型/长度/小数位三联：任一变更按 core-01a §2.2 参数化口径重新合成 `type_` 整串（`VARCHAR(32)` / `NUMERIC(10,2)`）
- 增字段：在选中表（或首张表）末尾追加默认字段；删字段：删除选中行（无需确认模态，可撤销）；上移/下移：字段在表内排序 ±1
- 选中行高亮（`state.listSel` 语义：{table_id, field_id}）；双击组头/行 → 切回画布并选中该表（保留 ST-SP-LIST-01 既有语义）
- 空表 → 空态提示，不布局塌陷

**V1 边界**（明确不做）：

- ❌ 主题域树 / 多表 Tab 条 / 逻辑实体 / 多表透视 / 数据字典（PDManer 左树体系）
- ❌ 数据域列、列设置、表格式编辑、入库/标注工具
- ❌ 分组模式切换（固定按表分组；原「不分组/ByTag」与批量类型面板由行内编辑取代）
