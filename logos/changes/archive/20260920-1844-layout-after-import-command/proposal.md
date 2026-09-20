# 变更提案：layout-after-import-command

> module: core | created: 2026-09-20
> 关联：https://github.com/dorname/coldrawdb/issues/23（#21 后续）

## 变更原因

#21 已交付线型与密度降噪；剩余「导入后 / 命令面板一键整理布局」跟踪于 #23。MCP 侧已有确定性 `force_directed_layout`（`layout_diagram`），生产前端尚未暴露等价能力，高密度导入图仍需手工拖表。

## 变更类型

设计级变更（功能规格 + 测试 + 代码；无 API/DB 变更——复用前端 store 坐标落账）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `core-01b-relationship.md`（取消「不做导入后自动整理布局」；新增 §4.5 一键整理）
  - `core-0a-code-editor.md` §7 Command Palette（新增 Action「整理布局」）
  - `core-01-editor-canvas.md`（命令入口交叉引用，如需）
- 影响的业务场景：S01
- 影响的测试用例：
  - `core-PB-relationship-test-cases.md` — UT-PB-16 / ST-PB-09
  - `core-CR-canvas-test-cases.md` — UT-CR-LAYOUT-01
- 影响的 API / DB / 编排 / smoke：无（MCP `layout_diagram` 已存在，本变更不改 MCP）

## 部署影响

- 是否需要部署：否
- 部署原因：本地可验证；生产由后续 release 执行
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: editor
    prototype: core-01-editor-prototype.html
    description: "Command Palette 增加「整理布局」；导入含 FK 后可自动跑力导向一次"
```

## 变更概述

1. 前端移植 MCP 同款 Fruchterman-Reingold 变体（确定性种子 42，iterations=100，spacing=180），对 `Table`/`Reference` 纯函数改写 x/y；孤立表不动；无边时原样返回。
2. Command Palette 增加 Action「整理布局」（id=`action:layout`，`data-testid="palette-action-layout"`），执行后写 store → dirty → schedule_save → 重绘。
3. 导入成功且本次导入关系非空时，自动执行一次整理（与手动入口共用 `force_directed_layout`）。
4. UT-PB-16 / UT-CR-LAYOUT-01：确定性、无重叠、孤立不变；ST-PB-09：palette 触发后坐标变化。
5. **不做**：边捆绑、手动指定出入侧、改 MCP。

## 产品拍板

1. 算法与 MCP `force_directed_layout` 对齐（iterations=100, spacing=180, seed=42）。
2. 导入后自动整理：仅当本次导入解析出关系非空时触发。
3. 不做边捆绑 / 手动指定出入侧 UI。
