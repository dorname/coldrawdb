## ADDED — §6 fix-issues-70-73 真实执行入口

以下条目只登记执行来源，不新增或复制用例语义；详细验收见各详细规格。所有 UT/ST 必须由对应测试函数或真实脚本调用 OpenLogos reporter，禁止仅在声明式 `PASS_IDS` 中添加 ID 冒充执行。

| ID | 详细规格 | 真实执行入口 |
|---|---|---|
| UT-MCP-41 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| UT-MCP-42 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| UT-MCP-43 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| UT-MCP-44 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| UT-MCP-45 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| UT-MCP-46 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| UT-MCP-47 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| UT-MCP-48 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| UT-MCP-49 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| ST-MCP-18 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import -- --ignored` 或预跑脚本中等价、默认启用的真实隔离 backend+stdio 测试 |
| ST-MCP-19 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| ST-MCP-20 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c12_auth_session` |
| ST-MCP-21 | `core-S06-test-cases.md` | `cargo test --manifest-path mcp-server/Cargo.toml --test c11_chunked_import` |
| UT-C-11 | `core-S05-test-cases.md` | `cargo test --manifest-path backend/Cargo.toml ut_c11` |
| UT-PB-38 | `core-PB-relationship-test-cases.md` | `cargo test --manifest-path frontend-rs/Cargo.toml --lib ut_pb_38` |
| ST-PB-22 | `core-PB-relationship-test-cases.md` | `SPEC_PARITY_ONLY=ST-PB-22 node frontend-rs/scripts/test-spec-parity-d.mjs` |

### 真实入账约束

1. `scripts/run-verify-tests.sh` 必须默认执行上述测试目标；若真实隔离服务测试使用 `#[ignored]`，预跑脚本必须显式带 `--ignored` 运行，不能在完整验收中静默跳过。
2. ST-MCP-18/20/21 必须断言真实 backend 与 stdio 子进程已启动并实际通信；ST-MCP-18 另断言 54/863/76 精确数量与逐调用参数预算。仅凭 mock HTTP、静态源码锚点或 0ms 声明式记录不得标记 PASS。
3. ST-PB-22 必须由 parity-d 的真实 `run(["ST-PB-22"])` 路径上报，并包含保存请求与重载断言；不得加入声明式 ST_PASS 兜底。
4. reporter 记录必须来自本次执行并携带实测 duration；真实 backend/浏览器 ST 的 duration 必须为非零值，快速 UT 可因计时精度为 0。失败写 FAIL。不得通过复制历史 JSONL、修改时间戳或运行 reporter 单项测试代替业务断言。
5. 预跑仍遵守既有账本事务约束：任一阶段失败恢复运行前账本，全部成功后才替换为完整新账本。
