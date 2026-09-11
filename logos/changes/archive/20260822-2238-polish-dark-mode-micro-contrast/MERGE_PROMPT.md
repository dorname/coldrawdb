# 合并指令

## 变更提案
- 提案名称：polish-dark-mode-micro-contrast
- 提案目录：logos/changes/polish-dark-mode-micro-contrast/

## 提案内容

# 变更提案：polish-dark-mode-micro-contrast

> module: core | created: 2026-08-21

## 变更原因

上一变更 `optimize-prototype-dark-glass-contrast`（已归档）的艺术总监审核中，73+ 组实测对比度全部通过 WCAG AA，但有 3 项非阻断优化项被明确划入「后续打磨批次」：

1. **P1 头像 initials 对比度**：`.avatar` 白字压 `#aa8cff` / `#68aef2` / 品牌渐变实测仅 2.35–2.66:1。总监建议 initials 改深色（如 `rgba(20,16,40,.85)`）或将成员色整体加深一档，rooms 卡片与 presence 条统一处理，并将「头像 initials × 各 accent 背景」组合纳入同一轮实测矩阵。
2. **P2 微字号地板**：`field-type`、`constraint` 为 9px，`table-count` / 状态栏为 10px。虽实测均 ≥5.96:1 达标，但 9px 低于常规可读下限，总监建议设计令牌设定 10px 地板。
3. **P2 invite h1 品牌色强调**：auth 页 h1 有 `<span>` 品牌青点缀（13.07:1），invite 页 h1 全白略平，建议给「推向下一版。」/「已经失效。」包一层品牌色 span，对齐视觉语言。

本提案将这三项一次性收口，延续「暗色玻璃态 + 对比度达标」的设计方向。

## 变更类型

设计级变更（仅原型文件与配套设计规格说明）。

## 变更范围

- 影响的需求文档：无（不改需求语义）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`（视觉基准/验收行补充头像与 invite 标题说明，不改交互）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`（视觉基准/验收行补充 presence 头像与微字号地板说明，不改交互）
- 影响的业务场景：S04 / S05 的页面视觉呈现
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：仅 HTML 原型文件变更，无需服务部署
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 房间卡片成员头像 initials 对比度优化（深色字压浅色填充）
  - id: invite
    prototype: core-01-editor-prototype.html
    description: 邀请页 h1 增加品牌色 span 强调，与 auth 视觉语言对齐
  - id: editor
    prototype: core-01-editor-prototype.html
    description: presence 头像 initials 对比度优化；field-type/constraint 微字号提升至 10px 地板
```

## 变更概述

本次仅修改统一主原型 `core-01-editor-prototype.html` 的视觉层细节：

1. `.avatar` initials 在暗色模式下改深色文字（候选值 `rgba(20,16,40,.85)`，以实测矩阵定稿），覆盖 rooms 卡片成员头像叠层与 editor presence 条，确保「initials × 各成员色/accent 背景」组合对比度 ≥ WCAG AA 4.5:1。
2. 微字号地板：`field-type`、`constraint` 由 9px 提升至 10px（`table-count`、状态栏已为 10px 不动），同步核对行高与截断风险。
3. invite 页 h1 第二行文案包品牌色 `<span>`（沿用 auth 页 h1 span 样式），含有效态「推向下一版。」与失效态「已经失效。」。

同步在 S04/S05 feature specs 的视觉基准/验收行补充对应说明。审核方式延续上一提案：AI 分轮产出原型调整版本，艺术总监（代理）真渲染实测反馈，直到回复「审核通过」；每轮保留修改说明。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

- Delta 文件：`logos/changes/polish-dark-mode-micro-contrast/deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md

- Delta 文件：`logos/changes/polish-dark-mode-micro-contrast/deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/polish-dark-mode-micro-contrast/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(polish-dark-mode-micro-contrast): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive polish-dark-mode-micro-contrast`。
