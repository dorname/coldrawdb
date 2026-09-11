# 变更提案：optimize-prototype-dark-glass-contrast

> module: core | created: 2026-08-21

## 变更原因

统一主原型 `core-01-editor-prototype.html` 当前视觉风格需要进一步向暗色调、玻璃态（glassmorphism）方向优化，以提升产品质感。但暗色背景与玻璃叠加容易造成组件、字体对比度不足，影响可读性和无障碍体验。本次变更在保留双区布局、页面流和交互行为的前提下，对全页面进行暗色+玻璃态重绘，并确保关键文字、按钮、表单错误、状态标签与艺术总监审核通过的对比度标准一致。

## 变更类型

设计级变更（仅原型文件与配套设计规格说明）。

## 变更范围

- 影响的需求文档：无（不改需求语义）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`（仅更新色彩/字体/对比度说明，不改交互）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`（同上）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`（同上）
- 影响的业务场景：S03 / S04 / S05 的页面视觉呈现
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
  - id: auth
    prototype: core-01-editor-prototype.html
    description: 登录/注册页暗色玻璃态优化，确保表单字段、错误提示、tab 对比度清晰
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 房间列表页暗色玻璃态优化，确保卡片、空状态、用户菜单对比度清晰
  - id: invite
    prototype: core-01-editor-prototype.html
    description: 邀请接受页暗色玻璃态优化，确保 preview、按钮对比度清晰
  - id: editor
    prototype: core-01-editor-prototype.html
    description: 协作编辑器暗色玻璃态优化，确保 tool-rail、inspector、presence、状态徽章对比度清晰
```

## 变更概述

本次仅修改统一主原型 `core-01-editor-prototype.html` 的视觉层：引入系统化的暗色色板（背景、表面、悬浮、强调、错误、成功）、玻璃态卡片与顶栏（半透明背景 + 背景模糊 + 细腻边框）、优化字体层级与字重、确保所有文字与背景对比度达到 WCAG AA 以上。同步在相关 feature specs 中更新设计 token 与对比度说明。

审核方式：由 AI 分轮产出原型调整版本，用户以「艺术总监」身份给出反馈，AI 持续迭代直到用户回复「审核通过」。每一轮调整后保留修改说明，便于追溯。
