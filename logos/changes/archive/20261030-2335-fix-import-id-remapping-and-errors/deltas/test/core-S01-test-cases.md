## ADDED — 导入 ID 隔离与错误诊断用例

### UT-S01-14 — 导入实体 ID 重映射与端点回填

- **位置**：`backend/src/diagram_persistence.rs`
- **输入**：payload 含固定 table `t1`、field `f1`、reference `r1`、area `a1`、note `n1`；reference 端点指向 `t1/f1`。
- **操作**：执行 `normalize_import_payload`。
- **断言**：五类保留实体均获得不同的 `auto-` 新 ID，且不等于输入 ID；reference 四个端点分别等于重映射后的 table/field ID；字段归属与数组顺序不变。

### UT-S01-15 — 嵌套 DTO 类型错误不得静默变空图

- **位置**：`backend/src/diagram_persistence.rs`
- **输入**：tables 中字段 `primary: 1`（布尔位置使用整数）。
- **操作**：构建导入 DiagramFull。
- **断言**：返回 `SaveDiagramError::BadRequest`；错误包含 `tables` 和 Serde 的布尔类型原因；不得返回 tables 为空的成功 DTO。

### ST-S01-06 — 重复导入、400/500 诊断与跨图隔离

- **位置**：`backend/src/diagrams_v1.rs::tests`，独立临时 SQLite。
- **步骤**：
  1. 构造四表四字段、三关系的固定 ID payload，连续两次 POST `/api/v1/diagrams/import`。
  2. 分别 GET 两个返回的 diagram_id，比较表、字段和关系。
  3. POST 字段 `primary: 1` 的 payload。
  4. 在临时库中移除 `reference` 表以制造持久化异常，再次 POST 合法 payload。
- **断言**：
  - 两次导入均为 200，各实际持久化 4 表 4 字段；diagram_id 不同，两个图的 table/field/reference ID 集合互不相交。
  - 每个图均有 3 条关系，所有端点指向本图重映射后的表和字段，不指向原始 ID 或另一图。
  - 类型错误返回 400，`details.kind=payload`，`details.message` 同时包含 `tables` 和布尔类型原因；不得返回 `imported_tables=0` 成功。
  - 持久化异常返回 500，`details.kind=database` 且 `details.message` 非空；不会留下可读取的半成品图。
- **reporter**：分别登记 UT-S01-14、UT-S01-15、ST-S01-06 到 `logos/resources/verify/test-results.jsonl`。

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S01-01 | 创建空 diagram | `backend/src/diagrams_v1.rs` |
| UT-S01-02 | 创建含 5 表 20 字段 | `backend/src/diagrams_v1.rs` |
| UT-S01-03 | PUT 带正确 revision | `backend/src/diagrams_v1.rs` |
| UT-S01-04 | PUT 带过期 revision | `backend/src/diagrams_v1.rs` |
| UT-S01-05 | DELETE → 级联删除 | `backend/src/diagrams_v1.rs` |
| UT-S01-06 | POST 导入 JSON | `backend/src/diagrams_v1.rs` |
| UT-S01-07 | GET 不存在 → 404 | `backend/src/diagrams_v1.rs` |
| UT-S01-08 | revision 单调递增 | `backend/src/diagrams/v1/service.rs` |
| UT-S01-09 | 并发 PUT 冲突 | `backend/src/diagrams/v1/service.rs` |
| UT-S01-10 | JSON 字段类型校验 | `backend/src/diagrams_v1.rs` |
| UT-S01-11 | field.tag 保存/加载往返 | `backend/src/diagram_persistence.rs` |
| UT-S01-12 | save_diagram name=None 不覆盖列值 | `backend/src/diagram_persistence.rs` |
| UT-S01-13 | normalize_import_payload 逐实体容错 | `backend/src/diagram_persistence.rs` |
| UT-S01-14 | 导入实体 ID 重映射与端点回填 | `backend/src/diagram_persistence.rs` |
| UT-S01-15 | 嵌套 DTO 类型错误不得静默变空图 | `backend/src/diagram_persistence.rs` |
| UT-ID-GLOBAL-01 | 前端实体 id 全局唯一(1000 个 id 互不重复) | `frontend-rs/tests/entity_id_uniqueness.rs` |
| UT-ID-GLOBAL-02 | 新格式 id 绕过 max+1 解析(兼容存量加载) | `frontend-rs/src/editor_panels.rs` |
| UT-S01-AUTH-01 | flag=on：无 token POST/PUT/DELETE/import → 401 | `backend/src/diagrams_v1.rs` |
| UT-S01-AUTH-02 | flag=on：非法/过期/伪造签名 token → 401 | `backend/src/diagrams_v1.rs` |
| UT-S01-AUTH-03 | flag=off：匿名直通（过渡态回归，行为同 v1.1） | `backend/src/diagrams_v1.rs` |
| UT-S01-AUTH-04 | flag=on：bridge 7 操作无 token → 401；带 token 正常 | `backend/src/phase3_bridge.rs` |
| UT-S01-AUTH-05 | share 端点：匿名可调（off）/401（on）；铸造-轮换-软删 404 | `backend/src/diagrams_v1.rs` |
| ST-S01-01 | 编辑保存端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-02 | 导入端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-04 | 导入缺 id/name payload 兜底补全 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-05 | 导入坏表丢弃 + warnings 明细 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-06 | 重复导入、400/500 诊断与跨图隔离 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-03 | 浏览器 wasm 渲染 | `frontend-rs/tests/wasm/` |
| ST-S01-AUTH-01 | 登录态编辑保存全链路（编排 core-S01 v1.1.0：匿名 401 → 登录 CRUD → 409 → 删除） | `backend/src/diagrams_v1.rs::tests` |
