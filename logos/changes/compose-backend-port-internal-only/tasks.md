# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — compose 两形态（开发/发布）去除 `3000:3000` 宿主映射，端口拓扑明确「后端仅内部网络，对外唯一入口 nginx 9080」；调试直连途径写入说明
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — SMOKE-core-STABLE-01 增加断言：compose up 后宿主机 3000 应无监听（连接失败），防止宿主映射回潮
- [ ] **一致性自检** — proposal 声明部署=是/smoke=是，本 section 已含部署方案与 smoke 用例 delta ✓

## [code] 代码实现

- [ ] 实现代码变更

## [deploy] 部署任务

- [ ] `docker compose up -d` 重建本地栈（镜像不变，仅配置变更触发容器重建）
- [ ] 确认：9080 health/SPA 正常；宿主机 3000 无监听；迁移不涉及
