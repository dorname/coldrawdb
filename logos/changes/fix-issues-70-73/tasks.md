# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md` — 扩展 FR-S06-03/08/09、凭据边界与大图/房间写验收
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 定义整表 create、ID 映射、批量关系、端点校验、房间错误与 login/refresh 会话
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 补充导入重键后关系写入口的实际 ID 与端点一致性约束
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md` — 明确存量非空图首次物化不重写实体 ID、不破坏关系与附属对象
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 增加关系名称检查器输入、blur 落账、只读与空名兜底
- [x] 产出 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 主原型同步关系名称控件与显示交互
- [x] 产出 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md` — 增加整表/批量分片、401 refresh 单次重试、房间回退与无效端点时序
- [x] 产出 `deltas/api/mcp-tools.yaml` — 扩展 update_table/update_reference 输入输出、ROOMS_AUTH_REQUIRED 与 details，并验证 OpenAPI 3.x YAML
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 登记 UT-MCP-41～49、ST-MCP-18～21 与真实 backend/stdio 验收要求
- [x] 产出 `deltas/test/core-S05-test-cases.md` — 登记 UT-C-11 存量图物化 ID 与关系保真
- [x] 产出 `deltas/test/core-PB-relationship-test-cases.md` — 登记 UT-PB-38、ST-PB-22 关系名称编辑与持久化
- [x] 产出 `deltas/test/core-PV-verify-pre-run-test-cases.md` — 登记新增 MCP/backend/parity 测试的真实执行入口，禁止声明式假入账
- [x] 产出 `deltas/scenario/core-S06-mcp-service.json` — 校正 12 工具及文本错误契约，新增分片、refresh 与端点保真编排

## [code] 代码实现

### #70 批：大图无损分片

- [ ] 修改 `mcp-server/src/service.rs` 与 `mcp-server/src/payload_limit.rs` — 实现整表 create、冲突安全 ID 重键与映射输出、批量关系 create、`type_` 和级联规范化
- [ ] 新增 `mcp-server/tests/c11_chunked_import.rs` 并接 reporter — 交付 UT-MCP-41～44 与真实隔离 backend + stdio 的 ST-MCP-18，验证全保真 54 表 / 863 字段 / 76 关系
- [ ] 更新 `mcp-server/README.md` 与四客户端配置示例 — 说明客户端实测尺寸边界、单表/关系分片顺序、ID 映射与跨调用非原子边界

### #71 批：房间凭据与错误语义

- [ ] 修改 `mcp-server/src/config.rs`、`mcp-server/src/api.rs` 与 `mcp-server/src/collab/` — 实现内存 login/refresh 凭据会话、401 单次重试、WS 握手单次重连与统一脱敏
- [ ] 在统一房间回退中实现 `ROOMS_AUTH_REQUIRED` 及原因 details，保留普通 401/403/5xx 语义
- [ ] 更新 `mcp-server/tests/c7_room_write.rs`、错误/配置测试并新增会话测试 — 交付 UT-MCP-45～48、ST-MCP-19/20 与 reporter
- [ ] 更新 `mcp-server/README.md` 与四客户端示例 — 说明静态 token 和动态登录互斥、房间写前置条件及安全配置，不写入真实凭据

### #72 批：关系端点一致性

- [ ] 修改 `mcp-server/src/service.rs` 的单个/批量关系 create — 校验四端点存在性及字段归属，无效输入在 PUT/WS 前返回且零写入
- [ ] 修改 `backend/src/collab/mod.rs` 或邻近既有测试模块 — 交付 UT-C-11，验证存量非空图首次物化保持实体 ID、关系端点、areas 与 notes
- [ ] 扩展真实 MCP 集成测试与 reporter — 交付 UT-MCP-49、ST-MCP-21，覆盖导入→实际 ID→建关系→开房间→首 op→读取

### #73 批：关系名称编辑

- [ ] 修改 `frontend-rs/src/editor_panels.rs` — 增加 `inspector-ref-name`、blur trim/清空/同值不写、只读禁用与 `on_update_ref_field` 的 name 分支
- [ ] 在关系纯函数测试与 `frontend-rs/scripts/test-spec-parity-d.mjs` 交付 UT-PB-38、ST-PB-22 — 覆盖真实编辑、保存重载、列表/标签显示、清空和只读
- [ ] 更新 `frontend-rs/tests/openlogos_reporter.rs` 及对应 reporter — 确保新增 UT/ST 真实写入 `logos/resources/verify/test-results.jsonl`
