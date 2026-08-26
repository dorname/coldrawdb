# 任务清单 — complete-skipped-e2e

> created: 2026-08-26 | status: pending

## Task 1 — Harness 骨架
- [ ] `frontend-rs/tests/e2e/package.json` 引入 `@playwright/test`
- [ ] `frontend-rs/tests/e2e/playwright.config.ts` 配置 Chromium headless、`baseURL=http://localhost:8081`、`--font-render-hinting=none`、`--force-device-scale-factor=1`
- [ ] `frontend-rs/tests/e2e/global-setup.ts` 启动本地 dev（复用 `scripts/start-local.sh`）
- [ ] `frontend-rs/tests/e2e/global-teardown.ts` 关闭本地 dev

## Task 2 — Reporter
- [ ] `frontend-rs/tests/e2e/reporter/openlogos.ts` 转换 playwright result 为 `verify/test-results.jsonl` 格式
- [ ] 字段映射：`{id, status, duration_ms, timestamp}`
- [ ] 写入路径：`logos/resources/verify/test-results.jsonl`（append）
- [ ] 单测 `reporter.test.ts`：playwright mock 结果 → 验证 jsonl 行格式

## Task 3 — 用例实现批次 A：S03（5）
- [ ] ST-FE-S03-01：register → 跳转 home
- [ ] ST-FE-S03-02：login → user-menu 显示用户名
- [ ] ST-FE-S03-03：refresh token → 401 → 自动重新登录
- [ ] ST-FE-S03-04：logout → token 失效 → 受保护路由跳转
- [ ] ST-FE-S03-05：未登录访问 /rooms → 跳 /login

## Task 4 — 用例实现批次 B：S04（6）
- [ ] ST-FE-S04-01：创建房间 → 跳转 editor
- [ ] ST-FE-S04-02：邀请成员 → 接受邀请 → 出现在房间
- [ ] ST-FE-S04-03：viewer 角色进入 → editor 只读
- [ ] ST-FE-S04-04：editor 角色被降级 → 工具栏失效
- [ ] ST-FE-S04-05：owner 删除房间 → 协作成员收到 disconnect
- [ ] ST-FE-S04-06：邀请链接过期 → 错误提示

## Task 5 — 用例实现批次 C：S05（6）
- [ ] ST-FE-S05-01：两个 tab 加入同一房间 → WS 握手
- [ ] ST-FE-S05-02：远端 op 应用 → 本地视图同步
- [ ] ST-FE-S05-03：presence 光标显示用户名
- [ ] ST-FE-S05-04：本地 op 与远端 op 冲突 → OT 解决
- [ ] ST-FE-S05-05：断网 5s → reconnect-banner → 重连后 sync
- [ ] ST-FE-S05-06：server-rev 落后 → 重新拉取整图

## Task 6 — 用例实现批次 D：V2 + PROTOTYPE + 其他（39）
- [ ] ST-FE-V2-01~04：跨场景全链路回归
- [ ] ST-FE-PROTO-01~08：与主原型像素相似度 ≥ 95%
- [ ] ST-CR-01：画布 resize
- [ ] ST-MM-01~03：模态
- [ ] ST-PC-01：导入导出抽屉
- [ ] ST-SP-01：侧栏折叠
- [ ] ST-UI-05：confirm modal
- [ ] UT-S01-02 / UT-S02-03~09 / ST-S01-03 / ST-S02-01~06：单元 + 集成（已有，覆盖）

## Task 7 — 集成
- [ ] `scripts/run-verify-tests.sh` 新增阶段 2：e2e harness
- [ ] `scripts/run-verify-tests.sh` 阶段 3 不变：cargo test
- [ ] `core-implementation-checklist.md` §7.5、§7.6 取消 "e2e harness 待接入" 标记
- [ ] `logos/resources/verify/test-results.jsonl` 历史 56 skip 保留，新数据追加

## Task 8 — 验收
- [ ] 跑 `openlogos verify` → 56 skip 转为 56 pass 或保留 skip（有合理依据的）
- [ ] 跑 `openlogos smoke` → SMOKE_PASS
- [ ] 写一份 `e2e-coverage-report.md` 列出每条用例的覆盖路径
- [ ] 走 `/openlogos:merge complete-skipped-e2e` → `/openlogos:archive complete-skipped-e2e`