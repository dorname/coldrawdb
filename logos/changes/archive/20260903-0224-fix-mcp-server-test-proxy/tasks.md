# 实现任务 — fix-mcp-server-test-proxy

> 配套：`proposal.md`
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/人工验证条目；回滚伪修复（c2_read_tools.rs 的 `payload.len()` → `payload.as_bytes().len()` 是 no-op——`String::len()` 本就返回字节数）

## [code] 代码实现

### 修复点（外环强制约束：测试与生产均不应走环境代理连 127.0.0.1 mock/本地后端）

- [ ] `mcp-server/src/api.rs:15` `ApiClient::new()`：`Client::builder()` 加 `.no_proxy()` 调用，显式禁用 reqwest 环境代理
  - 语义：`Client::builder()` 默认读 `HTTPS_PROXY`/`HTTP_PROXY` 环境变量；`.no_proxy()` 让 client 不走任何环境代理（与 `NO_PROXY=127.*` 不被 reqwest 识别为合法 CIDR 的问题无关）
  - 范围：测试（`mock_response` 直连 127.0.0.1:0）+ 生产（自托管部署走本地后端 127.0.0.1）均适用

### 回滚伪修复（黑板判词指出的认知错误）

- [ ] `mcp-server/tests/c2_read_tools.rs`：还原 `payload.len()` → `payload.as_bytes().len()` 的伪修复
  - 还原方式：`git checkout HEAD -- mcp-server/tests/c2_read_tools.rs`
  - 理由：Rust `String::len()` 本就返回字节数，与 `as_bytes().len()` **恒等**——该改动语义 no-op，属认知错误（feat-table-resize 批次3 时误加）

### 验证项（cargo test 触发，非独立 verify 节点）

- [ ] **带代理**跑 `cd mcp-server && cargo test --test c2_read_tools` → 全绿（7 passed / 0 failed）
- [ ] **清代理**跑 `cd mcp-server && env -u HTTPS_PROXY -u HTTP_PROXY -u https_proxy -u http_proxy cargo test --test c2_read_tools` → 全绿（7 passed / 0 failed）
- [ ] 双跑结果一致（均 7 passed / 0 failed）→ 证明 `.no_proxy()` 修复生效

## [spec] 规格登记（无新增用例，仅 spec 确认）

- [ ] 无新增用例（`ut_mcp_05_and_st_mcp_02_get_and_export` 是既有用例，修复后自然通过）
- [ ] 确认 ledger 无变化（`logos/resources/test/core-S06-mcp-service-test-cases.md` 等已有 UT-MCP-05/06/ST-MCP-02..09 登记行，无需新增）

## [archive] 归档（留待外环下一条 steer 派发，verify/archive 属独立 CLI 节点不列入 tasks）

## 实现顺序建议

1. `mcp-server/src/api.rs:15` `Client::builder()` 加 `.no_proxy()`
2. `git checkout HEAD -- mcp-server/tests/c2_read_tools.rs` 还原伪修复
3. 带代理跑 `cargo test --test c2_read_tools` → 全绿
4. 清代理跑 `cargo test --test c2_read_tools` → 全绿
5. 双跑结果一致（均 7 passed / 0 failed）
6. 全量 `openlogos verify` → Gate 3.5 + 3.6 双 PASS
7. 外环复验后 archive

每步独立 commit，commit message 格式 `fix(<module>): ...`。

## 不在范围（明确排除）

- 不修改测试断言
- 不修改 `ToolError` 错误码语义
- 不修改 request/response 处理逻辑
- 不在 CI 环境做代理清理
- 不引入生产环境代理配置项