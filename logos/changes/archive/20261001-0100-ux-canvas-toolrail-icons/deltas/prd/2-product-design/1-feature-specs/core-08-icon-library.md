## MODIFIED — §4 编辑器图标清单（ToolRail）

统一替换左侧 ToolRail 图标为更简洁、语义清晰的线型 SVG，保持 `data-testid` 与交互不变：

| 图标组件 | 用途 | 视觉描述（更新后） | 挂载位置 |
|---|---|---|---|
| `IconPan` | 平移画布 | 手形轮廓 | ToolRail `tool-pan` |
| `IconMarquee` | 框选多表 | 虚线矩形 | ToolRail `tool-marquee` |
| `IconAddTable` | 新建表 | 表格网格 + 右下角加号 | ToolRail `tool-add-table` |
| `IconAddArea` | 添加区域 | 矩形 + 四角支架 | ToolRail `tool-new-area` |
| `IconAddNote` | 添加便签 | 折角便签 + 横线 | ToolRail `tool-new-note` |
| `IconRelationshipCreate` | 创建关系 | 两个方块 + 连线 + 加号 | ToolRail `tool-relationship-create` |
| `IconRelationshipSelect` | 选中关系 | 两个方块 + 连线 + 对勾 | ToolRail `tool-relationship-select` |
| `IconDictionary` | 数据字典 | 两页书本/字典 | ToolRail `toolrail-dicts` |
| `IconSearch` | 搜索与命令 | 放大镜 | ToolRail `tool-search` |
| `IconActivity` | 协作动态 | 心跳折线 | ToolRail `tool-activity` |
| `IconSettings` | 画布设置 | 齿轮 | ToolRail `tool-settings` |

> 新增 `IconMarquee`（框选）与 `IconDictionary`（字典）两个独立组件；`IconSelect` 保留为备用选择指针，ToolRail 框选入口改用 `IconMarquee`。

**R1 验收**：`editor_panels.rs` ToolRail 全部使用上述 SVG 组件，不再使用 Emoji/Unicode 占位。
