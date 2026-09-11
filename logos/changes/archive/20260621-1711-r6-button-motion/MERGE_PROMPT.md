# 合并指令

## 变更提案
- 提案名称：r6-button-motion
- 提案目录：logos/changes/r6-button-motion/

## 提案内容

# 变更提案：r6-button-motion

> module: core | created: 2026-06-21

## 变更原因

R5 完成后，界面美学诊断 **R6** 仍待落地：按钮缺少 `:focus-visible` 焦点环与 `primary-active` 按压反馈；Inspector / IO Drawer 打开时无 spring 风格入场；`styles.css` 存在 **两处 `@keyframes cdb-pulse` 定义**（保存圆点 opacity 版 ~L524 与 E6 Issues 徽章 scale 版 ~L2004），后者覆盖前者导致保存中圆点动画行为不确定。

## 变更类型

设计级 + 代码级（CSS 动效与微交互，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-0c-motion.md` — R6 按钮 focus/active + 面板 spring 入场 + pulse 命名拆分
  - `core-07-design-tokens.md` — `--cdb-easing-spring` / `--cdb-shadow-focus` Token
- 影响的测试用例：`core-PE-design-system-test-cases.md` — UT-R6 断言
- 影响的业务场景：无行为变更（纯视觉/可访问性）
- 影响的 API / DB / 编排测试：无

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否
- 是否涉及数据迁移：否

## 变更概述

**R6 按钮微交互 + 面板 spring 入场**：

1. **Focus Ring**：`.cdb-btn` / `.cdb-tool-btn` / `.cdb-tab--icon` 增加 `:focus-visible` + `var(--cdb-shadow-focus)`。
2. **Primary 按压态**：`.cdb-btn--primary:active` 使用 `--cdb-color-primary-active` + `translateY(1px)`。
3. **Spring 入场**：`.cdb-inspector`、`.cdb-main.cdb-has-io-drawer .cdb-io-drawer`、`.cdb-app-bar__overflow-menu` 使用 `--cdb-easing-spring` 的 slide/fade 动画。
4. **修复 pulse 冲突**：保存圆点改用 `@keyframes cdb-pulse-opacity`；E6 Issues 徽章保留 `@keyframes cdb-pulse`（scale）。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/r6-button-motion/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md

- Delta 文件：`logos/changes/r6-button-motion/deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/r6-button-motion/deltas/test/core-PE-design-system-test-cases.md`
- 目标目录：`logos/resources/test/`
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
   git add -A && git commit -m "docs(r6-button-motion): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive r6-button-motion`。
