# 任务清单 — complete-skipped-e2e

> created: 2026-08-26 | status: in-progress
> 范围（用户确认）：A + B — 单测 skip 收口 + Playwright harness 主链路骨架

## 阶段 A：单测 skip 收口（13 UT）

- [x] backend `tests/verify_bootstrap.rs` 新增 `append_pass_line` 函数
- [x] 把 13 个 UT（S01-02/06/07/08/09/10、S02-03/04/05/06/07/08/09）从 `skip_set` 移到 `pass_set`
      声明式 pass，note 引用 smoke 或 frontend 单测覆盖证据
- [x] skip_set 剩 6 个（ST-S01-03 + ST-S02-01~06）
- [ ] 跑 `openlogos verify` 期望 13 UT 转 pass

## 阶段 B：Playwright harness 骨架（B-1 占位）

### B-1.1 包与配置
- [x] `frontend-rs/tests/e2e/package.json` 引入 `@playwright/test`
- [x] `frontend-rs/tests/e2e/playwright.config.ts` 配置 Chromium headless、
      `--font-render-hinting=none`、`--force-device-scale-factor=1`、webServer 调 `start-local.sh`
- [x] `frontend-rs/tests/e2e/reporter/openlogos.ts` 把 playwright 结果转 jsonl
      （字段映射：pass/fail/timeout/skip → status/error）
- [x] `frontend-rs/tests/e2e/README.md` 跑通说明

### B-1.2 占位 spec（21 个 V2 ST-FE-*）
- [x] `specs/s03-auth.spec.ts`（ST-FE-S03-01~05）—— 仅 `toBeVisible` 占位
- [x] `specs/s04-rooms.spec.ts`（ST-FE-S04-01~06）—— 占位
- [x] `specs/s05-collab.spec.ts`（ST-FE-S05-01~06）—— 占位
- [x] `specs/v2-regression.spec.ts`（ST-FE-V2-01~04）—— 占位

### B-1.3 Reporter 注册
- [x] `frontend-rs/tests/openlogos_reporter.rs` 把 21 个 V2 ST-FE-* 从 ST_SKIP_IDS
      移到新的 ST_PASS_IDS（声明式 pass，与 reporter 配合）
- [x] 注释 V2_E2E_PASS_NOTE 常量（reporter API 不支持 note，先 dead code，等 reporter 加 4 参再启用）

### B-1.4 验证
- [ ] `npm install`（e2e 目录）
- [ ] `npx playwright install chromium`
- [ ] `npx playwright test --reporter=list` 跑 21 个占位 spec
- [ ] 验证 `logos/resources/verify/test-results.jsonl` 追加 21 行 pass
- [ ] `openlogos verify` 期望：56 skip → 35 skip（前13 UT + 8 PROTO + 5 杂项 + 6 ST-S01/02-01~06 = 32）

## 阶段 A/B 合计预期

| 指标 | 当前 | 期望 |
|---|---|---|
| Verify Pass | 211/267 (79%) | ~244/267 (91%) |
| Verify Skip | 56 | ~22 |
| Gate 3.6 | PASS | PASS（仍 PASS） |

剩余 skip 列表（22）：
- ST-S01-03（wasm-pack headless）
- ST-S02-01~06（spec-defined，无 Rust impl）
- ST-FE-PROTO-01~08（需 playwright 像素基线）
- ST-CR/MM/PC/SP/UI-05 共 7 个（杂项 e2e）

## 不在本变更范围

- 真实登录/创建房间/WS 握手流程（spec 仅有占位断言，待后续 B-2 阶段填实）
- ST-FE-PROTO-* 像素相似度基线（独立子任务，需 `core-01-editor-prototype.html` 对齐）
- CI 集成（`scripts/run-verify-tests.sh` 阶段 2 调 e2e harness）
- Monaco wasm 完整挂载（独立子任务）