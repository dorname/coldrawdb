# ADDED — S02 测试用例规格
# 模块：core | 提案：add-baseline-docs
# 路径：`logos/resources/test/core-S02-test-cases.md`
# 对齐参考源：`core-S02-load-shared-diagram.md` + `core-02-diagram-persistence.md`

## 1. 范围

本文件覆盖场景 S02（分享链接加载）的全部 UT 和 ST 用例规格。

**对应实现**：`backend/src/diagrams_v1.rs::read` + `backend/src/diagrams/service.rs::read` + `frontend-rs/src/editor_data_access.rs::fetch_diagram`

**对账**：
- UT-S-01..03（来自 `core-S02-load-shared-diagram.md` §8）
- ST-S-01/02（来自 `core-S02-load-shared-diagram.md` §8）

## 2. UT 用例（单元测试）

### UT-S02-01 — GET /diagrams/{id} → 200 全量数据

- **位置**：`backend/src/diagrams_v1.rs::read` 单元测试
- **前置**：已存在含 5 表 20 字段的 diagram
- **步骤**：
  1. GET `/api/v1/diagrams/{id}`
  2. 验证响应 200
  3. 验证响应 body 结构完整
- **断言**：
  - `status_code == 200`
  - `response.id == expected_id`
  - `response.title == expected_title`
  - `response.revision == 5`
  - `response.tables.length == 5`
  - 所有 table.fields 非空
  - `response.references` 数量正确

### UT-S02-02 — GET /diagrams/{nonexistent} → 404

- **位置**：`backend/src/diagrams/service.rs::read` 单元测试
- **步骤**：
  1. GET `/api/v1/diagrams/00000000-0000-0000-0000-000000000000`
  2. 验证响应 404
- **断言**：
  - `status_code == 404`
  - `response.error == "not_found"`

### UT-S02-03 — GET /diagrams/{invalid-uuid} → 400

- **位置**：`backend/src/diagrams_v1.rs::read` 单元测试
- **步骤**：
  1. GET `/api/v1/diagrams/not-a-uuid`
  2. 验证响应 400
- **断言**：
  - `status_code == 400`
  - `response.error == "invalid_id"`

### UT-S02-04 — Service::read 聚合查询正确性

- **位置**：`backend/src/diagrams/service.rs::read` 单元测试
- **前置**：已存在 diagram 含 5 表 20 字段 + 3 references + 1 area + 2 notes
- **步骤**：
  1. 调用 service::read
  2. 验证返回 Diagram 树结构
- **断言**：
  - `result.is_ok()`
  - `result.tables.length == 5`
  - 所有 table.fields 数量正确
  - `result.references.length == 3`
  - `result.areas.length == 1`
  - `result.notes.length == 2`

### UT-S02-05 — Service::read 关联查询 N+1 检查

- **位置**：`backend/src/diagrams/service.rs::read` 单元测试 + sql 计数
- **前置**：已存在 diagram 含 5 表 20 字段
- **步骤**：
  1. 启动查询计数器（拦截 SQL）
  2. 调用 service::read
  3. 验证查询次数
- **断言**：
  - V1 接受 6 个查询（diagrams + tables + fields + references + areas + notes）
  - V2 计划优化为 1-2 个 JOIN 查询

### UT-S02-06 — fetch_diagram JSON 解析

- **位置**：`frontend-rs/src/editor_data_access.rs::fetch_diagram` 单元测试
- **前置**：mock HTTP 客户端
- **步骤**：
  1. mock 返回 200 + 完整 Diagram JSON
  2. 调用 fetch_diagram
  3. 验证解析结果
- **断言**：
  - `result.is_ok()`
  - `result.tables[0].fields.length == 4`

### UT-S02-07 — fetch_diagram 网络错误处理

- **位置**：`frontend-rs/src/editor_data_access.rs::fetch_diagram` 单元测试
- **前置**：mock 返回 500
- **步骤**：
  1. mock 返回 500
  2. 调用 fetch_diagram
- **断言**：
  - `result.is_err()`
  - `error.kind == "ServerError"`

### UT-S02-08 — set_diagram 初始化撤销栈

- **位置**：`frontend-rs/src/editor_core.rs::set_diagram` 单元测试
- **前置**：内存 Diagram 状态（带 3 步 undo 历史）
- **步骤**：
  1. 调用 set_diagram(new_diagram)
  2. 验证状态变更
- **断言**：
  - `undo_stack.len() == 0`（新 diagram 重置撤销栈）
  - `redo_stack.len() == 0`
  - `dirty == false`
  - `revision == new_diagram.revision`

### UT-S02-09 — 路由参数解析

- **位置**：`backend/src/diagrams_v1.rs::read` 单元测试
- **步骤**：
  1. 用 `actix_web::test` 构造请求
  2. path: `/api/v1/diagrams/{valid_uuid}`
  3. 验证 path 参数正确解析
- **断言**：
  - handler 接收到的 id 与 URL 一致

## 3. ST 用例（场景测试 / E2E）

### ST-S02-01 — A 创建 + Share → B 通过链接加载 → 一致

- **位置**：`backend/tests/scenarios/s02.rs`
- **类型**：Rust integration test
- **步骤**：
  1. 启动后端
  2. A POST `/api/v1/diagrams` body 含 5 表 20 字段 → 创建 diagram (id=X, rev=0)
  3. B 模拟浏览器：访问 `/editor/{X}`
  4. B 触发前端 mount → fetch_diagram
  5. 验证 B 端获取的 Diagram 与 A 提交的一致
- **断言**：
  - A 创建响应 201
  - B fetch 响应 200
  - 字段数 / 字段名 / 类型 / 关系 / 区域 / 便签 全部一致
  - B 端 revision == 0
  - B 端可成功编辑并 PUT 触发 S01 流程

### ST-S02-02 — A 编辑保存后 B 加载，B 编辑触发 409

- **位置**：`backend/tests/scenarios/s02.rs`（与 S01 ST-S01-02 协同）
- **步骤**：
  1. A 创建 diagram (rev=0)
  2. A PUT (rev=0 → rev=1)
  3. B GET 加载（持有 rev=1）
  4. A 再次 PUT (rev=1 → rev=2)
  5. B 编辑后 PUT 用 rev=1
  6. 验证 B 收到 409
- **断言**：
  - 响应 status == 409
  - 响应 body.error == "revision_conflict"
  - 响应 body.current.revision == 2

### ST-S02-03 — 不存在 diagram 链接

- **位置**：`backend/tests/scenarios/s02.rs`
- **步骤**：
  1. GET `/api/v1/diagrams/{nonexistent_uuid}`
  2. 验证响应 404
  3. 验证响应 body.error == "not_found"
- **断言**：
  - `status_code == 404`
  - 响应不泄漏数据库错误细节（仅 "not_found" + message）

### ST-S02-04 — 链接格式无效

- **位置**：`backend/tests/scenarios/s02.rs`
- **步骤**：
  1. GET `/api/v1/diagrams/not-a-uuid`
  2. 验证响应 400
- **断言**：
  - `status_code == 400`
  - `response.error == "invalid_id"`

### ST-S02-05 — 网络中断后重试成功

- **位置**：`backend/tests/scenarios/s02.rs`
- **步骤**：
  1. 启动后端
  2. 第一次 GET 网络中断（mock 网络断开）
  3. 重试 GET → 网络恢复
  4. 验证第二次 GET 成功
- **断言**：
  - 第 1 次返回网络错误
  - 第 2 次返回 200
  - 客户端重试逻辑生效

### ST-S02-06 — WASM 端到端（带浏览器）

- **位置**：`frontend-rs/tests/wasm/s02.rs`
- **类型**：wasm-pack test --headless
- **步骤**：
  1. 启动 staging 后端
  2. wasm-pack test --headless --chrome
  3. 打开 `/editor/{valid_id}`
  4. 等待画布渲染
  5. 验证 DOM 中存在 table 元素
- **断言**：
  - DOM.querySelectorAll('.table-card').length == 5
  - 侧栏 Tables Tab 列表项数 == 5

## 4. 测试数据工厂

`backend/src/diagrams/test_factory.rs`（与 S01 共用）：

```rust
pub fn make_full_diagram() -> Diagram {
    Diagram {
        id: "test-id".into(),
        title: "Test Diagram".into(),
        revision: 0,
        tables: vec![/* 5 张表，每张 4 字段 */],
        references: vec![/* 3 条关系 */],
        areas: vec![/* 1 个区域 */],
        notes: vec![/* 2 个便签 */],
    }
}
```

## 5. OpenLogos Reporter

所有测试运行结果通过 `logos/spec/test-results.jsonl` 记录（与 S01 相同 schema）：

```jsonl
{"test_id": "UT-S02-01", "status": "passed", "duration_ms": 8, "timestamp": "2026-06-08T10:01:00Z"}
{"test_id": "ST-S02-01", "status": "passed", "duration_ms": 2100, "timestamp": "2026-06-08T10:01:30Z"}
```

## 6. V1 边界

- ❌ 性能压测（V1 仅功能正确性）
- ❌ 真实浏览器跨平台测试（V1 仅 headless chrome）
- ❌ 移动端浏览器（V1 仅桌面）
- ❌ SSR 预渲染（V1 纯 SPA）

## 7. 对齐参考源

- `core-S02-load-shared-diagram.md`（时序图）
- `core-02-diagram-persistence.md`（API 端点）
- `core-00-information-architecture.md`（路由）
- `backend/src/diagrams_v1.rs::read`
- `backend/src/diagrams/service.rs::read`
- `frontend-rs/src/editor_data_access.rs::fetch_diagram`
- `frontend-rs/src/editor_core.rs::set_diagram`
- `backend/tests/scenarios/s02.rs`
- `logos/spec/test-results.md`

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S02-01 | GET 存在 diagram | `backend/src/diagrams_v1.rs` |
| UT-S02-02 | GET 不存在 → 404 | `backend/src/diagrams_v1.rs` |
| UT-S02-03 | GET 全量 fields/references | `backend/src/diagrams/v1/service.rs` |
| UT-S02-04 | GET pan/zoom 保留 | `backend/src/diagrams/v1/service.rs` |
| UT-S02-05 | GET is_deleted=1 隐藏 | `backend/src/diagrams/v1/service.rs` |
| UT-S02-06 | GET 跨 revision 一致 | `backend/src/diagrams/v1/service.rs` |
| UT-S02-07 | GET 大表（>50 字段） | `backend/src/diagrams/v1/service.rs` |
| UT-S02-08 | GET 多次并发 | `backend/src/diagrams/v1/service.rs` |
| UT-S02-09 | GET 含 area/note | `backend/src/diagrams/v1/service.rs` |
| ST-S02-01 | 分享链接加载 | `backend/tests/scenarios/s02.rs` |
| ST-S02-02 | A→B 实时同步（轮询） | `backend/tests/scenarios/s02.rs` |
| ST-S02-03 | 大 diagram 加载 | `backend/tests/scenarios/s02.rs` |
| ST-S02-04 | 网络断开重试 | `backend/tests/scenarios/s02.rs` |
| ST-S02-05 | 浏览器渲染 | `frontend-rs/tests/wasm/` |
| ST-S02-06 | 并发分享会话 | `backend/tests/scenarios/s02.rs` |
