# 合并指令

## 变更提案
- 提案名称：redesign-phase-e-design-system-migration
- 提案目录：logos/changes/redesign-phase-e-design-system-migration/

## 提案内容

# 变更提案：redesign-phase-e-design-system-migration

> module: core | created: 2026-06-15
> 前置：`redesign-phase-d-command-code` 已归档（Phase D 规格 6 spec + 1 prototype + 1 test cases 全部入 archive/，代码 0% 未实现）；Phase A/B/C 已落地为 V2 布局（AppBar + ToolRail + Inspector + IO 抽屉）。

## 变更原因

V2 重规划 Phase A/B/C 已完成业务逻辑与布局重构，但 drawdb-web 的视觉系统仍停留在自写 CSS 阶段，呈现以下问题：

1. **主色已对齐但色阶缺失**：`--cdb-color-primary: #175e7a` 与 main `defaultBlue` 一致，但 Semi Design 完整色阶（hover / active / disabled / soft tint）未翻译；hover/active 状态靠临时硬编码实现。
2. **图标库严重缺失**：drawdb-web 工具栏 / 表头 / 字段类型徽章 / Inspector 操作项普遍使用 unicode 字符或 emoji，跨浏览器 / 跨字号 / 暗色模式下表现不一致；main 用 `@douyinfe/semi-icons` 2000+ SVG 统一视觉。
3. **组件库散落实现**：Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet 等 8 类核心 UI 模式在 `editor_panels.rs` 与 `styles.css` 中以 `cdb-` 前缀散落实现，行为差异大、视觉不统一；main 用 Semi Design 25+ 组件统一封装。
4. **代码视图降级方案**：Phase D 决定 V1 用 `<textarea readonly>`，但 main `CodeEditor` 用 Monaco + DBML setup + 复制按钮，体验差距大；Monaco 引入会增大 WASM 体积但对开发体验显著提升。
5. **无暗色模式**：main 通过 `settings.mode === "dark"` 全局切换（`darkBgTheme = "#16161A"`），drawdb-web 仅 light。
6. **无动效**：main 用 framer-motion + Semi 内置 transition；drawdb-web 模态 / 抽屉 fade 缺失，hover / active 无过渡。

本提案分 **E1–E6 六个批次** 完成设计系统迁移，每批次独立可 merge，最终实现「V2 业务逻辑 + main 视觉语言 + Rust WASM 性能」的完整产品形态。

## 变更类型

**设计级 + 代码级**（六批次中：E1–E3 为视觉与组件规格 + 同步代码；E4 引入 Monaco 二进制依赖；E5–E6 为 token 扩展与微交互）

## 变更范围

### 影响的功能规格

- `core-00-information-architecture.md` — §1 顶层布局 z-index 体系扩展（Palette / Tooltip / Popover / Drawer 层级）、§3 暗色模式 token 切换说明
- `core-01-editor-canvas.md` — §5.2 token 引用统一替换为 E1 后的扩展名；§6 Inspector 组件对齐 E3 Button / Tag / Icon
- `core-01a-table-and-field.md` — 表头 / 字段类型徽章用 E2 图标替换 emoji
- `core-01b-relationship.md` — 关系工具 Tooltip / Popover 用 E3 组件
- `core-01c-index-enum-custom-type.md` — Tag / Collapse 用 E3 组件
- `core-01d-import-export.md` — 抽屉头部 Button / 复制按钮 Icon 用 E2/E3
- `core-04-side-panel-tabs.md` — Tool Rail 图标全量替换为 E2 SVG；Issues 徽章用 E3 Badge
- `core-05-top-menu-modals.md` — AppBar 按钮 / Dropdown 菜单 / Modal 全部对齐 E3
- **新增** `core-07-design-tokens.md` — E1 完整设计 token 规格
- **新增** `core-08-icon-library.md` — E2 图标库规格
- **新增** `core-09-core-components.md` — E3 核心组件规格（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet）
- **新增** `core-0a-code-editor.md` — E4 Monaco 集成规格（含 DBML setup 与复制按钮）
- **新增** `core-0b-dark-mode.md` — E5 暗色模式规格
- **新增** `core-0c-motion.md` — E6 动效规格
- `core-01-editor-prototype.html` — 原型视觉对齐 E1/E2/E3 后的 token / 图标

### 影响的业务场景

- S01（编辑保存）— 选中态/Inspector 视觉变化（业务语义不变）
- S02（加载分享）— 模态视觉变化
- core-CR / KB / PB / PC / UI-modals / SP 各 test cases — 选择器 / 视觉断言需要适配新组件类名

### 影响的部署方案

- 无（E4 除外：Monaco 增大 WASM 体积，部署方案需要增加 lazy-load 与浏览器缓存策略说明，但不产生新的部署任务）

### 影响的 API

- 无

### 影响的 DB 表

- 无

### 影响的编排测试

- 无（drawdb-web core 模块 `skip_phases: [api, database, scenario]`，无编排测试）

### 影响的 smoke 测试

- HP-01~HP-05 选择器需要适配 E2/E3 后的新组件类名
- 新增 HP-06 暗色模式切换（E5 后）
- 新增 HP-07 模态/抽屉动效（E6 后）
- 新增 HP-08 Monaco 加载（E4 后）

## 部署影响

- 是否需要部署：**否**（纯前端 WASM 增强，仍走现有 staging 部署流程）
- 部署原因：E 系列不引入后端 / 数据库 / 编排变更
- 影响环境：无（部署方案不需要重写）
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**（无 schema / API 变更，CSS 与组件回滚成本极低）
- 是否需要 smoke：**是**（HP-01~HP-05 适配 + 新增 HP-06/HP-07/HP-08）

> **注**：虽然 smoke_required=true，但本提案不创建 `[deploy]` section——smoke 任务在 verify 阶段由 `openlogos verify` 自动执行，不在 tasks.md 中。

## 变更概述

### Phase E 六批次（每批次独立可 merge）

| 批次 | 主题 | 主要产物 | Monaco 依赖 | smoke 增量 |
|---|---|---|---|---|
| **E1** | Design Tokens 补全 | `--cdb-color-*` 完整色阶（hover/active/disabled/tint）、阴影层级、动效时长 token、字体/字号 token；新增 `core-07-design-tokens.md` | 否 | 无 |
| **E2** | SVG 图标库 | 从 `@douyinfe/semi-icons` 精选 ~50 个核心图标（`IconAddTable` / `IconEdit` / `IconDelete` / `IconMore` / `IconUndo` / `IconRedo` / 等）转 Leptos 组件；新增 `core-08-icon-library.md` | 否 | HP-01~HP-05 选择器适配 |
| **E3** | 核心组件重写 | Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet 八个组件对齐 Semi 视觉；新增 `core-09-core-components.md` | 否 | HP-01~HP-05 选择器适配 |
| **E4** | Monaco 代码视图 | `monaco-editor` + `monaco-editor-wasm` 集成、DBML setup、复制按钮（参考 main `CodeEditor` `absolute right-6 bottom-2 z-10`）、lazy load 策略；新增 `core-0a-code-editor.md`；**为 Phase D Code View 实现铺路** | **是** | HP-08 Monaco 加载 |
| **E5** | 暗色模式 | `settings.mode` 全局 token 切换、`darkBgTheme = "#16161A"` 浅/深 token 映射、localStorage 持久化；新增 `core-0b-dark-mode.md` | 否 | HP-06 暗色模式切换 |
| **E6** | 动效与微交互 | 模态/抽屉 fade+slide（200/300ms）、hover/active 过渡曲线、Issues 徽章 pulse；新增 `core-0c-motion.md` | 否 | HP-07 动效断言 |

### Phase D 与 E 的衔接（不在本提案范围，明确边界）

- Phase D 规格 6 spec + 1 prototype + 1 test cases 已 archive 入库（不重开 Phase D 提案）
- Phase D 代码（CommandPalette / Code View）未实现
- **E1–E3 完成后**，Command Palette 用 E3 Modal + E2 图标实现，Code View 走 E4 Monaco
- **不再单独产出 Phase D 实现批次**——Palette 与 Code View 视为 E 提案的隐性收尾，按各批次增量交付

### 不在 Phase E 范围（明确 V1/V2/V3 边界）

- Mermaid / PNG / PDF 导出（main 已有，drawdb-web V1 不做）
- 模板市场 / Bug Report 页面（drawdb-web V1 不做）
- i18n 多语言切换（main 完整支持，drawdb-web V1 中文单语）
- 协同编辑（main 无，drawdb-web V1 不做）

## 关键决策（已与用户对齐）

1. **Monaco 引入策略**：E4 引入 `monaco-editor` + `monaco-editor-wasm`（wasm-bindgen 绑定），**接受 WASM 体积膨胀 ~30MB**，通过 lazy-load（首次进入 Code View 时按需加载）缓解
2. **图标库自建**：从 main `@douyinfe/semi-icons` 复制 SVG 路径到 Leptos 组件，**不引入** `leptos-icons` 等第三方库（避免版本漂移）
3. **批次执行顺序**：默认 E1 → E2 → E3 → E4 → E5 → E6（依赖关系：E3 依赖 E1；E4 依赖 E3；E5 依赖 E1；E6 独立）
4. **Phase D 暂存**：archive Phase D 后开 E，Phase D 代码随 E 各批次增量交付


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 11. deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 12. deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 13. deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 14. deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 15. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 16. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/redesign-phase-e-design-system-migration/deltas/test/core-PE-design-system-test-cases.md`
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
   git add -A && git commit -m "docs(redesign-phase-e-design-system-migration): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive redesign-phase-e-design-system-migration`。
