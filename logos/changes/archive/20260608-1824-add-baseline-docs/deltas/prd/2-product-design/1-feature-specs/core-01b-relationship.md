## ADDED — 关系编辑规格

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
> 对齐参考源：drawdb `Relationship.jsx`

# 关系编辑规格（V1）

## 1. 关系（Relationship）对象结构

```ts
interface Relationship {
  id: string;
  name: string;              // 关系名（可选；如 "user_posts"）
  startTableId: string;      // 起点表 id
  startFieldId: string;      // 起点字段 id
  endTableId: string;        // 终点表 id
  endFieldId: string;        // 终点字段 id
  cardinality: "one_to_one" | "one_to_many" | "many_to_one" | "many_to_many";
  onUpdate: "CASCADE" | "RESTRICT" | "SET NULL" | "NO ACTION" | "SET DEFAULT";
  onDelete: "CASCADE" | "RESTRICT" | "SET NULL" | "NO ACTION" | "SET DEFAULT";
}
```

## 2. 关系类型语义

| cardinality | SQL 表达（以 MySQL 为例） | 含义 |
|---|---|---|
| `one_to_one` | `FOREIGN KEY (...) REFERENCES ... UNIQUE` | 1 : 1（双向唯一） |
| `one_to_many` | `FOREIGN KEY (...) REFERENCES ...` | 1 : N（外键在 N 端） |
| `many_to_one` | 同 `one_to_many`（方向相反） | N : 1 |
| `many_to_many` | 中间表 `link_<rel_name>` | N : N（需中间表） |

> `many_to_many` 实际实现：coldrawdb V1 在导出 SQL 时自动生成中间表 `link_<name>`（含两端外键）。中间表不在前端展示，但占用 `table_link` 关联。

## 3. 关系操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 拖拽表 A 的字段 → 表 B 的字段 | 创建 Relationship 对象；自连线 = self-reference |
| 编辑 | 双击连线 | 打开 `RelationshipInfo` 侧栏面板 |
| 改 cardinality | 侧栏下拉 | 4 选 1 |
| 改 onUpdate / onDelete | 侧栏下拉 | 5 选 1 |
| 删除 | 选中后 Delete | 从 diagram 移除 |
| 翻转 | 侧栏按钮 | 互换 start/end（不变 cardinality 含义） |

## 4. 渲染

- **连线**：贝塞尔曲线（`BezierCurve` 算法，详见 `frontend-rs/editor_render` 中的 `calc_path`）
- **端点**：箭头（drawdb 用 SVG marker；coldrawdb V1 用 canvas 自绘）
- **标签**：起点端 cardinality 标签 + 终点端 cardinality 标签
- **路径重算**：起点 / 终点移动时实时重算
- **碰撞**：连线与表头有最小间距，绕开

## 5. 校验

- 起点表 / 起点字段 / 终点表 / 终点字段必须存在
- 起点 ≠ 终点（**允许** self-reference：起点 = 终点）
- 字段类型匹配检查：V1 **不强制**（允许任意类型间建关系）
- onUpdate / onDelete 必填

## 6. 撤销 / 重做

关系的所有操作（创建 / 编辑 / 删除）都进入 `UndoRedoContext`。V1 不持久化。

## 7. 与后端实体的对账

| 前端 | 后端 | 说明 |
|---|---|---|
| `Relationship` | `reference` 表 | 1:1 映射 |
| cardinality | `reference.cardinality` 字段 | 枚举值 |
| onUpdate | `reference.on_update` | snake_case 存库 |
| onDelete | `reference.on_delete` | snake_case 存库 |
| startTableId + startFieldId | `reference.start_table_id` + `reference.start_field_id` | UUID |
| endTableId + endFieldId | `reference.end_table_id` + `reference.end_field_id` | UUID |
| many_to_many 中间表 | `table_link` 表 | 自动生成 |

## 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-R-01 | 创建 one_to_many 关系 |
| UT-R-02 | 改 cardinality 触发 SQL 重生成 |
| UT-R-03 | many_to_many 自动创建中间表 |
| UT-R-04 | 拖拽端点改变起止字段 |
| UT-R-05 | 删除关系不级联删除表 |
| ST-R-01 | 端到端：用户-文章一对多 → SQL 含正确外键 |

## 9. V1 边界

- ❌ 关系标签编辑（V1 不可编辑，drawdb 有）
- ❌ 关系颜色（V1 不可改）
- ❌ 关系线型（实线/虚线）（V1 统一实线）
- ❌ 关系的元数据（注释、tag）（V1 不支持）

## 10. 对齐参考源

- drawdb `src/components/EditorCanvas/Relationship.jsx`
- drawdb `src/components/EditorSidePanel/RelationshipsTab/RelationshipInfo.jsx`
- drawdb `src/utils/calcPath.js`（贝塞尔路径计算）
- coldrawdb `frontend-rs/src/editor_render.rs`（连线渲染）
