# Delta — core-S05-ot-collab-design.md（修改）

> module: core | proposal: polish-dark-mode-micro-contrast
> 仅补充 presence 头像对比度与微字号地板的视觉说明，不改变任何交互语义、OT 行为、状态不变量与测试锚点。
> 实现载体为主原型 `core-01-editor-prototype.html`，对应 delta：`deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（房间内编辑器 + 实时同步层） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（模拟双端：本地编辑 + 远端 op / 光标 / 重连） |
| 历史参考 | `core-05-ot-collab-prototype.html`（ToolRail 控件未完整绑定，不再作为验收入口） |
| 生产实现 | 后端 collab REST/WS、OT 持久化与编排已实现；`frontend-rs` WS/OT/presence 尚未接入 |
| 视觉基准 | 在统一协作编辑器上叠加 presence、远端光标、连接态 Banner 与演示控制台；Dark 模式采用高对比度暗色色板（以主原型 `html[data-mode="dark"]` token 组为准：`--bg:#050f13`、`--surface` 不透明度 .86、文字层级 `--text:#f2fdfe` / `--text-2:#b8d2d8` / `--text-3:#86a3ab`），tool-rail、inspector、presence 列表、状态徽章与 Banner 文字对背景对比度均 ≥ WCAG AA 4.5:1；presence 头像 initials 对浅色填充统一深色字 `rgba(20,16,40,.85)`（dark 模式全部头像；light 模式内联浅色填充头像，渐变深底头像保持白字）；微字号地板 10px（field-type、constraint 由 9px 提升） |
| 痛点关联 | **P03**——消除「邮件传 JSON + 手动 merge」；多人同时改 schema 可收敛 |

## MODIFIED — 8. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 进入协作编辑器 | 房间编辑器 + WS 模拟器显示已连接 |
| 「模拟 Alice 创建表」 | 画布出现 orders + Activity 条目 |
| 「模拟 Alice 光标」 | 远端光标移动 |
| 「模拟断线重连」 | Banner 流程 + rev 更新 |
| 「模拟重连失败」 | 降级 Banner |
| 「Viewer 模式」 | 只读 + 仍可见远端 op |
| 切换 dark 主题 | `data-mode` 切换；tool-rail 图标、inspector 字段、presence 头像与用户名、连接态 Banner（含 banner--danger）、画布便签（canvas-note）、远端光标名字牌（remote-label）与诊断项（diagnostic）在暗色玻璃背景下对比度 ≥ WCAG AA 4.5:1，远端光标与本地选中框保持可区分；presence 头像 initials 深色字压各成员色填充 ≥ 4.5:1；field-type / constraint 以 10px 渲染且无截断 |

`core-05-ot-collab-prototype.html` 中 ToolRail 控件未完整绑定，只用于历史视觉对照，不纳入现行修复与验收。
