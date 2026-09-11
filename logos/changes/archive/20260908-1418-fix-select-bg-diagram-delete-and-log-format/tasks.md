# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-S04-room-lifecycle-design.md：创建房间模态删除图表入口 + queryAll 过滤口径
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — core-09-core-components.md：select 背景禁用 background 简写条款
- [x] 产出 delta 文件到 `deltas/test/` — core-S04-test-cases.md（UT-S04-15 / UT-S04-UI-16 / ST-S04-UI-16）+ core-PC-import-export-test-cases.md（UT-PC-23）+ core-PV-verify-pre-run-test-cases.md（UT-PU-21）

## [code] 代码实现

批次一：select 背景平铺修复（覆盖 UT-PC-23）
- [x] styles.css：两处 `background:` 简写改 `background-color:`；暗色 chevron 规则补 no-repeat/position
- [x] UT-PC-23 CSS 锚点

批次二：关联图表删除（覆盖 UT-S04-15 / UT-S04-UI-16 / ST-S04-UI-16）
- [x] 后端：query_all_diagrams 过滤 is_deleted=0 + UT-S04-15
- [x] 前端：创建房间模态「删除此图表」按钮 + 二次确认 + 刷新列表 + UT-S04-UI-16 锚点
- [x] e2e：ST-S04-UI-16（mock queryAll/DELETE，选中→删除→列表刷新→回退新建）

批次三：日志格式（覆盖 UT-PU-21）
- [x] main.rs init_log：去 pretty、with_ansi(false)、默认 filter 降 sqlx::query 至 warn
- [x] start-local.sh：trunk 启动 env NO_COLOR=1
- [x] UT-PU-21 锚点（后端源码 + 脚本）

每批均含：业务代码 + UT/ST 测试代码 + 写入 `logos/resources/verify/test-results.jsonl` 的 OpenLogos reporter
