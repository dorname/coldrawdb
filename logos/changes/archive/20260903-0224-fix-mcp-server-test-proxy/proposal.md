# 变更提案：fix-mcp-server-test-proxy

> module: core | created: 2026-09-02
> Guard: `logos/.openlogos-guard` 指向 `fix-mcp-server-test-proxy`
> 上游裁决：黑板条目6 外环(claude) 判词 2026-09-03 — feat-table-resize verify Gate 3.5 PASS 采认；Gate 3.6 FAIL 复验为 pre-existing 环境基础设施问题，解耦成立，插队于 D 案之前

## 变更原因

**pre-existing 测试基础设施缺陷**（feat-table-resize 提案 verify 阶段复验发现并解耦）：

1. 本机网络环境有 `HTTPS_PROXY=http://127.0.0.1:7897`（系统环境变量注入）
2. `mcp-server/tests/c2_read_tools.rs:147` `mock_response("200 OK", ...)` 启动 `TcpListener::bind("127.0.0.1:0")` 期望直连
3. reqwest 默认读 `HTTPS_PROXY`/`HTTP_PROXY` → 走代理 7897 → 代理 502 Bad Gateway（**不转发 127.0.0.1:0 端口**）
4. `ApiClient::new(config(base)).unwrap()` 拿到 502 而非 mock 返回 → `UPSTREAM_ERROR 上游返回非 JSON 响应`（`tests/c2_read_tools.rs:153` 断言失败）
5. reqwest 的 no_proxy 解析不支持 IP glob 语法（`NO_PROXY=127.*` 非合法 CIDR），代理仍介入
6. **机理四层完整证据链**（直接引用 `logos/changes/archive/20260903-0146-feat-table-resize/VERIFY_RUN.txt` 根因分析）：
   - 环境 → `HTTPS_PROXY=http://127.0.0.1:7897` 系统环境变量
   - 代码路径 → `mcp-server/src/api.rs:15` `Client::builder()` **未**调 `.no_proxy()`
   - 错误链 → reqwest 走代理 → 代理 502 → `UPSTREAM_ERROR 上游返回非 JSON 响应`
   - 修复路径 → `Client::builder()` 加 `.no_proxy()`（测试与生产均不应走环境代理连 127.0.0.1 mock/本地后端）

**影响**：feat-table-resize 提案 verify 阶段 Gate 3.6 FAIL（覆盖不完整——17 个 MCP 用例缺），被外环判词解耦为 pre-existing 环境问题，另开本案插队修复（否则后续每案 verify 都会复现此 FAIL，重复解释成本高于修复成本）。

**与本提案基线（`cc9919c`）零代码改动**：`git diff cc9919c..HEAD -- mcp-server/` 为空——本提案 commit 链对 mcp-server **零改动**（仅代码级修复路径，不动既有测试断言）。

## 变更类型

**代码级修复**（参考 `spec/tasks-spec.md` 与 `logos/skills/change-writer/SKILL.md` Step 3 判定）：
- 影响的 PRD/API/DB schema：**无**
- 影响的功能规格：**无**（仅测试基础设施修复）
- 影响的部署方案：**无**（纯客户端代码路径变更）
- 影响的 smoke：**无**

故 `tasks.md` 采用**代码级修复模板**（无 `[delta]`、`[deploy]` section）。

## 变更范围

- 影响的需求文档：**无**
- 影响的功能规格：**无**
- 影响的业务场景：**无**（`S06` MCP 场景的实现链路无业务语义变更；修复点在测试客户端代码路径）
- 影响的部署方案：**无**
- 影响的 API：**无**
- 影响的 DB 表：**无**
- 影响的编排测试：**无**
- 影响的 smoke 测试：**无**

**代码影响面**（`mcp-server/`）：
- `src/api.rs:15` `ApiClient::new()`：`Client::builder()` 加 `.no_proxy()` 禁用环境代理（测试与生产均不应走代理连本地后端）
- `tests/c2_read_tools.rs`：无代码改动（原 `payload.len()` 是 byte len，语义正确；**回滚**我在 feat-table-resize 期间加的伪修复 `payload.as_bytes().len()`——`String::len()` 本就返回字节数，与 `as_bytes().len()` 恒等，该改动是 no-op，认知错误已在黑板判词指出）

## 部署影响

- 是否需要部署：**否**
- 部署原因：纯测试客户端代码修复（`mcp-server` crate 内部），无对外部署节点；本地开发环境重新构建即生效
- 影响环境：**无**
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**（小切片，回滚 = revert commit）
- 是否需要 smoke：**否**

## 变更概述

给 `mcp-server/src/api.rs:15` `ApiClient::new()` 的 `Client::builder()` 加 `.no_proxy()` 调用，显式禁用 reqwest 环境代理。让 MCP 测试 `mock_response` 的 `TcpListener::bind("127.0.0.1:0")` 直连 mock 服务器，不再被系统环境变量 `HTTPS_PROXY=http://127.0.0.1:7897` 截断。测试与生产 MCP 客户端均不应走环境代理连 127.0.0.1 mock/本地后端（`NO_PROXY=127.*` 在 reqwest 不被识别为合法 CIDR，代理仍介入）。

## 设计决策记录（ADR-style 摘要）

| 决策 | 选 | 否 | 依据 |
|---|---|---|---|
| 修复点 | `Client::builder().no_proxy()`（api.rs:15）| 测试代码改断言 / CI 清环境变量 | 测试代码断言正确，无需改；CI 清环境变量治标不治本；`.no_proxy()` 显式且可测 |
| 测试改动 | 无 | 改 `ut_mcp_05` 断言 | 外环强制"禁止改测试断言" |
| `c2_read_tools.rs` 改动 | 回滚伪修复（`payload.len()` → `payload.as_bytes().len()` 是 no-op）| 保留 | 认知错误已在黑板判词指出——Rust `String::len()` 本就返回字节数，与 `as_bytes().len()` 恒等 |

## 范围外（明确排除）

- 不修改 `mcp-server/tests/c2_read_tools.rs` 测试断言（外环强制约束）
- 不修改 `mcp-server/src/api.rs` 的 `ToolError` 错误码语义（`UPSTREAM_ERROR`/`UPSTREAM_TIMEOUT`/`UPSTREAM_UNAVAILABLE` 等保持现状）
- 不修改 `mcp-server/src/api.rs` 的 request/response 处理逻辑（仅改 `Client::builder()` 链）
- 不在 CI 环境做代理清理（本地开发环境约束，CI 是否需清代理由后续专项评估）

## 风险点

- **R1**：`.no_proxy()` 可能影响生产环境用户对"需要代理才能访问后端"的场景——但本提案判定该场景不存在（MCP 客户端在自托管部署中走本地后端 127.0.0.1，无代理需求；如需代理未来再加 `with_proxy` 配置项）
- **R2**：reqwest `.no_proxy()` 在部分版本可能仅对 `Proxy::system()` 生效——实现时需验证当前 reqwest 版本（0.12.28）的 `.no_proxy()` 语义覆盖全代理类型
- **R3**：若 `.no_proxy()` 不能完全覆盖，备选方案是 `Client::builder().proxy(reqwest::Proxy::all().no_proxy(...))` 或显式 `.use_native_tls()` + 清除代理——需在实现时验证

## 替代方案否决理由

- **A 改测试断言**：外环强制禁止——测试断言正确，无需改
- **B CI 环境清代理**：治标不治本（本地开发环境仍需修复）
- **C 显式 `Proxy::all().no_proxy(...)`**：比 `.no_proxy()` 更繁琐，且在 reqwest 0.12 中等效——否决（用 `.no_proxy()` 最简）
- **D 在 `Config` 层加 `use_proxy: bool` 配置项**：过度设计，当前无生产场景需要代理——否决（YAGNI）

## 关联场景

- **S06（AI 客户端通过 MCP 管理数据库图表）**：MCP 客户端的 ApiClient 是 S06 链路的核心客户端，代理修复影响 MCP 测试的可靠性，不影响业务语义
- **feat-table-resize 提案 verify 阶段**：本案直接复验根因（同一 `HTTPS_PROXY` 环境变量导致同一 502）

## 关联任务清单

见 `tasks.md`。

## 不在范围（明确排除）

- 不修改 `ToolError` 错误码语义
- 不修改 request/response 处理逻辑
- 不在 CI 环境做代理清理
- 不引入生产环境代理配置项（YAGNI）