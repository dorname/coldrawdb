# 合并指令

## 变更提案
- 提案名称：compose-backend-port-internal-only
- 提案目录：logos/changes/compose-backend-port-internal-only/

## 提案内容

# 变更提案：compose 后端端口仅内部网络，消除双入口迷惑

## 变更原因

部署交付物（docker-compose.yml / compose.release.yml）将后端容器 `3000` 端口映射到宿主机 `3000:3000`，同时 nginx 门面入口在 `9080`。两个端口都能访问完全相同的页面与 API，使用者无法判断哪个是"正确入口"，已实际造成迷惑（2026-09-21 使用排查中暴露）。架构上 nginx 经 compose 内部网络（`coldrawdb:3000`）与后端通信，宿主映射并非功能必需，纯属历史沿袭（feat-docker-compose-deploy 时期无 nginx，compose-default-port-9080 加 nginx 后未清理）。

## 变更类型

部署级变更（部署方案 + smoke 用例 + 部署配置 + [deploy] 任务）

## 变更范围

- 影响的部署方案：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §4/§5 中 compose 两形态（开发/发布）的 `3000:3000` 宿主映射示例与端口说明
- 影响的 smoke 测试：`logos/resources/test/smoke/core-smoke-test-cases.md` — SMOKE-core-STABLE-01 增加「宿主机 3000 应无监听」断言（本次变更的直接验收点）
- 影响的代码/配置：
  - `docker-compose.yml` — 移除 `3000:3000` 宿主映射（容器内 3000 与内部网络保留）
  - `compose.release.yml` — 同上（发布交付物模板）
  - `README.md` — 健康检查段补限定语（compose 形态仅 9080；3000 仅适用于单容器 docker run / 开发模式）
- 影响的业务场景：无（纯部署形态）
- 影响的 API / DB 表 / 编排测试：无

## 部署影响

- 是否需要部署：是
- 部署原因：compose 配置变更需重建本地栈方可生效；同时验收「9080 门面不受影响、宿主 3000 不再监听」
- 影响环境：本地 / 测试
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（恢复一行端口映射即可，低风险）
- 是否需要 smoke：是（STABLE-01 断言更新后重新冒烟）

## 变更概述

compose 两形态（开发/发布）移除后端容器的 `3000:3000` 宿主机映射，后端端口仅保留在 compose 内部网络供 nginx 反代使用，对外唯一入口收敛为 nginx 9080。部署方案同步更新端口拓扑说明与 compose 示例；STABLE-01 冒烟增加「宿主机 3000 无监听」断言，防止映射回潮。调试期如需直连后端，可用 `docker compose run --service-ports coldrawdb` 或临时 `docker run -p 3000:3000` 补映射，不作为交付形态的一部分。


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/compose-backend-port-internal-only/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/compose-backend-port-internal-only/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(compose-backend-port-internal-only): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive compose-backend-port-internal-only`。
