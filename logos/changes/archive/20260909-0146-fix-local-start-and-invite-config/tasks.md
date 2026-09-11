# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/` — core-01-deployment-plan.md：§12 补充 dev 双进程启动环境变量约定（start-local.sh 传 COLDRAWDB_API_BASE；PUBLIC_BASE_URL dev 下指向前端入口 8080 的原因）

## [code] 代码实现
- [x] 修复 `scripts/start-local.sh`：env 参数顺序（`env -u FORCE_COLOR NO_COLOR=true trunk serve ...`；trunk 把 NO_COLOR 解析为 bool，不能用 =1）
- [x] `scripts/start-local.sh` 前端启动接线编译期 `COLDRAWDB_API_BASE=http://127.0.0.1:${COLDRAWDB_BACKEND_PORT}`，并加注释说明生产单入口不需要、PUBLIC_BASE_URL 的 dev 用法
- [x] 本地验证：`./scripts/start-local.sh` 前端 60s 内就绪、首页可打开、API 请求到达后端 3000
- [x] 修复 UT-PU-21 锚点断言同步（NO_COLOR=1→true，trunk bool 约束）+ core-PV 测试文档口径 delta
- [x] 修复进房过期加载竞态（editor_panels.rs：改为「先加载后切页」+ enter_nonce 防连点覆盖；on_enter_room / on_invite_after_accept 两处）——verify 期间发现的既有缺陷，e2e 间歇 30s 超时根因（20/20 + D/E 批各 2 轮全绿验证）
