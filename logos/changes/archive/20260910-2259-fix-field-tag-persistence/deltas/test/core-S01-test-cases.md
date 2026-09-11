# Delta — core-S01-test-cases.md（登记 UT-S01-11：field.tag 持久化往返）

## ADDED — UT-S01-11 — field.tag 保存/加载往返

### UT-S01-11 — field.tag 保存/加载往返

- **位置**：`backend/src/diagram_persistence.rs` 单元测试（照 0007 dict_code 往返测试模式）
- **前置**：内存构造 DiagramFull，字段 `tag = "核心"`（非空 tag）
- **步骤**：
  1. 保存 diagram（规范化写入 `field` 表，migration 0008 起含 `tag` 列）
  2. 重新加载同一 diagram
  3. 验证字段 `tag` 完整恢复
  4. 旧库兼容：无 `tag` 值（DEFAULT `''` 回填）加载为 `""` 不报错
- **断言**：
  - 保存→加载后 `loaded.tables[0].fields[0].tag == "核心"`
  - 存量行（未显式写 tag）加载后 `tag == ""`
  - reporter 登记 `UT-S01-11`

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
| UT-ID-GLOBAL-01 | 前端实体 id 全局唯一(1000 个 id 互不重复) | `frontend-rs/tests/entity_id_uniqueness.rs` |
| UT-ID-GLOBAL-02 | 新格式 id 绕过 max+1 解析(兼容存量加载) | `frontend-rs/src/editor_panels.rs` |
| ST-S01-01 | 编辑保存端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-02 | 导入端到端 | `backend/src/diagrams_v1.rs::tests` |
| ST-S01-03 | 浏览器 wasm 渲染 | `frontend-rs/tests/wasm/` |
