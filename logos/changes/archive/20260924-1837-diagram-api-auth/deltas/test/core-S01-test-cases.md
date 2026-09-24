# Delta — core-S01-test-cases.md（diagram-api-auth）

> 模块：core | 提案：diagram-api-auth
> 目标：登记鉴权强制（flag=on）相关 UT/ST 用例，OpenLogos verify 解析用。

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
| ST-S01-03 | 浏览器 wasm 渲染 | `frontend-rs/tests/wasm/` |
| ST-S01-AUTH-01 | 登录态编辑保存全链路（编排 core-S01 v1.1.0：匿名 401 → 登录 CRUD → 409 → 删除） | `backend/tests/scenarios/s01.rs` |
