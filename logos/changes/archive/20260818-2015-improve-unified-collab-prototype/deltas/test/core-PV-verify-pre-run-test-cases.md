## ADDED — 全文

# core-PV 验收预跑与 reporter 账本用例

> module: core | proposal: improve-unified-collab-prototype | type: verify infrastructure

## 1. 新增用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-PU-20 | 工作区不存在 `backend/test.sqlite` | 执行关联查询单测 | 使用唯一临时 SQLite 文件并初始化 schema；查询成功；不创建工作区数据库 |
| ST-PU-20 | 已存在一份正式测试账本 | 执行 `scripts/run-verify-tests.sh` | 后端、前端、ST-PU-19 与账本校验全部通过；失败时恢复旧账本；成功时只保留完整新账本 |

## 2. 既有标题式用例兼容索引

以下用例已有详细规格与真实 reporter，但旧文档只以 Markdown 标题声明。此表为 OpenLogos CLI 提供可解析索引，不改变原用例语义。

| ID | 详细规格 | reporter 来源 |
|---|---|---|
| UT-S03-01 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-02 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-03 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-04 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-05 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-06 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S03-07 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| ST-S03-01 | `core-S03-test-cases.md` | `backend/src/auth_v1.rs` |
| UT-S04-01 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-02 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-03 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-04 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-05 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-06 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-07 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-08 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-09 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-S04-10 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| ST-S04-01 | `core-S04-test-cases.md` | `backend/src/rooms_v1.rs` |
| UT-C-01 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-02 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-03 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-04 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| UT-C-05 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| ST-C-01 | `core-S05-test-cases.md` | `backend/src/collab_v1.rs` |
| ST-B-01 | `core-PC-import-export-test-cases.md` | `backend/src/phase3_bridge.rs` |
| UT-ALIGN-B01 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-ALIGN-B02 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-ALIGN-B03 | `core-PC-import-export-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-01 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-02 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |
| UT-R6-03 | `core-PE-design-system-test-cases.md` | `frontend-rs/tests/openlogos_reporter.rs` |

## 3. 验收约束

- 正式账本只允许在完整预跑成功后替换；任一阶段失败必须恢复运行前内容。
- 自动化 reporter 的 ID 必须全部出现在可解析用例表中，且所有非 `[manual]` 用例必须有最终结果。
- ST-PU-01～18 属于人工视觉与完整交互验收，不写 JSONL；ST-PU-19、ST-PU-20 必须写入 JSONL。

## 4. 本次实测结果（2026-08-18）

| 用例 | 结果 | 实测证据 |
|---|---|---|
| UT-PU-20 | PASS | 唯一临时 SQLite 文件完成 schema 初始化与空关联查询，测试结束后删除临时文件 |
| ST-PU-20 失败恢复 | PASS | 使用失败 cargo 可执行文件触发预跑中止；运行前后账本均为 35 行且 SHA-256 保持 `9ca340b9735bc750da37a5d897cfc2c65cc3aaf87a0b00d05094c11dba86252f` |
| ST-PU-20 完整预跑 | PASS | 后端 43/43 + bootstrap 1/1；前端 Rust 135 项；原型 ST-PU-19；账本 133 个定义、18 个人工、115/115 自动化结果 |
