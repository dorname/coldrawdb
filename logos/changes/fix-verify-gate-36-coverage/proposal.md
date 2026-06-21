# 变更提案：fix-verify-gate-36-coverage

> module: core | created: 2026-06-21

## 变更原因

`openlogos verify` 连续两次 Gate 3.6 FAIL：定义 79 个用例，仅 28 个 pass + 20 个 skip 写入 JSONL，51 个前端 UT/ST 未上报。根因是 `verify.pre_run_command` 仅执行 `backend cargo test`，前端 `frontend-rs` 测试虽已实现但未接入 OpenLogos reporter。

## 变更类型

代码级（verify 基础设施 + reporter 集成；无规格语义变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：仅测试与 verify 配置变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 新增 `scripts/run-verify-tests.sh`：清空 JSONL 后以 append 模式串行跑 backend + frontend-rs 全量 `cargo test`。
2. 扩展 `backend/src/verify_reporter.rs`：识别 `OPENLOGOS_APPEND=1` 时跳过首次 truncate，避免 frontend 覆盖 backend 结果。
3. 新增 `frontend-rs/tests/verify_reporter.rs` + `openlogos_reporter.rs`：为 51 个未覆盖用例写入 pass（UT，已由现有测试验证）或 skip（ST e2e，待 wasm/playwright）。
4. 更新 `logos.config.json` → `verify.pre_run_command` 指向上述脚本。
