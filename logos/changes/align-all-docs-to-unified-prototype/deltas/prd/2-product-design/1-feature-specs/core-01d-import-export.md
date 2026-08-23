# Delta — core-01d-import-export.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。IO 入口与格式以主原型「更多菜单 → 导入/导出抽屉」为准。

| 项 | 约定 |
|---|---|
| 页面流 | IO 仅在 `room-editor`（及 EmptyGuide 引导）触发；不依赖独立历史原型 |
| 演示 ≠ 生产 | 主原型导入为本地解析模拟；剪贴板/下载以 Toast 或本地 blob 演示。生产导入走 bridge API，导出客户端生成 |
| 实现状态 | bridge / 导出能力**后端或客户端已具备**；**生产前端部分接入**；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity` |

## MODIFIED — 2. 组件树

```
AppRoot
├── AppBar
│   └── 更多菜单 (btn-more-menu)
│         ├── btn-import  → ImportDrawer
│         └── btn-export  → ExportDrawer
├── ToolRail / Command Palette（可命令打开同一抽屉）
├── IoDrawer / SideSheet
│   ├── ImportDrawer  data-testid="import-drawer"
│   └── ExportDrawer  data-testid="export-drawer"
└── EmptyGuide → guide-import-sql → ImportDrawer
```

**主路径变更**：导入/导出**不再**以 AppBar 常驻 pill 为默认；统一经 **更多菜单**（与主原型 `renderMoreMenu` 一致）。`btn-import` / `btn-export` testid 保留在菜单项上。

## ADDED — 统一原型对齐补充：4 / 5 格式

导入与导出均支持 **SQL / DBML / JSON** 三分段（`import-format-tabs` / `export-format-tabs`）：

| format | 导入 | 导出 |
|--------|------|------|
| SQL | 粘贴/拖入；引擎选择（生产）；解析摘要 | 客户端 SQL 预览 + 复制/下载 |
| DBML | 粘贴/拖入；表块计数 | DBML 预览 |
| JSON | 合法 JSON + tables | persistence 同构 JSON |

抽屉标题副文案对齐主原型：「SQL · DBML · JSON」。

## MODIFIED — 3.1 互斥规则

### 3.1 互斥规则

- 打开 Import / Export 抽屉时折叠 Inspector（`data-testid="inspector"`），关闭时恢复。
- 与成员抽屉 / 活动抽屉互斥（同一 `drawer` 槽位，主原型一次只开一个）。
- Code View 打开时不叠加 IO 抽屉为默认路径。

## ADDED — §9.x 边界补充

- ❌ 将主原型「模拟导入」标为已完成生产 bridge 接线
- ❌ 以 V1 Import 模态为默认用户路径（模态仅保留回归 UT）
- ✅ 更多菜单 → 导入/导出抽屉；SQL/DBML/JSON
- ✅ Viewer：导入提交与改写 diagram 的导出下载策略按房间角色；只读角色不得提交导入写库（与 `canEdit` 对齐，具体权限见 S04）
