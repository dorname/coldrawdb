# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-01-editor-canvas.md：光标锚定缩放规则、渲染性能预算（视口裁剪/网格合批/字体缓存/拖动覆盖层）、面板宽度可调整规格
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-01a-table-and-field.md：表注释在表树与 Inspector 的展示/编辑规格
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-01d-import-export.md：导入保留表/列注释、导出 COMMENT ON TABLE
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-03-bridge-io.md：import/connect 响应改为结构化 tables JSON（含 comment）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-09-core-components.md：新增 splitter 分隔条组件规格
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-S04-room-lifecycle-design.md：邀请链接生成规则（PUBLIC_BASE_URL / Host 推导）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/` — core-01-editor-prototype.html：分隔条交互与表注释展示
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/` — core-S04-room-lifecycle.md：invite URL 推导时序
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/` — core-01-deployment-plan.md：新增 PUBLIC_BASE_URL 配置项说明
- [x] 产出 delta 文件到 `deltas/api/` — rooms.yaml：create invite 响应 invite_url 生成规则
- [x] 产出 delta 文件到 `deltas/api/` — bridge.yaml：import/connect 响应形状变更（结构化 JSON）
- [x] **验证 API YAML** — rooms.yaml / bridge.yaml delta 内嵌 YAML 块均通过 yaml.safe_load 校验
- [x] 产出 delta 文件到 `deltas/scenario/` — core-S04-room-lifecycle.json：invite_url 断言规则（内嵌 JSON 块校验通过）
- [x] 产出 delta 文件到 `deltas/test/` — core-CR-canvas-test-cases.md：光标锚定缩放与渲染性能用例（UT-CR-ZOOM-01/CULL-01/BATCH-01/FONTCACHE-01/DRAG-01、ST-CR-PAN-01/INSP-01）
- [x] 产出 delta 文件到 `deltas/test/` — core-S04-test-cases.md：邀请链接可用性用例（UT-S04-16/17、UT-S04-UI-17、ST-S04-02）
- [x] 产出 delta 文件到 `deltas/test/` — core-PC-import-export-test-cases.md：注释透传与 COMMENT ON TABLE 用例（UT-PC-24~30、ST-PC-07/08，另 MODIFIED UT-PC-07/11/12/13、ST-PC-04）
- [x] 产出 delta 文件到 `deltas/test/` — 面板 splitter 拖动调整用例（UT-PU-22~24、ST-PU-27~30）

## [code] 代码实现
- [x] 画布性能：视口 AABB 裁剪、网格点阵合批、字体解析缓存、拖动覆盖层替代深克隆、pan rAF 节流、refs 查找 HashMap 化（frontend-rs/src/editor_render.rs、editor_core.rs）
- [x] 滚轮缩放：生产分支 on_wheel 对齐光标锚定实现（anchor 减 rect.left/top）
- [x] 邀请链接：后端 invite_url 由 PUBLIC_BASE_URL/Host 推导（backend/src/rooms/mod.rs）；前端 Client base_url 同源派生（frontend-rs/src/editor_panels.rs、editor_data_access.rs）
- [x] 导入注释：PG introspection 抓 obj_description/col_description、IR 加 comment、import/connect 返回结构化 JSON；前端消费分支复用 JSON 导入（backend/src/bridge_introspect.rs、phase3_bridge.rs；frontend-rs/src/editor_panels.rs）
- [x] 注释展示：表树与 Inspector 增加表注释展示/编辑；导出补齐 COMMENT ON TABLE
- [x] 面板 resize：新增 splitter 组件，应用于 inspector 与 ListView 表树列，宽度持久化 localStorage
- [x] 编写/更新对应 UT/ST 测试并接入 OpenLogos reporter
