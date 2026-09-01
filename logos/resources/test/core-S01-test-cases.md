## 1. 范围

S01 覆盖编辑、自动保存、`SaveState`、非 OT 路径 409。房间协作合并见 S05。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（C 批）将 SaveState / 非 OT 409 / 协作禁 409 用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## 2. UT 用例（单元测试）

### UT-S01-01 — 创建空 diagram

- **位置**：`backend/src/diagrams_v1.rs::create` 的单元测试
- **前置**：干净数据库
- **步骤**：
  1. POST `/api/v1/diagrams` body `{"title": "Empty", "tables": []}`
  2. 验证响应 201
  3. 验证响应 body `id` 非空 UUID
  4. 验证 `revision == 0`
  5. 验证 `tables == []`
- **断言**：
  - `status_code == 201`
  - `response.id.is_uuid()`
  - `response.revision == 0`
  - 数据库 `diagram` 表新增 1 行

### UT-S01-02 — 创建含 5 表 20 字段

- **位置**：`backend/src/diagrams/service.rs::create` 单元测试
- **前置**：干净数据库
- **步骤**：
  1. 构造 DiagramCreateRequest 含 5 张表，每张 4 字段
  2. 调用 service::create
  3. 返回 Diagram
- **断言**：
  - `result.is_ok()`
  - 数据库 `diagram` 表 +1 行
  - 数据库 `table` 表 +5 行
  - 数据库 `field` 表 +20 行
  - 所有 field.table_id 都对应已创建的 table.id

### UT-S01-03 — PUT 带正确 revision → 200

- **位置**：`backend/src/diagrams_v1.rs::update` 单元测试
- **前置**：已存在 diagram (rev=5)
- **步骤**：
  1. PUT `/api/v1/diagrams/{id}` body 包含 `revision: 5` + 1 张新表
  2. 验证响应 200
  3. 验证响应 `revision == 6`
- **断言**：
  - `status_code == 200`
  - `response.revision == 6`
  - 数据库 diagram.revision 已更新

### UT-S01-04 — PUT 带过期 revision → 409

- **位置**：`backend/src/diagrams/service.rs::update` 单元测试
- **前置**：已存在 diagram (rev=5)
- **步骤**：
  1. PUT body `revision: 4`（过期）+ 1 张新表
  2. 验证响应 409
  3. 验证响应 body `current` 包含当前 diagram 状态（rev=5）
- **断言**：
  - `status_code == 409`
  - `response.error == "revision_conflict"`
  - `response.current.revision == 5`
  - 数据库 diagram 行未变更（事务回滚）

### UT-S01-05 — DELETE → 级联删除

- **位置**：`backend/src/diagrams/service.rs::delete` 单元测试
- **前置**：已存在含 5 表 20 字段的 diagram
- **步骤**：
  1. DELETE `/api/v1/diagrams/{id}`
  2. 验证响应 204
  3. 直接查 `SELECT * FROM table WHERE diagram_id = ?` → 0 行
  4. 直接查 `SELECT * FROM field WHERE table_id IN (...)` → 0 行
  5. 直接查 `SELECT * FROM reference WHERE diagram_id = ?` → 0 行
- **断言**：
  - `status_code == 204`
  - diagram 表 -1 行
  - table 表 -5 行
  - field 表 -20 行

### UT-S01-06 — 事务原子性

- **位置**：`backend/src/diagrams/service.rs::update` 单元测试
- **前置**：已存在 diagram (rev=5)
- **步骤**：
  1. 构造 PUT body：1 张有效表 + 1 张引用不存在字段的表（触发外键错误）
  2. 验证响应 400
  3. 验证 diagram.revision 仍为 5（事务回滚）
- **断言**：
  - `status_code == 400`
  - `response.error == "validation_error"`
  - 数据库 diagram.revision == 5
  - 数据库未新增 table 行

### UT-S01-07 — 表创建触发 undo 栈

- **位置**：`frontend-rs/src/editor_core.rs::create_table` 单元测试
- **前置**：内存 Diagram 状态
- **步骤**：
  1. 调用 `create_table(table)` 3 次
  2. 验证 undo_stack.length == 3
  3. 调用 `undo()` 1 次
  4. 验证 diagram.tables.length == 2
- **断言**：
  - `undo_stack.len() == 3`（在 3 次 create 后）
  - `diagram.tables.len() == 2`（undo 1 次后）

### UT-S01-08 — debounce 触发 save

- **位置**：`frontend-rs/src/editor_data_access.rs::save_loop` 单元测试（用 mock 时间）
- **前置**：mock HTTP 客户端；clean Diagram
- **步骤**：
  1. 启动 save_loop
  2. 调用 `create_table` 触发 dirty
  3. 等待 1100ms（debounce + 100ms 缓冲）
  4. 验证 mock HTTP 收到 1 次 PUT 请求
  5. 验证请求 body 含新建的 table
- **断言**：
  - `mock.requests.len() == 1`
  - `mock.last_request.body.tables.len() == 1`

### UT-S01-09 — 自动保存失败重试

- **位置**：`frontend-rs/src/editor_data_access.rs::save` 单元测试
- **前置**：mock HTTP 客户端返回 500
- **步骤**：
  1. 调用 save
  2. 验证第 1 次重试间隔 ≈ 1s
  3. 验证第 2 次重试间隔 ≈ 2s
  4. 验证第 3 次失败后 SaveState 变 `Error`
- **断言**：
  - `mock.requests.len() == 4`（首次 + 3 次重试）
  - `save_state == "Error"`

### UT-S01-10 — 字段类型校验

- **位置**：`backend/src/diagrams/service.rs::validate_field` 单元测试
- **步骤**：
  1. 提交字段 `name: "field_1"`, `type: "INVALID_TYPE"`, `primary: false`
  2. 验证响应 400
- **断言**：
  - `status_code == 400`
  - `response.error == "invalid_field_type"`

## 3. ST 用例（场景测试 / E2E）

### ST-S01-01 — 编辑 → 自动保存 → 重新加载 → 一致

- **位置**：`backend/tests/scenarios/s01.rs`
- **类型**：Rust integration test（用 wiremock + actix-web test）
- **步骤**：
  1. 启动后端 + WASM（headless）
  2. POST 创建空 diagram
  3. 通过 frontend-rs API（headless）：创建 5 表 20 字段
  4. 等待 1.2s（debounce 触发 save）
  5. GET `/api/v1/diagrams/{id}` → 全量数据
  6. 验证 GET 响应与本地状态一致
- **断言**：
  - GET 响应 200
  - 响应 tables.length == 5
  - 所有字段名 / 类型一致
  - 响应 revision == 本地 revision

### ST-S01-02 — A 编辑保存后 B 加载，B 编辑触发 409

- **位置**：`backend/tests/scenarios/s01.rs`
- **步骤**：
  1. 启动后端
  2. A POST 创建 diagram (rev=0)
  3. A PUT 修改 (rev=0 → rev=1)
  4. B GET 加载（看到 rev=1）
  5. A 再次 PUT (rev=1 → rev=2)
  6. B 仍持有 rev=1，B PUT 用 rev=1
  7. 验证 B 收到 409
- **断言**：
  - B 响应 status == 409
  - B 响应 body.error == "revision_conflict"
  - B 响应 body.current.revision == 2

### ST-S01-03 — WASM 端到端（带浏览器）

- **位置**：`frontend-rs/tests/wasm/s01.rs`（wasm-pack test --headless）
- **步骤**：
  1. 启动 staging 后端
  2. wasm-pack test --headless --chrome
  3. 打开 `/editor`
  4. 模拟点击 "+" 创建表
  5. 等待 1.2s
  6. 验证 SaveState 变 "Saved"
- **断言**：
  - SaveState 文本 == "Saved"

## 4. 测试数据工厂

`backend/src/diagrams/test_factory.rs`：

```rust
pub fn make_diagram() -> DiagramCreateRequest {
    DiagramCreateRequest {
        title: "Test".into(),
        tables: vec![
            make_table("users", vec![
                make_field("id", "INT", true, false, true),
                make_field("name", "VARCHAR(255)", false, false, false),
            ]),
            make_table("posts", vec![
                make_field("id", "INT", true, false, true),
                make_field("user_id", "INT", false, false, true),
                make_field("title", "VARCHAR(255)", false, false, false),
            ]),
        ],
        references: vec![
            make_reference(0, 1, "one_to_many"),
        ],
        areas: vec![],
        notes: vec![],
    }
}
```

## 5. OpenLogos Reporter

所有测试运行结果通过 `logos/spec/test-results.jsonl` 记录：

```jsonl
{"test_id": "UT-S01-01", "status": "passed", "duration_ms": 12, "timestamp": "2026-06-08T10:00:00Z"}
{"test_id": "UT-S01-02", "status": "passed", "duration_ms": 45, "timestamp": "2026-06-08T10:00:01Z"}
{"test_id": "ST-S01-01", "status": "passed", "duration_ms": 1230, "timestamp": "2026-06-08T10:00:05Z"}
```

字段定义见 `logos/spec/test-results.md`。

## SaveState 与页面锚点

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| UT-S01-SS-01 | dirty 编辑 | debounce 触发 PUT 成功 | `save-state`：未保存→保存中→已保存；`revision-display` +1 | 本提案 C 批实现 |
| UT-S01-SS-02 | PUT 网络失败 | 重试耗尽 | `save-state=Error`；可手动重试；不丢本地 dirty | 本提案 C 批实现 |
| ST-S01-SS-01 | room-editor 可写角色 | 改表后等待自动保存 | AppBar 保存态与 revision 与主原型文案阶段一致 | 本提案 C 批实现 |

## 非 OT 409

| ID | 说明 | 状态 |
|---|---|---|
| UT-S01-04 / ST-S01-02 | 过期 revision → 409 → `modal-conflict`（reload/force/cancel）；仅**非 OT** 快照冲突路径 | 既有；本提案回归 |
| ST-S01-409-SCOPE | 协作模式（S05 已连接 OT）下服务器合并成功；**禁止**出现 `modal-conflict`；Toast/Activity 反馈 | 本提案 C 批实现 |

## 协作模式禁 409（合同）

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S01-NO-409-OT | 两用户 OT 已连接 | A、B 近同时编辑并 ack | 无 S01 409 模态；`ot-rev` 前进；Activity 有记录 | 本提案 C 批实现 |
| ST-S01-409-LOCAL-ONLY | 用户选择「仅本地编辑」后 PUT 冲突 | 可走 409 模态 | 须持续显示离线/409 风险文案 | 本提案 C 批实现 |

## 附录 A 增量：统一原型对齐用例 ID

| ID | 标题 | 对齐实现 | 状态 |
|---|---|---|---|
| UT-S01-SS-01 | SaveState 成功路径 | `editor_data_access` + AppBar | 本提案 C 批实现 |
| UT-S01-SS-02 | SaveState 失败 | 同上 | 本提案 C 批实现 |
| ST-S01-SS-01 | 保存态 UI | room-editor | 本提案 C 批实现 |
| ST-S01-NO-409-OT | 协作禁 409 模态 | 与 S05 联测 | 本提案 C 批实现 |
| ST-S01-409-LOCAL-ONLY | 降级后允许 409 | S01+S05 | 本提案 C 批实现 |

## 6. V1 边界

- ❌ 性能压测（V1 仅功能正确性；V2 计划）
- ❌ 模糊测试（V1 仅 happy path + 边界）
- ❌ 并发写入压力（V1 假设单写）

## 7. 对齐参考源

- `core-S01-edit-and-save-diagram.md`（时序图）
- `core-02-diagram-persistence.md`（API + 事务 + 乐观锁）
- `core-01a-table-and-field.md`（Table/Field 对象）
- `core-01-architecture-overview.md`（模块边界）
- `backend/src/diagrams_v1.rs`
- `backend/src/diagrams/service.rs`
- `backend/tests/`（integration test 位置）
- `frontend-rs/tests/wasm/`（wasm-pack test 位置）
- `logos/spec/test-results.md`（reporter 格式）

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

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
| UT-ID-GLOBAL-01 | 前端实体 id 全局唯一(1000 个 id 互不重复) | `frontend-rs/tests/entity_id_uniqueness.rs` |
| UT-ID-GLOBAL-02 | 新格式 id 绕过 max+1 解析(兼容存量加载) | `frontend-rs/src/editor_panels.rs` |
| ST-S01-01 | 编辑保存端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-02 | 导入端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-03 | 浏览器 wasm 渲染 | `frontend-rs/tests/wasm/` |
