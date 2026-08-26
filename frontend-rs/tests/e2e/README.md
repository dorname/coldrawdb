# Playwright e2e Harness (coldrawdb V2)

change-20260826-1330-complete-skipped-e2e 占位骨架。

## 当前覆盖

| 文件 | 测试 ID 前缀 | 状态 |
|---|---|---|
| `specs/s03-auth.spec.ts` | `ST-FE-S03-01~05` | 占位 |
| `specs/s04-rooms.spec.ts` | `ST-FE-S04-01~06` | 占位 |
| `specs/s05-collab.spec.ts` | `ST-FE-S05-01~06` | 占位 |
| `specs/v2-regression.spec.ts` | `ST-FE-V2-01~04` | 占位 |

共 **21 个占位测试**；test title 形如 `ST-FE-S03-NN: ...`，由
`reporter/openlogos.ts` 提取前缀写入 `logos/resources/verify/test-results.jsonl`。

## 跑通步骤

```bash
cd frontend-rs/tests/e2e
npm install
npx playwright install chromium
cd ../../..   # 回 repo 根
cd frontend-rs/tests/e2e
npx playwright test --reporter=list
```

`webServer` 配置会调 `scripts/start-local.sh` 起 backend + trunk serve。

## Reporter 输出

`reporter/openlogos.ts` 把 Playwright 结果追加到
`logos/resources/verify/test-results.jsonl`，与 backend `tests/verify_bootstrap.rs`
和 frontend `tests/openlogos_reporter.rs` 协作。

## 后续待办

- [ ] 每个 spec 写真实登录/创建房间/WS 握手流程
- [ ] 配套 `frontend-rs/tests/e2e/setup/auth.ts`（testid 库 + 共享 fixture）
- [ ] ST-FE-PROTO-* 像素相似度基线（待与 `core-01-editor-prototype.html` 对齐）
- [ ] CI 集成：在 `scripts/run-verify-tests.sh` 阶段 2 调 e2e harness