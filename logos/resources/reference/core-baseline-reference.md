# coldrawdb 项目基线参考

> 模块：core | 路径：`logos/resources/reference/core-baseline-reference.md`
> 用途：新成员快速上手、日常开发速查、术语与模块索引

---

## 1. 术语表

| 术语 | 说明 |
|---|---|
| core | 当前唯一模块，包含编辑器全部核心功能 |
| DBER | Database Entity Relationship，数据库实体关系图 |
| diagram | 一个完整的 ER 图，包含表、字段、关系、区域、注释 |
| revision | 服务端乐观锁版本号，PUT 时通过 `expected_revision` 防止覆盖 |
| dirty | 本地状态相对服务端是否有未保存变更 |
| debounce | 前端 1s 静默期后触发自动保存 |
| bridge | 桥接层，负责 SQL / DBML / JSON 等格式导入导出 |
| WASM | WebAssembly，前端 Rust 代码编译目标 |
| trunk | Rust WASM 构建与开发服务器工具 |

---

## 2. 模块清单

### 2.1 前端模块（`frontend-rs/src/`）

| 文件 | 职责 |
|---|---|
| `editor_data_access.rs` | HTTP 客户端（diagrams / bridge API）、自动保存调度 |
| `editor_core.rs` | Diagram 状态机、RwSignal、Undo/Redo、DebounceTrigger |
| `editor_panels.rs` | TopMenuBar / Toolbar / LeftPanel / RightPanel / 模态框 |
| `editor_render.rs` | Canvas 渲染、表/字段/关系/区域/注释绘制 |
| `lib.rs` | 应用入口、pathname 解析、debug 测试钩子 |
| `editor_panels.rs` | TopMenuBar / Toolbar / LeftPanel / RightPanel / 模态框 |
| `editor_render.rs` | Canvas 渲染、表/字段/关系/区域/注释绘制 |
| `lib.rs` | 应用入口、pathname 解析、debug 测试钩子 |

### 2.1.1 V2 前端布局层（redesign-phase-a/b/c，2026-06-14）

| 模块 | 文件 | 职责 |
|---|---|---|
| AppBar | `frontend-rs/src/components/app_bar.rs`（E1 重构） | 顶栏（标题 + 主菜单 + 全局操作） |
| ToolRail | `frontend-rs/src/components/tool_rail.rs`（E1 重构） | 左侧工具轨（6 个工具按钮） |
| Inspector | `frontend-rs/src/components/inspector.rs`（E1 重构） | 右侧属性编辑面板 |
| ModalRoot | `frontend-rs/src/components/modal_root.rs`（E1 重构） | 6 个模态（New/Open/Share/Rename/Settings/Confirm） |
| Drawer | `frontend-rs/src/components/io_drawer.rs`（Phase C） | 导入 / 导出抽屉 |

### 2.1.2 设计系统层（redesign-phase-d/e，2026-06-15）

| 组件 | 规格文件 | 状态 |
|---|---|---|
| Design Tokens | `core-07-design-tokens.md` | E1 ✅ |
| Icon Library | `core-08-icon-library.md` | E2 ✅ |
| Core Components | `core-09-core-components.md` | E3 ✅ |
| Monaco CodeEditor | `core-0a-code-editor.md` | E4 ✅ |
| Dark Mode | `core-0b-dark-mode.md` | E5 ✅ |
| Motion | `core-0c-motion.md` | E6 ✅ |


### 2.2 后端模块（`backend/src/`）

| 模块 | 职责 |
|---|---|
| `diagrams_v1.rs` | `/api/v1/diagrams/*` REST 端点、409 冲突 |
| `phase3_bridge.rs` | `/api/v1/bridge/*` 导入导出端点 |
| SeaORM 实体 | `diagram`、`table`、`field`、`reference`、`area`、`note`、`indice`、`task` 等 |

---

## 3. 开发环境速查

### 3.1 本地启动

```bash
# 后端（端口 3000）
cd backend
cargo run

# 前端（端口 8080）
cd frontend-rs
trunk serve
```

### 3.2 常用验证

```bash
# 后端健康检查
curl http://127.0.0.1:3000/

# 创建 diagram
curl -X POST http://127.0.0.1:3000/api/v1/diagrams \
  -H 'Content-Type: application/json' \
  -d '{"name":"demo"}'
```

### 3.3 常用测试

```bash
# 前端单元测试
cd frontend-rs && cargo test --lib

# e2e smoke（需先启动前后端）
node frontend-rs/scripts/e2e-smoke.mjs
```

---

## 4. 重要文件索引

| 用途 | 路径 |
|---|---|
| 项目配置 | `logos/logos.config.json` |
| 资源索引 | `logos/logos-project.yaml` |
| AI 指令 | `AGENTS.md` |
| 用途 | 路径 |
|---|---|
| 项目配置 | `logos/logos.config.json` |
| 资源索引 | `logos/logos-project.yaml` |
| AI 指令 | `AGENTS.md` |
| 历史归档索引 | `logos/changes/archive/`（含 add-frontend-completeness / redesign-phase-a~e 等 15 个已归档提案） |
| 当前活跃变更 | `logos/changes/<slug>/`（如 `add-baseline-docs`） |

### 4.1 最近归档的设计系统类变更（2026-06）

| 提案 slug | 阶段 | 关键产出 |
|---|---|---|
| `redesign-phase-a-layout` | Phase A（V2 布局） | AppBar + ToolRail + Inspector + ModalRoot 4 容器 + z-index 体系 |
| `redesign-phase-b-relationship` | Phase B | 关系工具栏 + Tooltip / Popover |
| `redesign-phase-c-import-export` | Phase C | IO 抽屉（替代 V1 Import 模态） |
| `redesign-phase-d-command-code` | Phase D | Command Palette + Code View（Phase D 已 archive，E4 Monaco 升级版生效） |
| `redesign-phase-e-design-system-migration` | Phase E | E1–E6 设计系统迁移（tokens / icons / components / Monaco / dark mode / motion） |

| 项目说明 | `README.md` |
| 验收结果 | `logos/resources/verify/test-results.jsonl` |
| smoke 报告 | `logos/spec/smoke-report.md` |
| 当前活跃变更 | `logos/.openlogos-guard` |
| 变更提案目录 | `logos/changes/` |
| 已归档变更 | `logos/changes/archive/` |

---

## 5. OpenLogos 快速链接

- 查看下一步：`openlogos next`
- 查看状态：`openlogos status`
- 创建变更：`openlogos change <slug>`
- 合并变更：`openlogos merge <slug>`
- 验收：`openlogos verify <slug>`
- 归档：`openlogos archive <slug>`

---

## 6. 最近归档变更

- `add-frontend-completeness`
- `fix-modal-overlay-blocking`
- `fix-add-frontend-stub-leftover`

