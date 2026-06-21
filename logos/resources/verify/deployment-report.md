# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `fix-diagram-entity-persistence` |
| 部署时间 | 2026-06-21 |
| 目标环境 | 本地开发 |
| 状态 | 成功 |

## 执行摘要

1. `./scripts/start-local.sh` — 启动后端（`:3000`）与前端 trunk serve（`:8080`）
2. 健康检查：`http://127.0.0.1:3000/` 与 `http://127.0.0.1:8080/` 均就绪
3. `openlogos smoke fix-diagram-entity-persistence` — 6/6 通过（Gate 3.8 PASS）

## 迁移

无数据迁移（SQLite schema 未变更）。

## 回滚点

- 停止服务：`./scripts/stop-local.sh`
- 代码回滚：revert 本变更相关 commit

## 访问地址

- 前端：http://127.0.0.1:8080/editor
- 后端：http://127.0.0.1:3000/

## 风险

无未解决风险。
