# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — #7/#8：dropzone 拖放/点击选文件交互 + `.ddl` 放行
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — #9：按相对位置自动选择出入侧；#12：关系线颜色配置
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — #11：尺寸变化 DPR 同步；#17：拖拽/选中圆角高亮；#10：注释展示与显示开关
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — #10：表/字段注释展示口径；#12：表头/表边框颜色配置
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — #13/#14/#15：顶栏返回区与协作芯片样式/截断策略
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型对齐（#9/#10/#12 行为；#13/#15 仅微校）
- [x] 产出 delta 文件到 `deltas/api/auth.yaml` — #16：access TTL 可配置化描述与 expiresIn 示例
- [x] 产出 delta 文件到 `deltas/api/diagrams.yaml` — #12：Reference 增加 `color` 字段
- [x] 产出 delta 文件到 `deltas/api/mcp-tools.yaml` — #12：`UpdateReferenceInput` 增加 color；#18：写工具瘦响应约定
- [x] **验证 API YAML** — `logos/resources/api/` 下所有 delta 文件必须为有效 YAML 且符合 OpenAPI 3.x 规范（所有包含 `:` 或特殊字符的 `description`/`summary` 值必须用双引号包裹）
- [x] 产出 delta 文件到 `deltas/database/coldrawdb-v1.sql` — #12：reference 表新增 `color` 列
- [x] 产出 delta 文件到 `deltas/test/core-PC-import-export-test-cases.md` — #7/#8 拖放/.ddl 用例
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — #9 自动选侧/#12 关系颜色用例
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — #10 注释展示/#17 圆角高亮用例
- [x] 产出 delta 文件到 `deltas/test/core-RP-canvas-hidpi-test-cases.md` — #11 DPR 同步用例
- [x] 产出 delta 文件到 `deltas/test/core-PE-design-system-test-cases.md` — #13/#14/#15 顶栏用例
- [x] 产出 delta 文件到 `deltas/test/core-S03-test-cases.md` — #16 TTL 配置化用例
- [x] 产出 delta 文件到 `deltas/test/core-S06-test-cases.md` — #18 契约测试用例

## [code] 代码实现

批次 A — 导入（#7、#8）：
- [ ] `frontend-rs/src/editor_panels.rs`：dropzone 绑定 drag/drop 事件 + 隐藏 file input + `.ddl` 放行（文案/accept/白名单）
- [ ] 批次 A 测试代码 + OpenLogos reporter（覆盖 core-PC 新增用例）

批次 B — 画布渲染修复（#9、#11、#17）：
- [ ] `frontend-rs/src/editor_render.rs`：`calc_path` 按相对位置自动选择出入侧；拖拽建关系双侧 port 可作起/终点
- [ ] `frontend-rs/src/editor_render.rs`（+ `splitter.rs`）：ResizeObserver/resize 驱动 canvas 缓冲与 DPR transform 同步
- [ ] `frontend-rs/src/editor_render.rs`：拖拽幽灵层圆角高亮（替代 CSS outline）
- [ ] 批次 B 测试代码 + OpenLogos reporter（覆盖 core-PB/core-CR/core-RP 新增用例）

批次 C — 顶栏 UI（#13/#14、#15）：
- [ ] `frontend-rs/src/styles.css` + `editor_panels.rs`：返回按钮对齐原型 ghost 语言；room-badge 截断规则与弹性收缩策略
- [ ] 批次 C 测试代码 + OpenLogos reporter（覆盖 core-PE 新增用例）

批次 D — 展示增强（#10、#12）：
- [ ] `frontend-rs/src/editor_render.rs`：表头/字段行注释展示 + 显示开关；表边框色跟随 `table.color`；关系线按 `color` 渲染
- [ ] `frontend-rs/src/editor_core.rs` + 后端 entity + `backend/migrations/0006_*`：Reference 增加 `color` 字段并落库
- [ ] `frontend-rs/src/editor_panels.rs`：Inspector 表颜色与关系颜色入口
- [ ] JSON 导入导出保留颜色（SQL/DBML 降级忽略）
- [ ] 批次 D 测试代码 + OpenLogos reporter（覆盖 core-CR/core-PB 新增用例）

批次 E — auth TTL（#16）：
- [ ] `backend/src/auth/jwt.rs` + `init.rs` + `config.toml`：access TTL 配置化（默认 3600s + env 覆盖），expiresIn 与 JWT exp 一致
- [ ] 批次 E 测试代码 + OpenLogos reporter（覆盖 core-S03 新增用例）

批次 F — MCP 契约（#18）：
- [ ] `mcp-server/src/service.rs`：写工具返回前 normalize 为 outputSchema 瘦响应
- [ ] 批次 F 契约测试（structuredContent 通过 outputSchema 校验）+ OpenLogos reporter（覆盖 core-S06 新增用例）
