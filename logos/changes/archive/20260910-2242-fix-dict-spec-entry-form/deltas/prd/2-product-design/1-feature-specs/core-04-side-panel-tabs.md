# Delta — core-04-side-panel-tabs.md（字典入口移出侧栏 Tab 体系登记）

## MODIFIED — 7A. Dictionary Tab（数据字典，V1 仅前端 state）

## 7A. 数据字典（ToolRail 入口 + 抽屉面板，随图表持久化）

> 详细规格见 `core-01e-data-dictionary.md`。本节仅登记入口位置与摘要。
> 形态说明：数据字典**不在**本文件的侧栏 Tab 体系内——生产前端 `LeftPanel`/`SidePanelTab` 未上线（死代码），字典入口为 ToolRail 按钮 `toolrail-dicts` + 右侧 DictPanel 抽屉（`dict-panel`），以主原型 `core-01-editor-prototype.html` 为准。
> 持久化口径：字典经 migration 0007（`diagram.dictionaries` + `field.dict_code`）随图表持久化，与 Enums/Types「仅前端 state」口径不同。

### 7A.1 列表项

每项：字典名称 + 编码 + 字典项数量 + 引用计数（绑定该字典的字段数）。

### 7A.2 操作

| 操作 | 触发 | 效果 |
|---|---|---|
| 单击 | 鼠标左键 | 展开/收起字典详情（名称/编码/说明内联 blur 编辑 + 字典项 value/label 行内编辑） |
| 删除 | 列表行删除按钮 | 未被引用直接删除；被引用弹确认模态 |
| "+" | 面板底部 | 创建新字典（默认 `dict_N` / `code_N`） |
| 导出 | 面板头部 | 导出数据字典 Markdown |

> 引用一致性：删除被引用字典前提示「被 N 个字段引用，删除后绑定置空，是否继续？」（口径同 Enums Tab 引用检查）

## MODIFIED — 10.5 列表视图（ListView 表树 + 单表字段网格，PDManer 式）— V1 边界

**V1 边界**（明确不做）：

- ❌ 主题域树 / 多表 Tab 条 / 逻辑实体 / 多表透视（PDManer 左树体系——本变更的表树仅为表级扁平列表，不含主题域分组层级）
- ❌ 数据域列、列设置、表格式编辑、入库/标注工具
- ❌ 跨表全量网格模式（「全部表」虚拟节点）/ 分组模式切换 / 批量类型面板

> 注：「数据字典」自 S07（feat-data-dictionary）起纳入，入口形态为 ToolRail `toolrail-dicts` + DictPanel 抽屉（非本文件侧栏 Tab 体系，见 §7A 与 `core-01e-data-dictionary.md`），不再列为排除项；ListView 字段网格「说明」列仅追加只读字典映射摘要，绑定编辑仍在 Inspector。

## MODIFIED — 11. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-SP-01 | 单击 Tables Tab 表项 → 画布高亮 + 滚动 |
| UT-SP-02 | 搜索 "user" → 列表过滤只含 user* |
| UT-SP-03 | 双击枚举名 → 进入重命名 |
| UT-SP-04 | 引用检查：删除被引用的枚举 → 弹确认 |
| UT-SP-05 | Issues Tab：表名重复 → 错误项出现 |
| UT-SP-06 | Issues Tab：单击错误 → 跳转 + 画布闪烁 |
| UT-SP-07 | DBML Editor：编辑后 Apply → 解析成功 → Diagram 更新 |
| UT-SP-08 | DBML Editor：编辑非法 DBML → 错误消息 + 不应用 |
| ST-SP-01 | 端到端：编辑 5 表 → Issues Tab 显示 0 error |
| UT-SP-09 | 6 业务 Tab 切换（点击 Tab A→B→C，验证激活态 + 内容区切换）— B2 范围 |
| UT-SP-10 | 全局搜索跨 Tab 过滤（spec §10，搜索框过滤 Tables/Areas/Enums 等多 Tab 列表）— B2 范围 |
| UT-SP-11 | 字典面板开合（ToolRail `toolrail-dicts` + DictPanel 抽屉）+ 字典 CRUD/绑定/导出（详细用例见 `core-01e-data-dictionary.md` §8） |
