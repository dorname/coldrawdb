# Delta — core-S02-test-cases.md（diagram-api-auth）

> 模块：core | 提案：diagram-api-auth
> 目标：登记 share_token 匿名读豁免相关 UT/ST 用例，OpenLogos verify 解析用。

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

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
| UT-S02-10 | flag=on：GET 无 token 且无 share_token → 401 | `backend/src/diagrams_v1.rs` |
| UT-S02-11 | flag=on：GET share_token 匹配 → 200 匿名读（S02 豁免） | `backend/src/diagrams_v1.rs` |
| UT-S02-12 | flag=on：GET share_token 不匹配/图未开分享 → 404（不暴露存在性） | `backend/src/diagrams_v1.rs` |
| UT-S02-13 | share 铸造/轮换：轮换后旧 token 立即 404；软删图 share → 404 | `backend/src/diagrams_v1.rs` |
| ST-S02-01 | 分享链接加载 | `backend/tests/scenarios/s02.rs` |
| ST-S02-02 | A→B 实时同步（轮询） | `backend/tests/scenarios/s02.rs` |
| ST-S02-03 | 大 diagram 加载 | `backend/tests/scenarios/s02.rs` |
| ST-S02-04 | 网络断开重试 | `backend/tests/scenarios/s02.rs` |
| ST-S02-05 | 浏览器渲染 | `frontend-rs/tests/wasm/` |
| ST-S02-06 | 并发分享会话 | `backend/tests/scenarios/s02.rs` |
| ST-S02-07 | 分享匿名读全链路（编排 core-S02 v1.1.0：铸造→匿名读→401/404→轮换失效） | `backend/tests/scenarios/s02.rs` |
