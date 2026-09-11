# 变更提案：fix-global-entity-id-uniqueness

> module: core | created: 2026-08-27

## 变更原因

**线上 Bug（有完整证据链）**：新账户创建房间（新建空白模型）后，在编辑器中创建第一张表，自动保存永远失败（AppBar 显示「保存失败 · rev 0」）。

根因：table/field/reference/area/note 的 id 在后端 DB 是**全局单列主键**，而前端按「当前 diagram store 内 max+1」的局部计数器生成 `auto-N` / `ref-N` / `area-N` / `note-N` 形式的 id。任何新 diagram（新账户为典型场景）都会从 `auto-1` 重新计数；保存时后端 `INSERT INTO "table"(id='auto-1',…)` 撞上其他 diagram 已占用的全局主键 → `UNIQUE constraint failed` → 事务回滚 → HTTP 500 → 前端落入「保存失败」兜底分支。

证据（2026-08-27 本地库实测）：
- diagram `7498748549979574549`（数据模型评审）存在且 `revision=0` → 排除 404/409；
- 旧 diagram `7497136689798844428`（rev 76）已通过 `diagram_link` 占用全局 `"table"` 表的 `auto-1/auto-2/auto-4/auto-9/auto-10`；
- 前端 `editor_panels.rs:6183-6189`：空 store 时 `next_id=1` → 第一张表 id=`auto-1`，字段 id=`auto-1-field-id`；
- 后端 `diagram_persistence.rs:480-491`：`purge` 只清当前 diagram 链接的实体，随后 INSERT 撞全局主键 → `error/mod.rs:28` 映射 HTTP 500。

注：上一提交 `d0a63e7` 只修复了「同一 diagram 内」的 id 冲突，未覆盖跨 diagram 的全局冲突。

## 变更类型

代码级修复（代码 + 重新验收 + 部署影响分析）

## 变更范围

- 影响的需求文档：无（实体 id 生成规则不是需求级事实；主原型 `core-01-editor-prototype.html` addTable 使用 `<name>-<时间戳尾4位>` 的唯一 id，`auto-N` 纯为实现侧发明）
- 影响的功能规格：无（规格中无 `auto-N` id 字面量断言，grep 命中均为 auto-save/auto-fit 等无关词）
- 影响的业务场景：S01（编辑并保存 diagram——保存链路）、S04（房间生命周期——新房间新图触发路径）、S05（OT 协作——op 携带实体 id，需确认生成规则变更对 op 应用链路无影响）
- 影响的部署方案：无
- 影响的 API：无（`PUT /api/v1/diagrams/{id}` 契约不变）
- 影响的 DB 表：无 schema 变更（`table`/`field`/`reference`/`area`/`note` 全局主键保持不变，存量 `auto-N` 数据原样保留，新旧 id 共存）
- 影响的编排测试：无（scenario 编排为 API 级，不感知前端 id）
- 影响的 smoke 测试：无

**代码影响面**（`frontend-rs/`）：
- `src/editor_core.rs`：新增全局唯一实体 id 生成器
- `src/editor_panels.rs`：6 处 id 生成点——建表（L6185）、表默认字段（L6189）、新增字段（L6340）、关系（L6434）、区域（`new_default_area` L906）、便签（`new_default_note` L919）
- 受影响 UT：断言 `auto-N` / `ref-N` / `area-N` / `note-N` 字面量的用例改为前缀+唯一性断言

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 WASM 代码修复，本地开发环境重新构建即生效；当前项目处于开发阶段，无独立部署节点
- 影响环境：无
- 是否涉及数据迁移：否（存量 `auto-N` 数据原样保留并继续可用；新旧 id 共存无冲突）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: false            # 不触及任何界面/交互，仅 id 生成策略
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述

前端实体 id 生成从「图内局部计数器」改为「全局唯一随机 id」：在 `editor_core.rs` 新增统一生成器 `new_entity_id(prefix)`，产出 `{prefix}-{16位hex随机串}`（如 `auto-3f9a2c8e1b7d40f6`），随机源 wasm 侧取 `js_sys::Math::random` + 时间种子 + 原子计数器，非 wasm（host 单测）侧走 `std::time` + 原子计数器，保证 `cargo test --lib` 可测。保留现有前缀（`auto-`/`ref-`/`area-`/`note-`）以兼容 data-testid、OT op 与大部分字符串断言；表默认字段沿用 `{table_id}-field-id` 格式（table_id 唯一后字段 id 自然唯一，零改动）。

替换 6 处生成点后，`next_id`/`next_id_from_store` 不再承担实体 id 分配（仅存留于不持久化的 enum/type stub），`parse_num_suffix` 对新 id 解析失败返回 `None`，不影响存量数据加载。

**已否决的备选方案**：① 后端保存时对冲突 id remap——破坏 reference 引用一致性；② DB 改复合主键 `(diagram_id, id)`——迁移成本高且违背 `diagram_link` 实体复用设计；③ 后端为实体分配雪花 id——每实体一次 API 往返，破坏编辑器离线/即时性。

**范围外（遗留风险，另案评估）**：bridge/import 路径两次导入同一 drawdb 文件仍可能撞后端全局主键（由 DB 单列主键 schema 决定）；enum/type stub 的 `enum-auto-N`/`type-auto-N` 不参与持久化，维持现状。
