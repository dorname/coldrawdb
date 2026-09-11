# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 更新 §12.4：`PUBLIC_BASE_URL` 默认值由"空"改为"脚本默认注入 `http://127.0.0.1:${COLDRAWDB_FRONTEND_PORT}`"，保留覆盖说明

## [code] 代码实现
- [x] `scripts/common.sh` 新增 `PUBLIC_BASE_URL` 默认值导出（置于 `COLDRAWDB_FRONTEND_PORT` 定义之后）
- [x] `scripts/tests/test-local-scripts.sh` 补充用例：默认值注入指向前端端口；显式设置 `PUBLIC_BASE_URL` 时覆盖优先
