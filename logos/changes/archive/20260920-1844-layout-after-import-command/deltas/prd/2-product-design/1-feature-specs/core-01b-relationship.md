# Delta — core-01b-relationship.md（修改）

> 模块：core | 提案：layout-after-import-command
> 关联 issue：#23（一键整理布局 / 导入后自动整理）

## MODIFIED — §3.4「本版本仍不做」收口

将原句：

> **本版本仍不做**：自动绕障 routing、边捆绑、导入后自动整理布局、手动指定出入侧 UI。  
> **本提案（fix-open-issues-19-22 / #21）新增**：正交折线 / 直线路径类型与虚线线型（§4.3）；选中态密度降噪（§4.4）。出入侧仍由 §3.4 `pick_port_sides` 自动决定。

替换为：

> **本版本仍不做**：自动绕障 routing、边捆绑、手动指定出入侧 UI。  
> **本提案（layout-after-import-command / #23）新增**：一键整理布局与导入后自动整理（§4.5）。出入侧仍由 §3.4 `pick_port_sides` 自动决定。

## ADDED — §4.5 一键整理布局（layout-after-import-command / #23）

| 规则 | 规格 |
|---|---|
| 算法 | 复用 MCP `force_directed_layout`（Fruchterman-Reingold 变体）；前端 `frontend-rs/src/layout.rs` 纯函数；**不改 MCP** |
| 参数 | `iterations=100`，`spacing=180`，随机种子固定 `42`（确定性） |
| 输入 | 当前 `Table[]`（id/x/y）与 `Reference[]`（start_table_id / end_table_id） |
| 输出 | 仅改写连通表的 `x`/`y`；其他字段不变 |
| 孤立表 | 无关联边的表位置**不变** |
| 无边 | 关系为空时原样返回，不扰动坐标 |
| 命令入口 | Command Palette Action：id=`action:layout`，label=`整理布局`，`data-testid="palette-action-layout"` |
| 落账 | 写 store → dirty → schedule_save → 重绘（与拖表同链路） |
| 导入后自动 | 导入成功且**本次导入**解析出关系非空时，自动执行一次整理；关系为空不触发 |
| 不做 | 边捆绑、手动指定出入侧、绕障 routing |

## MODIFIED — 8. 测试用例 ID 索引

追加：

| TC ID | 描述 |
|---|---|
| UT-PB-16 | 力导向：确定性 / 无重叠 / 孤立不动 |
| ST-PB-09 | Command Palette「整理布局」触发后连通表坐标变化 |
