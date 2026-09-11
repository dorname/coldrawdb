# 合并指令

## 变更提案
- 提案名称：fix-canvas-zoom-invite-comment-resize
- 提案目录：logos/changes/fix-canvas-zoom-invite-comment-resize/

## 提案内容

# 变更提案：fix-canvas-zoom-invite-comment-resize

> module: core | created: 2026-09-08

## 变更原因

用户实测反馈四个问题（2026-09-08）：

1. **画布性能与缩放**：画布表很多时操作不流畅；滚轮缩放未以光标位置为中心。
2. **协作邀请链接无效**：生成的邀请链接打不开，多人协作能力验证失败。
3. **数据库导入丢失注释**：从数据库导入时未导入表/列注释（comment）；列表视图需展示注释名称，字段部分需有注释字段。
4. **布局面板不可调整**：右侧栏、画布、列表菜单列宽等既不能自适应也无法拖动调整大小。

根因定位（只读探查结论）：

- **问题1 性能**：`frontend-rs/src/editor_render.rs` 任何信号变化触发全量重绘且无视口裁剪；网格点阵逐点绘制（≈1.5 万次 Canvas 调用/帧）；每帧每表 6 次 DOM 字体探测（100 表 ≈ 3000 次 DOM 跨界/帧）；拖动中每个 mousemove 全表 Vec 深克隆；pan 路径无 rAF 节流；refs 绘制 O(refs×tables) 线性查找。
- **问题1 缩放**：生产分支 `drawdb-web` 的 `on_wheel`（editor_render.rs:298-299）坐标空间混用——反向计算 pan 用了视口坐标而非画布相对坐标，实际锚定点落在 (rect.left, rect.top)。当前工作分支已有正确实现（:1239-1262），需对齐合入。
- **问题2**：`backend/src/rooms/mod.rs:495` 硬编码 `http://localhost/invite/{token}`（无端口、不可配置，与 S04 时序图规格不符）；前端 API base URL 硬编码 `http://127.0.0.1:3000`（editor_panels.rs:8788-8791），被邀请人跨机访问必然失败。邀请页路由与 preview/accept API 本身已接通。
- **问题3**：数据库导入走「introspect → DDL 文本 → 前端解析 DDL」纯文本管道，查询（bridge_introspect.rs）、IR、DDL 渲染、前端 DDL 解析四个环节全部无注释概念；而前后端数据模型 comment 字段齐全、JSON 路径已透传。表注释在前端无任何展示/编辑入口（字段注释列在 ListView 已存在）。
- **问题4**：`.cdb-main` 三列网格宽度全部写死（toolrail 64px / inspector 330px），无任何 splitter / resize 监听；列表树列写死 230px。上游 drawdb 亦无此功能，属新功能。

## 变更类型

设计级变更（含接口级子项：bridge 导入响应形状变更、invite URL 生成规则变更）

## 变更范围

- 影响的需求文档：
  - `prd/1-product-requirements/core-04-scenario-detail.md`（S04 邀请验收条件、画布性能/注释展示验收条件）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（光标锚定缩放、性能预算、面板宽度调整）
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（表注释展示/编辑入口）
  - `prd/2-product-design/1-feature-specs/core-01d-import-export.md`（导入保留注释、导出 COMMENT ON TABLE）
  - `prd/2-product-design/1-feature-specs/core-03-bridge-io.md`（bridge 导入结构化响应）
  - `prd/2-product-design/1-feature-specs/core-09-core-components.md`（新增 splitter 分隔条组件规格）
  - `prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`（邀请链接生成规则）
- 影响的原型：
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html`（分隔条交互、表注释展示）
- 影响的业务场景：S01（画布编辑）、S04（房间邀请）
- 影响的技术方案：
  - `prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md`（invite URL 由配置/请求推导）
  - `prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`（新增 PUBLIC_BASE_URL 配置项说明）
- 影响的 API：
  - `api/rooms.yaml`（create invite 响应 invite_url 生成规则说明）
  - `api/bridge.yaml`（import/connect 响应由 DDL 文本改为结构化 tables JSON）
- 影响的 DB 表：无（comment 字段在现有模型/表中已存在）
- 影响的编排测试：
  - `scenario/core-S04-room-lifecycle.json`（invite_url 断言规则）
- 影响的测试用例：
  - `test/core-CR-canvas-test-cases.md`（光标锚定缩放、视口裁剪/渲染性能用例）
  - `test/core-S04-test-cases.md`（邀请链接可用性用例）
  - `test/core-PC-import-export-test-cases.md`（导入注释透传、导出 COMMENT ON TABLE 用例）
  - `test/core-PU-unified-prototype-test-cases.md` 或新增 splitter 用例（面板拖动调整）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本地自托管工具的缺陷修复与前端功能增强，无生产部署任务；部署文档仅补充 PUBLIC_BASE_URL 配置说明（delta 文档变更，非部署执行）
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

本次变更修复四个用户实测问题：

1. **画布性能与光标锚定缩放**：为 draw_canvas 增加视口 AABB 裁剪；网格点阵合并为单 path 一次填充；字体家族解析结果缓存；拖动中的位置更新改为覆盖层（只存被拖对象 id+x/y）替代全表深克隆；pan 路径接入 rAF 节流；refs 查找改 HashMap。滚轮缩放将 drawdb-web 生产分支的 on_wheel 对齐为光标锚定实现（anchor 减去 rect.left/top）。
2. **协作邀请链接可用**：后端 invite_url 改为从 PUBLIC_BASE_URL 配置或请求 Host 推导；前端各 Client 的 base_url 改为同源派生（window.location.origin，dev 可配）。
3. **导入/展示注释**：`/bridge/import/connect` 由返回 DDL 文本改为直接返回结构化 tables JSON（含表/列 comment），前端复用 JSON 导入路径；PG introspection 抓取 obj_description/col_description；导出补齐 COMMENT ON TABLE；表树与 Inspector 增加表注释展示/编辑。
4. **面板可拖动调整**：新增 splitter 分隔条组件（pointer capture + 拖动期直写 CSS 变量，绕过响应式重渲染），应用于 inspector 宽度与 ListView 表树列宽，宽度持久化 localStorage，最小/最大宽度钳制。


## 需要合并的 Delta 文件

### 1. deltas/api/bridge.yaml

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/api/bridge.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/api/rooms.yaml

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/api/rooms.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 11. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 12. deltas/scenario/core-S04-room-lifecycle.json

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/scenario/core-S04-room-lifecycle.json`
- 目标目录：`logos/resources/scenario/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 13. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 14. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 15. deltas/test/core-PU-unified-prototype-test-cases.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/test/core-PU-unified-prototype-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 16. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/fix-canvas-zoom-invite-comment-resize/deltas/test/core-S04-test-cases.md`
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
   git add -A && git commit -m "docs(fix-canvas-zoom-invite-comment-resize): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-canvas-zoom-invite-comment-resize`。
