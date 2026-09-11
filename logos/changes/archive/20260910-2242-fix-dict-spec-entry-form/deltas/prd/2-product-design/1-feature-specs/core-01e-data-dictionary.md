# Delta — core-01e-data-dictionary.md（字典入口形态对齐：ToolRail + 抽屉）

## MODIFIED — 2. 字典 Tab（侧栏）

## 2. 字典面板（ToolRail 入口 + 抽屉）

入口：ToolRail 新增「数据字典」按钮（testid `toolrail-dicts`），点击开合右侧 **DictPanel 抽屉**（testid `dict-panel`，`dict-panel-close` 关闭）。

> 形态说明：生产前端侧栏 Tab 体系（`LeftPanel`/`SidePanelTab`）未上线（死代码，从未实例化），线上布局为 AppBar + ToolRail + Canvas + Inspector 抽屉；字典入口以主原型 `core-01-editor-prototype.html` 的 ToolRail + 抽屉形态为准，不落入 core-04 的侧栏 Tab 体系。

### 2.1 列表项

每项：字典名称 + 编码 + 字典项数量 + **引用计数**（绑定该字典的字段数）。

### 2.2 操作

| 操作 | 触发 | 效果 | testid |
|---|---|---|---|
| 单击 | 鼠标左键 | 展开/收起字典详情（`dict-detail-{id}`：名称/编码/说明内联编辑 + 字典项列表，见 §2.3） | `dict-item-{id}` |
| 名称/编码/说明编辑 | 详情内联输入，blur 落账 | 名称/编码空值或编码冲突 → 红标并阻断落账；编码改名同步改写所有引用字段（§3，同一次 undo 事务） | `dict-name-{id}` / `dict-code-{id}` / `dict-comment-{id}` |
| 删除 | 列表行删除按钮 | 未被引用 → 直接删除（可撤销）；被引用 → 弹确认（§2.4） | `dict-del-{id}` |
| "+" | 面板底部 | 新建字典（默认 `dict_N` / `code_N`） | `dict-add` |
| 导出 | 面板头部 | 导出数据字典 Markdown（§4） | `dict-export` |

### 2.3 字典项编辑（详情面板）

- 表格行：`value | label` 两列行内编辑（blur 落账，`dict-item-value-{iid}` / `dict-item-label-{iid}`）+ 行尾删除按钮（`dict-item-del-{iid}`）；底部「+ 添加项」`dict-item-add-{dict_id}`（新项 value 默认取下一个空闲数字，label 默认「新项 N」）
- 上移/下移调整 `sort`（`dict-item-up-{iid}` / `dict-item-down-{iid}`，作用于选中行）
- 校验（实时，行内红标 `dict-item-dup-{iid}`）：value 非空且字典内唯一；label 非空

### 2.4 校验与引用一致性

- 字典名/编码非空；编码同图内唯一（冲突时红标 + 阻断落账）
- **删除被引用字典**：引用计数 > 0 时弹确认模态（`confirm-dict`，`confirm-dict-ok` / `confirm-dict-cancel`）「被 N 个字段引用，删除后这些字段的绑定将置空，是否继续？」；确认后所有 `dict_code = 该编码` 的字段置空（同一次 undo 事务）
- 删除未被引用字典：无需确认，可撤销

## MODIFIED — 5. 协作与只读

- S05 OT：字典/字典项/绑定编辑均为模型 op，随既有 op 通路广播与合并，**协议零变更**；远端 op 到达后字典面板与 Inspector 摘要即时刷新
- S02 分享只读：字典面板可浏览、可导出（ToolRail 入口不禁用），编辑控件禁用
- S06 MCP：本变更不扩展 MCP 工具；`dictionaries` 随 diagram JSON 读写自然透出
