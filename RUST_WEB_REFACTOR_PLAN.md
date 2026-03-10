# drawDB Web 改造实施文档（Rust 体系 + 数据库存储）

> 目标：在可回滚、可灰度的前提下，完成 drawDB Web 从“本地存储优先”到“数据库持久化优先”的迁移，并逐步把 Web 技术栈统一到 Rust。

## 1. 范围、目标、非目标

### 1.1 本次改造目标（必须达成）
1. **存储迁移到数据库**：diagram 全量数据支持数据库持久化（读/写/版本控制/恢复）。
2. **Rust 体系统一**：后端保持 Rust；前端重建为 Rust Web（WASM）实现主流程。
3. **平滑迁移**：支持旧本地数据导入，迁移期可灰度、可回滚。

### 1.2 非目标（本期不做）
- 实时多人协作（仅预留协议能力，不在本期交付）。
- 完整插件系统重构。
- 全量主题系统重写（先保证功能一致性）。

---

## 2. 现状与改造原则

### 2.1 现状判断
- 仓库当前为前端工程 + Rust backend 并存形态。
- backend 已具备实体模型与 SQL 初始化基础，可承接数据库落地。

### 2.2 改造原则
- **先数据、后界面**：先把持久化切换完成，再替换前端技术实现。
- **双轨过渡**：迁移期允许“旧前端 + 新后端”并行。
- **接口先行**：先冻结 API 和 DTO，减少前后端返工。
- **可观测优先**：所有关键链路必须可监控、可追踪、可审计。

---

## 3. 总体架构（Rust-only 导向）

## 3.1 分层
- **Client（Rust WASM）**：编辑器 UI 与交互逻辑。
- **API（Rust HTTP）**：认证（可选）、鉴权（可选）、路由、参数校验。
- **Domain（Rust）**：ER 模型规则、命令处理、冲突策略。
- **Repository（Rust）**：数据库读写、事务、批量操作。
- **DB（SQLite/PostgreSQL）**：开发环境 SQLite，生产 PostgreSQL。

## 3.2 技术边界说明
- 不接入重型第三方全栈框架。
- 可以使用 Rust 生态基础库（HTTP、ORM/SQL、serde、WASM 构建工具等）。

---

## 4. 数据库存储迁移方案（重点）

## 4.1 核心表设计（v1）
建议围绕现有实体确认以下主表与关联表：
- 主表：`diagrams`, `tables`, `fields`, `references`, `indices`, `notes`, `areas`
- 关联表：`table_link`, `diagram_link`, `indice_link`

统一字段规范：
- 主键：`id`（UUID）
- 审计：`created_at`, `updated_at`, `created_by`（可选）
- 并发控制：`revision`（每次保存 +1）
- 软删除：`is_deleted`（v1 必选）

## 4.2 事务策略
- `保存一个 diagram` = **单事务**。
- 失败即回滚，保证图内数据一致。
- 使用 `revision` 乐观锁：
  - 客户端携带 `expected_revision`
  - 服务端不匹配则返回 `409 CONFLICT`

## 4.3 存储迁移流程
1. 冻结 DDL（v1 schema）并评审。
2. 上线 migration 机制（版本化脚本）。
3. 实现全量读写 API（先正确，再优化）。
4. 增加批量 upsert 与差量写入优化。
5. 数据校验与索引优化。

## 4.4 旧数据导入
- 客户端检测本地草稿（localStorage/导出 JSON）。
- 提示用户执行“导入到数据库”。
- 导入接口支持：
  - 字段缺省值填充
  - 非法枚举容错
  - 错误明细回传

---

## 5. API 契约（v1 草案）

## 5.1 资源接口
- `POST /api/v1/diagrams`：创建 diagram
- `GET /api/v1/diagrams/{id}`：读取 diagram 全量结构
- `PUT /api/v1/diagrams/{id}`：保存（需 `expected_revision`）
- `DELETE /api/v1/diagrams/{id}`：删除/软删除
- `POST /api/v1/diagrams/import`：导入旧 JSON

## 5.2 统一响应约定
- 成功：`{ code: 0, data, request_id }`
- 失败：`{ code: <业务码>, message, request_id, details? }`
- 冲突：HTTP `409` + 冲突版本信息（`current_revision`）

## 5.3 DTO 关键字段
- `DiagramDTO`：`id`, `name`, `revision`, `tables[]`, `references[]`, `notes[]`, `areas[]`
- `TableDTO`：`id`, `diagram_id`, `name`, `x`, `y`, `fields[]`, `indices[]`
- `FieldDTO`：`id`, `table_id`, `name`, `type`, `nullable`, `default_value`, `pk`, `unique`

---

## 6. Rust Web 改造路线

## 6.1 框架选型执行方式
候选：Leptos / Yew / Dioxus。

采用 **2 周 PoC 评分制**（满分 100）：
- 编辑器交互适配（40）
- 性能（25）
- 工程可维护性（20）
- 团队学习成本（15）

得分最高者作为正式方案。

## 6.2 前端模块分层
- `editor-core`：画布坐标、节点/边模型、命令栈（undo/redo）
- `editor-render`：节点渲染、连线渲染、缩放平移
- `editor-panels`：左侧资源/右侧属性/顶部工具栏
- `data-access`：API 请求、缓存、冲突处理

## 6.3 与后端协同顺序
1. 先用最小 Rust 页面打通“加载 + 保存”链路。
2. 再实现核心交互（建表、加字段、建关联）。
3. 最后迁移高级能力（模板、主题、导出 SQL）。

---

## 7. 分阶段计划（10~12 周）

## Phase 0：方案冻结（第 1 周）
- 产物：ERD、OpenAPI 草案、迁移策略文档
- 退出条件：架构评审通过

## Phase 1：数据库落地（第 2~3 周）
- 产物：migration、repository、全量读写 API
- 退出条件：复杂 diagram 可稳定保存/读取

## Phase 2：迁移桥接（第 4 周）
- 产物：旧前端接入新 API、本地导入流程、双写开关（可选）
- 退出条件：迁移演练通过，无数据丢失

## Phase 3：Rust Web MVP（第 5~8 周）
- 产物：Rust 编辑器主流程（建模、保存、加载、导出）
- 退出条件：可替换 80% 主流程

## Phase 4：补齐与灰度（第 9~10 周）
- 产物：高级功能补齐、灰度发布、监控面板
- 退出条件：线上指标达标

## Phase 5：切流与收尾（第 11~12 周）
- 产物：切流报告、回滚预案、运维交接文档
- 退出条件：旧链路下线

---

## 8. 验收标准（可量化）

## 8.1 功能一致性
- 表/字段/索引/关系/备注/区域：增删改查全通过。
- 自动保存恢复成功率 ≥ 99.9%。

## 8.2 性能指标
- `GET /diagrams/{id}`：P95 < 300ms（中型图，内网基准）
- `PUT /diagrams/{id}`：P95 < 500ms
- 首屏可交互时间（Rust Web）不劣于旧版 10% 以上

## 8.3 稳定性指标
- 保存成功率 ≥ 99.95%
- 迁移失败率 < 0.1%
- 数据一致性错误（事务后脏数据）= 0

---

## 9. 风险与应对

1. **Rust Web 编辑器交互复杂度高**  
   应对：PoC 先验证拖拽、缩放、连线；采用命令模式降低状态复杂度。

2. **迁移期用户数据格式多样**  
   应对：导入器做 schema 版本识别与容错转换，提供失败重试包。

3. **双轨期维护成本上升**  
   应对：设置双轨时限（最多 4 周），到期强制收敛。

4. **性能与体积风险**  
   应对：建立基线，持续压测；按需拆分 WASM 包与静态资源。

---

## 10. 交付物清单

- 《数据库 Schema 设计说明（v1）》
- 《Migration 执行与回滚手册》
- 《OpenAPI 契约文档》
- 《旧数据导入规范与异常码》
- 《Rust Web 架构与编码规范》
- 《灰度发布与应急预案》
- 《最终切流复盘报告》

---

## 11. 两周内可执行任务（立即开始）

### Week 1
- 冻结 schema + OpenAPI v1
- 完成 diagram 全量保存/读取接口
- 增加 revision 冲突处理

### Week 2
- 完成旧数据导入链路
- 完成 Rust Web PoC（加载 + 保存 + 建表）
- 输出 PoC 对比结论并定型技术路线

> 执行优先级：**先打通数据库闭环，再推进 Rust Web 替换**。该顺序能最大化降低切换风险并保障业务连续性。
