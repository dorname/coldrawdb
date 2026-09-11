# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/api/` — bridge.yaml：`POST /api/v1/bridge/import/connect` 请求体新增可选 `schema` 字段
- [x] **验证 API YAML** — delta 中所有 YAML 片段符合 OpenAPI 3.x 规范
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-01d-import-export.md：ImportDrawer 数据库来源 schema 输入（仅 PG）+ 导入合并 ID 重键条款
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-03-bridge-io.md：§13 请求体新增可选 `schema`（默认 `public`，仅 postgres 生效）
- [x] 产出 delta 文件到 `deltas/test/` — core-PC-import-export-test-cases.md：新增 UT-PC-20（合并 ID 重键唯一性）/ UT-PC-21（PG schema introspection）/ UT-PC-22（前端 schema 输入锚点）/ ST-PC-06（二次导入+保存 e2e）；MODIFIED 登记 UT-PC-08/09 断言口径

## [code] 代码实现

批次一：导入合并 ID 重键（覆盖 UT-PC-20 + UT-PC-08/09 口径修正 + ST-PC-06）
- [x] 前端：`merge_import_into_store` 对新表/字段/关系做 `new_entity_id` 重键，reference 四端点按映射改写
- [x] 前端：UT-PC-20 重键唯一性/一致性 UT；UT-PC-08/09 断言改为重键口径
- [x] e2e：ST-PC-06 连续两次数据库导入后保存成功（mock 端点 + 真实保存链路断言）

批次二：PG schema 定位（覆盖 UT-PC-21/22）
- [x] 后端：`ImportConnectReq` 新增可选 `schema`；`introspect` / `introspect_postgres` 四查询按 schema 精确过滤（默认 `public`）
- [x] 后端：UT-PC-21 嵌入式 PG 双 schema 夹具（同名表隔离 + 指定 schema 只导该 schema）
- [x] 前端：ImportDrawer 引擎选 postgres 时显示 schema 输入并随请求发送（sqlite 不显示不发送）+ UT-PC-22 锚点
- [x] e2e：ST-PC-06 同时覆盖 schema 参数透传断言

每批均含：业务代码 + UT/ST 测试代码 + 写入 `logos/resources/verify/test-results.jsonl` 的 OpenLogos reporter
