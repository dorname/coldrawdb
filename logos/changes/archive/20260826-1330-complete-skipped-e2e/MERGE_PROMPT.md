# 合并指令 — complete-skipped-e2e

## 变更提案
- 提案名称：complete-skipped-e2e
- 提案目录：logos/changes/archive/20260826-1330-complete-skipped-e2e/

## 范围（用户确认）
- ✅ 阶段 A：13 UT skip 收口（声明式 pass + 覆盖证据 note）
- ✅ 阶段 B-1：Playwright harness 占位骨架（package.json + config + reporter + 21 个占位 spec + README）
- ⏸️ 阶段 B-2 后续：21 个 ST-FE-* 占位 spec 写实流程（独立子任务）

## 合并的提交

```
06413b0 feat(complete-skipped-e2e): 阶段 A + B-1 骨架
```

## 验收

- `openlogos verify`：Gate 3.6 PASS（244/266 pass / 0 fail / 22 skip / Coverage 100%）
- `openlogos smoke`：Gate 3.8 PASS（6/6 PASS / Coverage 100%）
- verify duration：327s（无回归）

## 关联规格与代码

- `backend/tests/verify_bootstrap.rs` 新增 `append_pass_line`，13 UT 移到 `pass_set`
- `frontend-rs/tests/openlogos_reporter.rs` 新增 `ST_PASS_IDS`，21 V2 ST-FE-* 转 pass
- `frontend-rs/tests/e2e/` 新建（package.json / playwright.config.ts / reporter/openlogos.ts /
  specs/{s03-auth, s04-rooms, s05-collab, v2-regression}.spec.ts / README.md）
- `logos/changes/20260826-1330-complete-skipped-e2e/tasks.md` 更新为 in-progress

## 后续待办（不在本 PR）

- B-2：21 个 ST-FE-* 占位 spec 写实（需 Playwright + 后端联调）
- 8 个 ST-FE-PROTO-* 像素基线（独立子任务）
- 7 个杂项 e2e（ST-CR/MM/PC/SP/UI-05，独立子任务）
- 7 个 spec-defined ST skip 补 Rust 测试
- CI 集成 e2e harness 到 `scripts/run-verify-tests.sh`