## MODIFIED — §5.12 R-LOD-01 与 R-VIEW-DIM-01～04

- **R-LOD-01**：恢复 `zoom <= 0.5` 的自动拓扑判档，但仅作用于 `ViewDimension::Auto`；显式 Table/Field 不随 zoom 改档。自动判档为当前 zoom 的纯函数，不恢复已退役的 ±0.03 滞回。
- **R-VIEW-DIM-01**：`ViewDimension { Auto, Table, Field }`，默认 Auto。Auto 按 zoom 自动选择详情/拓扑；Table 固定仅表头；Field 固定字段展开。切换顺序为 Auto → Table → Field → Auto。
- **R-VIEW-DIM-02/03**：右键、ToolRail、快捷键 V、FloatingControls 继续共用同一状态；自动模式指示为“维度：自动”。
- **R-VIEW-DIM-04**：`cdb.view-dimension` 支持 `auto` / `table` / `field`；旧值保持有效，缺省/非法值回落 Auto；仍不写 diagram、不进撤销栈。
- 渲染、表/字段/关系命中和悬停摘要必须调用同一带 zoom 判档函数，禁止只改绘制而保留旧命中档位。
