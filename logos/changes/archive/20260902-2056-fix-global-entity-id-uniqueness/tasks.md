# 实现任务

> 变更类型：代码级修复（无 delta、无 deploy）。proposal.md 声明「是否需要部署：否 / 是否需要 smoke：否」，故无 `[delta]` / `[deploy]` section。

## [code] 代码实现

- [x] `frontend-rs/src/editor_core.rs` 新增全局唯一实体 id 生成器：`new_entity_id(prefix) -> String`（`{prefix}-{16位hex}`），wasm 侧随机源 `js_sys::Math::random` + `js_sys::Date::now` 种子 + `AtomicU64` 计数器，非 wasm 侧 `#[cfg]` 走 `std::time` + 原子计数器，保证 host 可测
- [x] `frontend-rs/src/editor_panels.rs` 替换建表 id（`auto-{n}` → `new_entity_id("auto")`）；表默认字段保持 `{table_id}-field-id` 不变（table_id 唯一后自然唯一）
- [x] `frontend-rs/src/editor_panels.rs` 替换新增字段 id（`auto-{n}` → `new_entity_id("auto")`）
- [x] `frontend-rs/src/editor_panels.rs` 替换关系 id（`ref-{n}` → `new_entity_id("ref")`）
- [x] `frontend-rs/src/editor_panels.rs` 替换区域/便签 id（`new_default_area` / `new_default_note`：`area-{seq}`/`note-{seq}` → `new_entity_id("area")`/`new_entity_id("note")`；seq 参数保留用于命名与层叠落位）
- [x] 更新受影响 UT 断言：凡断言 `auto-N` / `ref-N` / `area-N` / `note-N` 具体字面量的用例改为「前缀匹配 + 互不相同」断言；确认 enum/type stub（`enum-auto-N`/`type-auto-N`，不持久化）无需改动
- [x] 新增回归 UT（含 OpenLogos `verify_reporter::report_pass`，写入 `logos/resources/verify/test-results.jsonl`）：连续生成 1000 个实体 id 互不重复；新格式 id 不被 `parse_num_suffix` 解析（`next_id_from_store` 返回 0，兼容存量加载）
- [x] 回归验证：`cd frontend-rs && cargo test --lib` 与集成测试全绿；`cd backend && cargo test` 全绿（确认后端零改动）

## 完成记录（2026-08-27）

- 全部 7 项完成。grep 确认既有测试不依赖旧 `auto-N` 字面量，故第 6 项实际为「确认无需改写 + 由新增用例覆盖前缀/互异断言」。
- 回归 UT 拆分：`UT-ID-GLOBAL-01`（4 类前缀 × 1000 个 id 唯一性 + 格式，`tests/entity_id_uniqueness.rs` 自行上报）；`UT-ID-GLOBAL-02`（新格式 id 绕过 max+1 解析 + 区域/便签前缀互异，`src/editor_panels.rs` 测试模块断言，`openlogos_reporter` 批量上报）。
- 测试结果：backend / frontend-rs / mcp-server 三 crate `cargo test` 全部通过（frontend-rs 190 passed / 15 suites）；账本 201 行，含 UT-ID-GLOBAL-01/02 pass 记录。
