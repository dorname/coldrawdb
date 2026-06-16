## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md`
> 策略：移除文件开头的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` 标记块及其紧随的 `>` 元数据行，保留正文首个一级标题以下所有内容原样。

# S01 时序图：编辑 + 自动保存（How 层 — 第 2 步：场景）

## 1. 场景描述

**用户故事**：作为开发者，我在编辑器中新建一张表（含 3 个字段），编辑器自动将变更同步到后端，刷新页面后内容不丢失。

**触发**：用户在画布上添加表 / 编辑字段 / 删除对象

**成功标志**：右上角 SaveState 变为 `Saved` + revision 自增

**覆盖范围**：CAP-CANVAS-01/02（Table/Field）+ CAP-PERSIST-01/02（创建/更新）+ 全部 11 张表写入

## 2. 参与者

| 角色 | 模块 | 文件 |
|---|---|---|
| User | — | — |
| EditorPanels | `frontend-rs/src/editor_panels.rs` | TablesTab / TableInfo |
| EditorCore | `frontend-rs/src/editor_core.rs` | `push_undo` / `mark_dirty` / `update_revision` |
| EditorDataAccess | `frontend-rs/src/editor_data_access.rs` | `save`（debounce 1s） |
| Browser HTTP | — | `fetch` API |
| BackendDiagrams | `backend/src/diagrams_v1.rs` | `create` / `update` handler |
| BackendRepository | `backend/src/repository/...` | `DiagramRepo` / `TableRepo` / `FieldRepo` |
| SQLite | — | `data/coldrawdb.db` |

## 3. 时序图

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant EP as EditorPanels
    participant EC as EditorCore
    participant EDA as EditorDataAccess
    participant HTTP as Browser Fetch
    participant BD as BackendDiagrams
    participant BR as BackendRepository
    participant DB as SQLite

    User->>EP: 单击 "+" 创建表
    EP->>EC: create_table(name="users")
    EC->>EC: push_undo() + mark_dirty()
    EC-->>EP: signal: tables.length++
    Note over EDA: debounce 1000ms<br/>（用户停止编辑后触发）
    EDA->>EDA: collect dirty state
    EDA->>HTTP: PUT /api/v1/diagrams/{id}<br/>Body: { revision, tables, ... }
    HTTP->>BD: route handler
    BD->>BD: validate body (类型 + revision)
    BD->>BR: BEGIN IMMEDIATE TRANSACTION
    BR->>DB: SELECT revision FROM diagram WHERE id=?
    DB-->>BR: current_rev=5
    BR->>BR: compare: req.revision=5 == current_rev=5 ✓
    BR->>DB: UPDATE diagram SET revision=6, ...
    BR->>DB: INSERT/UPDATE field (3 rows)
    BR->>BR: COMMIT
    BR-->>BD: ok
    BD-->>HTTP: 200 OK<br/>{ id, revision: 6, ... }
    HTTP-->>EDA: parsed response
    EDA->>EC: update_revision(6)
    EC-->>EP: signal: revision=6
    EP-->>User: SaveState="Saved"
```

## 4. 步骤详解

### 4.1 用户操作 → EditorCore

```rust
// frontend-rs/src/editor_panels.rs::on_create_table
fn on_create_table(name: String) {
    let table = Table { id: uuid::Uuid::new_v4().to_string(), name, ..default() };
    self.core.create_table(table);  // 调用 editor_core
}
```

### 4.2 EditorCore 处理

```rust
// frontend-rs/src/editor_core.rs
fn create_table(&mut self, table: Table) {
    self.undo_stack.push(UndoOp::CreateTable(table.clone()));
    self.diagram.tables.push(table);
    self.dirty = true;
    self.save_signal.notify();  // 唤醒 debounce 计时器
}
```

### 4.3 EditorDataAccess debounce

```rust
// frontend-rs/src/editor_data_access.rs
async fn save_loop(&self) {
    let mut debounce = tokio::time::interval(Duration::from_millis(1000));
    debounce.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        debounce.tick().await;
        if self.core.is_dirty() {
            self.save().await;  // HTTP PUT
        }
    }
}
```

### 4.4 Backend handler

```rust
// backend/src/diagrams_v1.rs
#[put("/api/v1/diagrams/{id}")]
async fn update(
    path: Path<String>,
    body: Json<DiagramUpdateRequest>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    diagrams::service::update(&pool, &id, body.into_inner()).await
        .map(|diagram| HttpResponse::Ok().json(diagram))
        .map_err(AppError::from)
}
```

### 4.5 Repository 事务

```rust
// backend/src/diagrams/service.rs
pub async fn update(pool: &Pool, id: &str, req: DiagramUpdateRequest) -> Result<Diagram> {
    let mut tx = pool.begin().await?;
    let current = diagram_repo::find(&mut *tx, id).await?
        .ok_or(AppError::NotFound)?;
    if current.revision != req.revision {
        return Err(AppError::Conflict { current });
    }
    diagram_repo::update(&mut *tx, id, &req).await?;
    field_repo::replace_all(&mut *tx, id, &req.fields).await?;
    // ... 其他表
    tx.commit().await?;
    diagram_repo::find(pool, id).await
}
```

## 5. 错误处理

### 5.1 409 Conflict（最复杂分支）

```mermaid
sequenceDiagram
    participant EDA
    participant BD
    participant EC
    participant UI as Conflict Dialog

    EDA->>BD: PUT /diagrams/{id} (rev=5)
    BD-->>EDA: 409 Conflict { current_diagram (rev=7) }
    EDA->>EC: conflict_detected(current_diagram)
    EC-->>UI: show_dialog(local + remote)
    UI-->>EC: user_choice (reload / force / cancel)
    alt reload
        EC->>EDA: fetch_latest()
        EDA-->>EC: remote (rev=7) overwrites local
    else force
        EC->>EDA: save_with_rev(current_rev + 1)
    else cancel
        EC->>EC: mark_conflict_pending
    end
```

### 5.2 404 Not Found

- diagram 不存在（被他人删除）
- 弹"该 diagram 已被删除"提示
- 提供"返回列表"按钮

### 5.3 网络中断

- 指数退避重试（1s / 2s / 4s）
- 3 次失败后右上角"重试"按钮
- SaveState 变 `Error`

### 5.4 后端 validation 错误

- 400 Bad Request + 详细错误消息
- 弹 toast 提示
- 不更新本地状态

## 6. 性能与资源

| 资源 | 数量 |
|---|---|
| HTTP 请求 | 1（PUT 全量） |
| 数据库事务 | 1（IMMIDIATE） |
| 数据库写语句 | N（diagram + N tables + M fields + ...） |
| WASM 内存增量 | 撤销栈 +1 步（≈ 1 KB） |
| 网络流量 | 单次 PUT ≈ 表格数 × 1 KB + 字段数 × 0.5 KB |

## 7. 测试用例映射

| TC ID | 场景 | 对应 S01 步骤 |
|---|---|---|
| UT-P-01 | 创建空 diagram | 4.1 - 4.5（首次 PUT → 201） |
| UT-P-02 | 创建含 5 表 20 字段 | 4.5（事务写 11 张表相关行） |
| UT-P-03 | PUT 带正确 revision → 200 | 4.4 - 4.5 |
| UT-P-04 | PUT 带过期 revision → 409 | 5.1 |
| UT-P-05 | DELETE → 级联删除 | （本场景不覆盖；S02 涉及） |
| ST-P-01 | 端到端：编辑 → 自动保存 → 重新加载 → 一致 | 完整链路 |

## 8. V1 边界

- ❌ 局部 PUT（V1 全量更新；V2 计划改为 PATCH 局部）
- ❌ 实时协作同步（V1 仅单人；V2 OT 引擎）
- ❌ 后端 schema diff（V1 直接 replace）
- ❌ WASM 端 ORM（V1 仅后端 ORM）

## 9. 对齐参考源

- `core-01-architecture-overview.md`（数据流 §5）
- `core-02-diagram-persistence.md`（API + 事务 + 乐观锁）
- `core-01a-table-and-field.md`（Table / Field 对象结构）
- `backend/src/diagrams_v1.rs`（5 端点 Rust 路由）
- `backend/src/diagrams/service.rs`（事务逻辑）
- `frontend-rs/src/editor_core.rs`（状态机）
- `frontend-rs/src/editor_data_access.rs`（HTTP + debounce）
- `docs/drawdb-capability-checklist.md` §2.5

