# Delta — core-01-deployment-plan.md（fix-issues-38-41-canvas-interaction）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#41

## ADDED — migration 0012_area_lock 部署注记

- `backend/migrations/0012_area_lock.up.sql` 随后端镜像打包（`COPY backend/migrations /app/migrations` 既有通路），启动时自动执行，无需手工 SQL。
- 迁移内容：`area` 表加 `locked INTEGER NOT NULL DEFAULT 0`；既有行自动为未锁定，幂等可重入。
- 部署顺序无新要求：后端先起来完成 migration，前端随后即可（前端对旧后端容忍——locked 缺省 false）。
- 回滚：移除新二进制即可；`down.sql` 提供 DROP COLUMN 配对，常规回滚不需要执行（旧代码 serde default 兼容缺列读取方向）。
- 部署后 smoke 增补：SMOKE 区域锁定持久化用例（见 deltas/test/smoke/）。
