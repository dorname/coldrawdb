# 变更提案：replace-upstream-assets

> module: core | created: 2026-09-11

## 变更原因

仓库中仍残留上游 drawDB 的品牌资产与配置，与「纯 Rust 自研」定位（见已归档提案 reposition-self-developed）矛盾：

1. README 头图 `drawdb.png` 为上游 drawDB 编辑器截图；`public/hero_ss.png` 为上游官网 hero 图（均为上游产物，非本应用界面）
2. `.github/workflows/build.yml` 触发分支仍含已删除的 `drawdb-web`；后端测试步骤注释声称存在「2 个既有失败」（实际本地 419/419 全过）；`docker.yml` 分支触发同样残留 `drawdb-web`
3. `.github/FUNDING.yml` 指向上游作者（`github: 1ilit`），与本仓库无关

## 变更类型

仓库资产级（README 资产替换 + CI 配置适配 + 上游残留清理）。非方法论规格变更：不涉及 `logos/resources/` 主文档、场景、API、DB、测试，无 delta 产出。

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 受影响文件：
  - `README.md` — 头图替换为本应用真实运行截图（后端单进程托管 SPA + Playwright 截取编辑器界面）
  - `.github/workflows/build.yml` — 分支触发改为 `main`；修正过时的后端测试注释；actions 版本升级（checkout/cache/setup-node v3 → v4）
  - `.github/workflows/docker.yml` — 分支触发 `drawdb-web` → `main`
  - `.github/FUNDING.yml` — 删除（上游作者赞助配置，无法适配）
  - `drawdb.png`、`public/hero_ss.png` — 删除（上游截图；public/ 目录为上游 React 时代残留且无其他内容）
  - `frontend-rs/scripts/capture-readme-screenshot.mjs` — 新增截图脚本（注册/登录 → 建图 → 写入演示模型 → Playwright 截图），供后续更新 README 头图复用

## 部署影响

- 是否需要部署：否
- 部署原因：文档资产与 CI 配置变更，无运行时行为变化
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

以本应用真实界面替换上游 drawDB 截图：构建前端 release 产物 → 后端以 `COLDRAWDB_STATIC_DIR` 单进程托管 SPA → Playwright 走真实注册/登录/建图/写入流程后截取编辑器（暗色主题）→ 存入 `docs/assets/` 并更新 README 引用；同步适配 `.github` CI（分支名/注释/版本），删除无法适配的上游赞助配置与全部上游截图资产。
