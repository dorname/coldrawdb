# 变更提案：release-v0-2-0

> module: core | created: 2026-09-20

## 变更原因
v0.1.0（2026-09-16 发布）至今累积 14 个提交，包含两个已闭环变更（mcp-canvas-tools、fix-remote-github-issues-7-18），新增功能与缺陷修复值得作为一个小版本对外发布：让远程仓库的 tag / Release 反映当前主线，版本号与产物一致。

## 变更类型
代码级（发布工程变更；仅三端 `Cargo.toml` 的 `version` 字段 0.1.0 → 0.2.0，无业务逻辑改动）

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（verify 已 PASS 467/467，本变更不动代码语义）

## 部署影响
- 是否需要部署：否（本次只 bump 版本号并打 tag / 建 GitHub Release；部署另有 release-deploy-bundle 流程）
- 部署原因：版本号对齐发布物
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（tag 可删除重建）
- 是否需要 smoke：否（无运行代码变更；本地服务可用性已在前序 issue 修复批次中验证）

## UI/UX 变更声明

```yaml
ui_impact: false            # 本次是否触及界面（GUI 项目才有意义）
design_system_mode: generated   # generated | fallback（fallback 时须填 design_system_fallback_reason）
design_system_fallback_reason: ""
pages: []                   # 每项 {id, prototype: core-NN-<slug>.html, description}
```

## 变更概述
1. `backend/Cargo.toml`、`frontend-rs/Cargo.toml`、`mcp-server/Cargo.toml` 的 `version` 由 `0.1.0` 提升至 `0.2.0`。
2. 打附注 tag `v0.2.0` 并推送；在 GitHub 创建 Release（基于 v0.1.0..v0.2.0 提交自动生成 release notes，突出 mcp-canvas-tools 与 issue #7~#18 修复）。
