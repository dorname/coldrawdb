# 变更提案：remove-debug-smoke-artifact

> module: core | created: 2026-06-13

## 变更原因

在 `add-local-run-scripts` 提案的合并阶段，`git add -A` 不慎将工作区中的 `frontend-rs/scripts/debug-smoke.mjs` 一并提交（首次出现在 commit `43097e7`，随后在 `0ee398b` 归档提交中再次被带入）。该文件是作者在排查 e2e 失败时使用的临时调试脚本，不属于本仓库的功能文件。为保持仓库历史清洁，需要从 git 索引中移除该文件并通过一次显式变更完成清理。

## 变更类型

代码级修复

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的源文件：`frontend-rs/scripts/debug-smoke.mjs`（从 git 索引移除，工作区保留）

## 部署影响

- 是否需要部署：否
- 部署原因：仅清理误提交文件，不影响运行
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

本提案将 `frontend-rs/scripts/debug-smoke.mjs` 从 git 索引中移除（`git rm --cached`），工作区中的文件本体保留，开发者可继续本地使用。同时在 `.gitignore` 中加入该路径，防止后续再次被误提交。此次仅产生一个代码提交，无规格 delta。
