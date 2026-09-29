## MODIFIED — §5.12 表/字段维度与可读性补偿（fix-open-issues-26-33 / #30；fix-issues-34-35 / #35 注释补齐 + 表级锚定；fix-issues-36-37 / #36 显式维度化；fix-issue-48-topology-card-readability / #48 表维度卡宽上限放宽）

> 演进：#30 引入 zoom ≤ 0.5 隐式切换的 LOD 分档；#35 补齐拓扑档注释与表级锚定；#36 将「进入哪个维度」从 zoom 阈值手中收回——维度由用户显式切换（R-VIEW-DIM），zoom 回归纯导航/可读性职责，仅在表维度内驱动字号/线宽补偿。原「拓扑档/详情档」措辞统一为「表维度/字段维度」。#48 发现表维度下英文表名 + 中文注释并排常被截断，原因是表维度与字段维度共用 480px 上限，而表维度字号经补偿后变大，因此为表维度单独放宽上限。

| ID | 约束 |
|---|---|
| R-LOD-01 | （#36 修订）**zoom 阈值触发废止**：渲染档位不再由 `lod_tier(zoom)` 判定；档位 = 显式维度模式（R-VIEW-DIM-01：Table → 表维度渲染，Field → 字段维度渲染）。`LOD_TIER1_MAX` 阈值与 ±0.03 滞回退役（token 勘误见 §15.4） |
| R-LOD-02 | 表维度表卡渲染：隐藏字段行，仅渲染表头（表名 + 可选注释一行，对齐当前注释显示模式）；表头块高度可压缩（去掉字段区），卡体配色/圆角/选中态与字段维度一致。（#35 注记：注释副文本——字号 = 表维度表名字号 × 0.8、字重 ≥ 500，前景按 R-CMT-CONTRAST 双端点判据决策，宽度沿用注释感知估算，超长截断） |
| R-LOD-03 | 表维度表名字号补偿：zoom 仅作**维度内可读性补偿**——按世界字号提升使屏幕有效字号 ≥ 11px（`世界字号 = max(默认字号, 11 / zoom)`，夹紧上限避免过度放大）；字重 ≥ 500；字段维度任意 zoom 不补偿（默认字号）。补偿不触发维度切换 |
| R-LOD-04 | 表维度关系线线宽补偿：按 `max(默认线宽, LOD_LINE_MIN_SCREEN_PX / zoom)`（`LOD_LINE_MIN_SCREEN_PX` 默认 1.5，夹紧 ≤ 4× 默认线宽）；字段维度任意 zoom 不补偿；选中高亮倍率（§4.4）与补偿叠乘 |
| R-LOD-05 | 精灵缓存兼容：维度模式纳入 R-PERF-07 卡体指纹（维度切换触发重光栅，同维度内拖动/平移仍位块传输）；表维度卡体与字段维度卡体缓存互不污染 |
| R-LOD-06 | 表维度下命中/交互不变：表/关系选中、拖动、resize（#27）与字段维度一致；hover 表卡 tooltip 可见字段摘要属可选增强，不在本期 |
| R-LOD-07 | 补偿参数登记到 `core-07-design-tokens.md` §15.4（`canvas.lod.*`），亮/暗主题成对 |
| R-LOD-08 | 表维度关系线**表级锚定**（#35）：字段行隐藏时，关系线两端锚点收敛到表级端口——纵坐标 = 表头纵向中线（`table.y + TABLE_HEADER_HEIGHT / 2`），横坐标 = 卡体左/右缘（按当前 `comment_mode` 下的有效宽计算），选侧沿用 §3.4 `pick_port_sides` 并传入当前 `comment_mode`（bezier / orthogonal / straight 三线型同口径）；同一表多条关系线在表级端口汇聚；切回字段维度恢复字段维度锚定。crow's foot 端点记号随表级锚点绘制（fix-issue-49）。 |
| R-LOD-09 | **表维度表头高随表名字号补偿自适应**（#48）：拓扑档表头高 = `max(TABLE_HEADER_HEIGHT, scaled_label_font_world(lod_table_font_size(zoom, tier), zoom, label_scale).0 + 2 × 留白)`，与 R-WIDTH-07 同源，确保表维度下放大表名后不被截断。 |
| R-LOD-10 | **表维度卡宽上限单独放宽**（fix-issue-48-topology-card-readability / #48）：`resolve_table_width_for` 在 `LodTier::Topology` 下使用 `TABLE_WIDTH_MAX_TOPOLOGY`（默认 640px），字段维度维持 `TABLE_WIDTH_MAX`（480px）。表维度下单行表名+注释的估算宽度放宽到该上限后再截断，减少中英文并排长文本被 `…` 截断；关系锚点随放宽后的宽度同源取缘（R-WIDTH-08 / #49），不得错位。 |
| R-VIEW-DIM-01 | 显式维度模式 `ViewDimension { Table, Field }`：Table = 表维度（仅表头，R-LOD-02 渲染），Field = 字段维度（字段展开，原详情档渲染）；**默认 Field**；任何 zoom 下维度保持不变——缩放控件只负责缩放 |
| R-VIEW-DIM-02 | 三入口切换（状态一致）：① 画布/表**右键菜单**「切换为表维度 / 切换为字段维度」项；② **ToolRail 维度按钮**（`cdb-is-active` 激活态可见，只读/Viewer 下仍可用——纯视图切换）；③ **快捷键 `V`**（无修饰键，沿用工具快捷键门控链：编辑器页 / 非文本输入目标 / 无浮层；只读下可用） |
| R-VIEW-DIM-03 | 状态指示：FloatingControls 常驻「维度：表 / 维度：字段」按钮（对齐「注释：…」先例），点击同样切换；三入口与指示同一信号驱动，状态永远一致 |
| R-VIEW-DIM-04 | 持久化：localStorage 键 `cdb.view-dimension`（值 `table` / `field`），非法/缺省回落 Field；视图偏好**不落库** diagram snapshot、不参与保存链路（同 R-CMT-04 口径） |
| R-VIEW-DIM-05 | 切换即重绘：维度经 R-LOD-05 纳入精灵指纹；维度为纯视图状态，不进 CommandStack（undo/redo 不改变维度） |

**验收口径**：表维度下缩放 35%～200% 始终保持仅表头（字段行不出现）；字段维度下任意缩放始终保持字段展开；滚轮缩放永不触发维度切换；右键菜单 / ToolRail / 快捷键 `V` 三入口切换后 FloatingControls 指示与渲染一致；刷新页面后维度保持；表维度小缩放下字号/线宽补偿、中文注释可见与表级锚定（#35 合同）保持有效；表维度下典型中英文表卡（英文名 10+ ASCII、注释 8+ CJK）在 100% zoom 下完整可读或仅极长文本尾部截断。
