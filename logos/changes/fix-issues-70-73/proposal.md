# 变更提案：修复 MCP 大图分片、房间凭据、关系端点与关系命名

> module: core | created: 2026-11-02
> 关联 issue：#70、#71、#72、#73

## 变更原因

四个 issue 共同阻塞“大图经 MCP 导入后进入协作房间并继续编辑”的主链路，但根因不同，必须分批闭环：

1. **#70 大图无法无损分片**：全保真 54 表 / 863 字段 / 76 关系 payload 约 103,718 B，超过特定客户端约 63 KB 的单次工具参数实测边界；现有 `update_table` 不能新增整表，`update_reference` 一次只能写一条，迫使调用方裁剪数据或执行数十次全图 GET/PUT。附带发现关系快照误写 `cardinality` 而后端消费 `type_`，以及级联枚举大小写不规范。
2. **#71 房间写凭据不可持续**：房间图 PUT 收编后回退到 rooms + WS；未配 token 时先打必失败的 `GET /api/v1/rooms`，只返回笼统 `UNAUTHENTICATED`。静态 access token 仅启动时读取且默认 1 小时过期，长会话无法自愈。
3. **#72 关系端点悬空**：静态调用链确认 ID 重写发生在导入规范化，而非房间物化；同一导入 payload 内 references 已正确 remap，但后续 `update_reference` 接受导入前旧 ID，未校验表/字段存在性及归属便落库。房间首次物化只是原样加载，因此不能靠物化阶段猜测已丢失的旧→新映射修复。
4. **#73 关系缺少命名入口**：Reference.name 已贯通模型、持久化、MCP 和主要显示链路，唯独生产关系检查器没有名称输入，`on_update_ref_field` 也缺少 `name` 分支。

## 变更类型

设计级变更（扩展 MCP 工具输入与凭据会话契约、明确关系端点一致性，并补充可见的关系检查器交互）；不新增业务场景编号。

## 变更范围

- 影响的需求文档：`core-S06-mcp-service-requirements.md` FR-S06-03/08/09、MVP 凭据边界与验收场景。
- 影响的功能规格：
  - `core-S06-mcp-service-design.md`：整表 create、批量关系 create、ID 映射、端点校验、房间写错误与 login/refresh 会话。
  - `core-02-diagram-persistence.md`：导入重键与后续关系写入口的一致性约束，不恢复保留调用方旧 ID 的 #62 前行为。
  - `core-S05-ot-collab-design.md`：存量非空图首次物化不得重写实体 ID 或破坏 references/areas/notes。
  - `core-01b-relationship.md`：关系检查器名称输入、blur 落账、只读禁用和空名显示兜底。
  - `core-01-editor-prototype.html`：主原型关系检查器同步名称控件与空名展示。
- 影响的业务场景：S01（关系编辑与保存）、S05（存量图物化）、S06（MCP 分片写入与房间凭据）。
- 影响的部署方案：无；本提案仅实现与验收，不执行部署。
- 影响的 API：仅 `mcp-tools.yaml` 的工具 JSON Schema、输出与错误契约；复用现有 S03 login/refresh 和 rooms/WS API，不新增 REST 端点，不改变 S03 Cookie 契约。
- 影响的 DB 表：无；不新增 service token 表，不做数据迁移。
- 影响的编排测试：`core-S06-mcp-service.json`，同步 12 工具现状并新增分片、refresh 与端点保真链路。
- 影响的测试：`core-S06-test-cases.md`、`core-S05-test-cases.md`、`core-PB-relationship-test-cases.md`、`core-PV-verify-pre-run-test-cases.md`。
- 影响的代码：`mcp-server/src/{config,api,service,payload_limit}.rs`、`mcp-server/src/collab/`、`backend` 物化回归测试、`frontend-rs/src/editor_panels.rs`、生产 parity 与 OpenLogos reporter、MCP README/四客户端示例。
- 明确不做：Streamable HTTP/OAuth、service token 新签发体系、全局延长 JWT TTL、已有悬空关系的启发式数据修复、旧 Relationships Tab 搜索扩展、命令面板空名展示改造、关系路由或 DB Schema 变更。

## 变更概述

### #70：现有 12 工具上的无损分片

- `update_table` 新增缺省兼容的 `action=create|update`；create 一次追加一张含完整 fields 的表，update 不允许携带 fields。
- 为保持 #62 的全局 ID 隔离，create 不盲目接受调用方简单 ID；采用与导入一致的冲突安全重键，并在 create 响应中返回实际 table ID 与 field 旧→新映射。后续关系必须使用服务端实际 ID。
- `update_reference` 保留平铺单条 create/update/delete，新增仅用于 `action=create` 的 `references` 数组；单批完整校验后一次 GET/PUT 追加，任一项非法则整批不发送写请求。
- 单个与批量关系共用构造/规范化逻辑：外部 `cardinality` 写入后端 `type_`，级联值显式转为大写下划线。文档建议单表一片、关系按数组分片，单次参数保守控制在约 48 KiB；约 63 KB 仅作为特定客户端实测值。
- 非房间单批 PUT 是单次快照写；房间批量关系经多个 op 逐条 ack，不宣称跨调用回滚、整链原子或 revision 只加 1。

### #71：房间写错误上下文化 + 内存凭据会话

- 在统一 `USE_OP_CHANNEL` 回退点区分无凭据与 rooms 401，返回 `ROOMS_AUTH_REQUIRED` 及 `TOKEN_MISSING` / `TOKEN_INVALID_OR_EXPIRED`，保留安全上游诊断；403 与 5xx 不混淆。
- 保留静态 `COLDRAWDB_ACCESS_TOKEN` 兼容路径；新增显式 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD` 登录配置（与静态 token 二选一），复用现有 login/refresh，在内存维护 access token 与 refresh cookie。
- API 401 时最多 refresh 一次并重试原请求一次；refresh 自身不递归。WS 仅在握手 401 时尚未发送 op 的前提下 refresh 并重连一次，已发送 op 不盲目重放。
- 密码与 refresh cookie 不进入 URL；access token 仅按既有 Authorization header 与 WS 握手 query 契约传输。所有日志、reporter、错误摘要和诊断 URL 必须删除密码、token 与 cookie，不要求用户复制 httpOnly refresh token，也不新增长期高权 token。

### #72：阻止悬空关系产生并证明物化保真

- `update_reference` 单个/批量 create 在写前校验四个端点真实存在，且字段分别属于指定表；旧 ID、悬空 ID 或表字段不归属均在 PUT/WS 前返回 `VALIDATION_ERROR`。
- 保留导入规范化既有 remap，不在 collab 物化阶段二次重键，不猜测已污染数据的历史映射。
- 新增存量非空图物化回归，证明 table/field/reference/area/note ID 与关系端点保持不变；真实链路覆盖“导入→取实际 ID→建关系→开房间→首 op→再读取”。

### #73：关系检查器名称编辑

- 在 `inspector-reference-form` 顶部增加 `inspector-ref-name` 文本输入，使用原始 `r.name` 作为 value，统一显示名作为 placeholder；blur 时 trim 并落账，允许空串清除名称，同值 blur 不写。
- `on_update_ref_field` 增加 `name` 分支，复用 store → dirty → schedule_save；只读禁用且回调保留第二道只读保护。
- 空名继续由 `relation_display_name` 回退为 `未命名关系（<id>）`，不引入 `ref-{id}` 新兜底，不扩展 UndoRedo。

## 兼容性影响

- #70 旧 `update_table` 缺省 action 仍为 update，旧平铺 `update_reference` 保持不变，工具总数仍为 12；只有 create 新形态携带 ID 映射扩展输出。
- #71 静态 token 用户可继续原方式使用；普通 API 401 仍为 `UNAUTHENTICATED`，只有房间写回退增加上下文化错误。动态登录与静态 token 明确互斥，避免凭据优先级歧义。
- #72 以往会“成功”写入的无效端点现在会在写前拒绝，这是防止静默数据损坏的有意行为变更；有效端点调用不变。
- #73 数据格式、API、DB 与空名显示函数不变，仅新增编辑入口。

## 部署影响

- 是否需要部署：否。
- 部署原因：本提案只完成本地 MCP、后端回归和前端实现及验收，随下次常规发布携带。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 分批与验收策略

1. **#70 批**：UT-MCP-41～44 覆盖整表、批量关系、字段/基数/级联规范化与房间 diff；ST-MCP-18 使用真实隔离 backend + stdio 验证 54 表 / 863 字段 / 76 关系全保真分片。
2. **#71 批**：UT-MCP-45～48 覆盖缺失/失效凭据、单次 refresh/retry、WS 握手与脱敏；ST-MCP-19/20 覆盖真实 stdio 错误和 login→expiry→refresh→房间 ack。
3. **#72 批**：UT-MCP-49 覆盖无效端点零写入，UT-C-11 覆盖存量物化保真；ST-MCP-21 覆盖导入到房间的完整端点链路。
4. **#73 批**：UT-PB-38 覆盖显示与空值口径；ST-PB-22 覆盖真实检查器编辑、保存重载、清空和只读。

每批同时交付业务代码、对应 UT/ST 与 OpenLogos reporter，不将测试推迟到最后一批。
