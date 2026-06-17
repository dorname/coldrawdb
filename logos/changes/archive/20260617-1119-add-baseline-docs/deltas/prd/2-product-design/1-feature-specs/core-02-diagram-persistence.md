## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md`
> 策略：移除文件开头的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` 标记块及其紧随的 `>` 元数据行，保留正文首个一级标题以下所有内容原样。

# 图表持久化规格（V1）

## 1. 持久化对象

```ts
interface Diagram {
  id: string;                  // 服务端 UUID（自增 BIGINT 映射为字符串）
  title: string;               // diagram 标题
  revision: number;            // 乐观锁版本号（自增）
  createdAt: string;           // ISO8601
  updatedAt: string;           // ISO8601
  tables: Table[];             // 含 fields / indices（V1 字段级，不为索引独立建表）
  references: Relationship[];  // 关系
  areas: Area[];               // 区域
  notes: Note[];               // 便签
  enums: Enum[];               // V1 仅前端 state
  customTypes: CustomType[];   // V1 仅前端 state
}
```

> **V1 持久化覆盖范围**：table + field + reference + area + note + diagram_link + table_link；index / enum / customType **不**进入后端。

## 2. API 端点（5 个 diagrams 端点）

| 端点 | 方法 | 用途 | 状态码 |
|---|---|---|---|
| `/api/v1/diagrams` | POST | 创建新 diagram（含内嵌 tables/references/areas/notes） | 201 / 400 / 409 |
| `/api/v1/diagrams/{id}` | GET | 读取完整 diagram | 200 / 404 |
| `/api/v1/diagrams/{id}` | PUT | 全量更新（含 revision 乐观锁） | 200 / 400 / 404 / 409 |
| `/api/v1/diagrams/{id}` | DELETE | 级联删除 | 204 / 404 |
| `/api/v1/diagrams/import` | POST | 从 JSON 导入（drawdb JSON 格式） | 201 / 400 |

完整 OpenAPI 规格见 `deltas/api/diagrams.yaml`。

## 3. 持久化事务策略

- **创建 / 更新 / 删除**：单事务内写入 `diagram` + 所有关联（`field` / `reference` / `area` / `note` / `table_link` 等）
- **级联**：删除 diagram → 级联删除所有 fields / references / areas / notes / table_link / diagram_link
- **隔离级别**：SQLite 默认（SERIALIZABLE）；backend 显式 `BEGIN IMMEDIATE TRANSACTION` 防止并发写入
- **错误处理**：事务回滚后返回 400 + 详细错误

## 4. Revision 乐观锁 + 409 冲突语义

### 4.1 协议

请求：`PUT /api/v1/diagrams/{id}` 的 body 中 `revision` 字段为客户端最后已知的版本号。

服务端：
- 读当前 `revision`（如 `current_rev = 5`）
- 比较请求 `revision` 与 `current_rev`
- 不等 → 返回 `409 Conflict` + 当前 `diagram` 全量（让客户端可重载 / merge）

### 4.2 客户端行为（V1）

1. 加载 diagram → 缓存 `revision`
2. 编辑（添加/删除表等）→ 本地累积变更
3. 自动保存（debounce 1s）→ `PUT /api/v1/diagrams/{id}` 带 `revision`
4. 服务端返回 200 → 更新本地 `revision = response.revision`
5. 服务端返回 409 → 弹出冲突对话框：
   - "本地有未保存的修改" + "远端已被他人更新"
   - 选项 A：**重新加载**（丢弃本地）
   - 选项 B：**保留本地**（强制 `revision=current_rev+1` 覆盖）— 仅单人场景
   - 选项 C：**取消**（保留冲突标记，用户手动合并）

### 4.3 多人场景（V1 限制）

V1 不实现实时协作；同一 diagram 同时被多个浏览器编辑会触发 409。**V2 计划**：OT 引擎在客户端合并，无需 409 弹窗。

## 5. JSON 导入（drawdb 兼容）

### 5.1 导入流程

`POST /api/v1/diagrams/import` body 为 drawdb 导出的 JSON：

```json
{
  "title": "Imported Schema",
  "tables": [...],
  "references": [...],
  "areas": [...],
  "notes": [...]
}
```

### 5.2 字段映射

| drawdb JSON | coldrawdb V1 | 说明 |
|---|---|---|
| `tables[].x / y` | ✅ | 直接映射 |
| `tables[].color` | ⚠️ 前端 state | V1 不持久化 |
| `tables[].indices` | ❌ | V1 导入时丢弃 |
| `tables[].enums` | ❌ | V1 导入时丢弃 |
| `references[]` | ✅ | 1:1 映射 |
| `subject_areas` → `areas` | ✅ | 字段名转换 |

### 5.3 导入日志

`POST /api/v1/diagrams/import` 成功后返回 `task_id`（导入日志 id）。通过 `GET /api/v1/bridge/import/local/logs` 查看所有导入日志。

## 6. 与后端实体的对账（11 张表）

| 前端对象 | 后端表 | 字段映射 |
|---|---|---|
| `Diagram.id` | `diagram.id` | UUID |
| `Diagram.title` | `diagram.title` | VARCHAR |
| `Diagram.revision` | `diagram.revision` | BIGINT，自增 |
| `Diagram.createdAt` | `diagram.created_at` | TIMESTAMP |
| `Diagram.updatedAt` | `diagram.updated_at` | TIMESTAMP |
| `Table` | `table` + `field` | 表元数据 + 字段行 |
| `Table.fields` | `field` | 关联到 `table.id` |
| `Table.indices` | （V1 不持久化） | 仅前端 |
| `Table.enums` | （V1 不持久化） | 仅前端 |
| `Relationship` | `reference` | start/end table_id + field_id |
| `Area` | `area` | x/y/width/height/color |
| `Note` | `note` | x/y/content |
| `Diagram ↔ Table 关联` | `table_link` | 多对多（drawdb 用） |
| `Diagram ↔ Diagram 关联` | `diagram_link` | 多对多（drawdb 用） |
| `Index ↔ Field 关联` | `indice_link` | V1 不写入 |
| `Todo`（导入任务） | `task` | status / message / created_at |

> 11 张表逐项对账：见 `deltas/database/coldrawdb-v1.sql`。

## 7. 自动保存策略

| 参数 | 值 | 说明 |
|---|---|---|
| debounce 间隔 | 1000 ms | 编辑停止 1s 后触发 PUT |
| 失败重试 | 指数退避（1s / 2s / 4s，最多 3 次） | 失败后右上角"重试"按钮 |
| 状态指示 | `idle / saving / saved / error` | 顶部 SaveState 指示器 |
| 网络中断 | 暂停保存，恢复后立即重试 | Service Worker 不支持（V1） |

## 8. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-P-01 | 创建空 diagram → POST 201，revision=0 |
| UT-P-02 | 创建含 5 表 20 字段 → 写入 11 张表中对应行 |
| UT-P-03 | PUT 带正确 revision → 200，revision 自增 |
| UT-P-04 | PUT 带过期 revision → 409，返回远端当前状态 |
| UT-P-05 | DELETE → 级联删除所有 fields / references / areas / notes |
| UT-P-06 | 导入 drawdb JSON → 创建 diagram + 关联行 |
| ST-P-01 | 端到端：编辑 → 自动保存 → 重新加载 → 一致 |
| ST-P-02 | 端到端：A 编辑保存后 B 加载，B 编辑触发 409 |

## 9. V1 边界

- ❌ 实时协作（多人同时编辑无需 409）— V2 计划
- ❌ 离线保存（V1 强依赖 API 可达）
- ❌ 自动冲突合并（V1 弹窗让用户决策）
- ❌ Index / Enum / CustomType 持久化（V1 仅前端）
- ❌ Diagram 历史版本（V1 仅 latest revision）
- ❌ 软删除（V1 直接 DELETE，无回收站）

## 10. 对齐参考源

- drawdb §2.5 持久化语义
- drawdb `src/utils/saveToLocal.js`（drawdb 客户端 localStorage 策略）
- `backend/src/diagrams_v1.rs`（5 端点 Rust 路由）
- `backend/src/diagrams/`, `backend/src/fields/`, `backend/src/references/`, `backend/src/areas/`, `backend/src/notes/`（5 个领域子模块）
- `backend/src/tables/` + `backend/src/indices/`（含 table_link / indice_link 关联表）
- `backend/init.sql`（11 张表 DDL）
- `database_design.json`（字段命名对账）
- `docs/drawdb-capability-checklist.md` §2.5

