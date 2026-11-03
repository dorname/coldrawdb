# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：fix-issues-70-73 | 关联 issue：#73
> 目标：补齐生产关系检查器的名称编辑入口；不改变关系数据格式、API 或 DB Schema。

## ADDED — §3.5 关系检查器名称编辑（#73）

| 合同项 | 规格 |
|---|---|
| 入口 | 在生产 `inspector-reference-form` 顶部增加文本输入 `data-testid="inspector-ref-name"`，作为首个可编辑控件，位于 cardinality 等其他编辑控件之前。 |
| 原始值 | 输入框 `value` 必须绑定当前关系的原始 `r.name`，不得把统一显示名写成 value。 |
| 占位提示 | `placeholder` 使用 `relation_display_name(r)`；当原始名称为空串或仅含空白时，显示 `未命名关系（<id>）`。placeholder 仅作提示，不得自动落账为关系名称。 |
| blur 规范化 | 仅在 blur 时提交；对输入值执行 `trim()` 后写入。输入为空或仅含空白时写入空串，表示清除名称。 |
| 同值不写 | 将 trim 后的值与当前原始 `r.name` 比较；相同则不更新 store、不置 dirty、不调用 `schedule_save`，不得产生保存请求或协作 op。 |
| 保存链 | `on_update_ref_field` 增加 `name` 分支，将值写入目标 `Reference.name`，随后复用现有 `store.references.set → store.dirty.set(true) → schedule_save` 链路；不新增 REST 端点或独立保存按钮。 |
| 只读 | Viewer、分享只读及房间只读权限下输入框 disabled；`on_update_ref_field` 入口现有只读判断继续作为第二道保护，回调被直接触发时也不得写入。 |
| 显示联动 | 名称变化后，所有既有 `relation_display_name` 消费方使用新名称；清空后统一回退为 `未命名关系（<id>）`，不得引入 `ref-<id>` 等其他兜底格式。 |
| 变更边界 | 不扩展 UndoRedo，不改造旧 Relationships Tab 搜索或命令面板空名展示，不改变线旁标签的既有布局与显隐规则。 |

## MODIFIED — §9 V1 边界 / 关系名称与标签

将「关系标签编辑（V1 不可编辑）」修订为：

- 关系名称可通过 Inspector 的 `inspector-ref-name` 编辑并持久化；线旁关系标签仍为只读展示，不提供画布内直接编辑入口。

## MODIFIED — §8 测试用例 ID 索引（追加）

| TC ID | 描述 |
|---|---|
| UT-PB-38 | `relation_display_name`：具名关系显示原始名称；空串与纯空白名称回退 `未命名关系（<id>）` |
| ST-PB-22 | e2e：关系检查器真实编辑、trim、同值不写、保存重载、列表/标签联动、清空与只读禁用 |

> 详细步骤见 `core-PB-relationship-test-cases.md`。
