# 变更提案：下拉背景平铺修复 + 关联图表删除 + 日志格式友好化

> module: core | created: 2026-09-08

## 变更原因

用户目标（2026-09-08，三项，附截图）：

1. **创建空间时下拉出现混乱背景字符**（截图：创建协作房间模态中「关联 diagram / 数据库引擎 / 默认邀请角色」三个 select 满框平铺 `˅˅˅˅…`）。根因：`styles.css:1733` `.cdb-create-room-modal .cdb-input` 与 `:1417` `.cdb-auth-card .cdb-input` 使用 `background:` **简写**（优先级 0,2,0），把 `background-image`/`background-repeat` 重置为 `none`/`repeat`；暗色规则 `[data-mode="dark"] .cdb-select`（:2904，同为 0,2,0 且靠后）只重设了 chevron `background-image` 而未重设 `background-repeat: no-repeat` → 暗色模式下 chevron 被平铺成满框箭头字符。这是上一轮下拉视觉重做（UT-PC-19）的级联遗漏。
2. **关联图表无法删除**（截图：关联 diagram 下拉里堆积数十个「Untitled Diagram / 新图」孤儿图）。两个缺陷叠加：(a) 遗留端点 `GET /diagrams/queryAll`（diagrams/mod.rs:23）`Diagram::find().all()` **不过滤 `is_deleted`**，软删的图仍出现在选项里；(b) 前端没有任何删除孤儿图的入口——后端 `DELETE /api/v1/diagrams/{id}`（软删）早已存在且有 client 封装，但 UI 未接线。
3. **前后端日志格式不友好**（截图：日志文件里全是 `ESC[0m ESC[32m` ANSI 转义 + sqlx 每条 SQL 多行全量打印）。根因：(a) 后端 `init_log()`（main.rs:73）同时链 `.compact().pretty()`（后者生效，多行格式）且未 `.with_ansi(false)`，默认 filter `info` 把 `sqlx::query` 的逐条 SQL 全量 INFO 输出；(b) `scripts/start-local.sh:83` 用 `env -u NO_COLOR -u FORCE_COLOR trunk serve` **主动清除** NO_COLOR，trunk 输出带 ANSI 落盘 `logs/frontend.log`。

## 变更类型

代码级修复（CSS 级联 + 遗留端点过滤 + 前端删除入口接线 + 日志初始化/启动脚本）；无 DB 结构变更、无部署变更

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `core-S04-room-lifecycle-design.md` — 创建房间模态：关联 diagram 选择器旁新增「删除选中图表」入口（仅选中既有图时可用，二次确认）；queryAll 过滤 is_deleted 口径
  - `core-09-core-components.md` — select 视觉条款补充：禁止以 `background` 简写覆盖 select 背景（会破坏 chevron/no-repeat），须用 `background-color`
- 影响的业务场景：S04（房间创建）
- 影响的部署方案：无
- 影响的 API：无契约变更（`GET /diagrams/queryAll` 行为修正：排除已软删；`DELETE /api/v1/diagrams/{id}` 既有端点接 UI）
- 影响的 DB 表：无结构变更
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的测试用例文档：
  - `core-S04-test-cases.md` — UT-S04-15（queryAll 过滤软删）/ UT-S04-UI-16（删除入口锚点）/ ST-S04-UI-16（e2e 删除流程）
  - `core-PC-import-export-test-cases.md` — UT-PC-23（select 背景简写修复 CSS 锚点）
  - `core-PV-verify-pre-run-test-cases.md` — UT-PU-21（日志格式锚点：with_ansi(false)、无 pretty、sqlx::query 降噪、NO_COLOR=1）

## 部署影响

- 是否需要部署：否
- 部署原因：开发阶段缺陷修复，本地重启即生效；无新部署物、无配置变更、无数据迁移
- 影响环境：无
- 是否涉及数据迁移：否（存量软删图仅不再出现在选项中）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**修复 1：select 背景平铺。** 把 `.cdb-create-room-modal .cdb-input` 与 `.cdb-auth-card .cdb-input` 中的 `background:` 简写改为 `background-color:`（两处都只意图改背景色），消除对 chevron `background-image`/`background-repeat` 的高优先级重置；暗色 chevron 规则同步补 `background-repeat: no-repeat; background-position` 作为双保险。

**修复 2：关联图表删除。** 后端 `query_all_diagrams` 增加 `is_deleted=0` 过滤（软删图不再出现在选项）。前端创建房间模态在「关联 diagram」选择器下方、选中既有图（非 `__new__`）时显示「删除此图表」按钮 + 二次确认模态（文案明确「进入回收站不可恢复/画布内容将删除」按软删实际语义），调既有 `DELETE /api/v1/diagrams/{id}`，成功后刷新选项列表并回退选中态到「新建空白模型」。

**修复 3：日志格式。** 后端 `init_log()`：去掉 `.pretty()`（保留 `.compact()` 单行）、加 `.with_ansi(false)`、默认 filter 改为 `info,sqlx::query=warn`（RUST_LOG 可覆盖恢复原行为）。启动脚本 `start-local.sh`：trunk 启动改为 `env NO_COLOR=1`（不再主动清除），前端落盘日志无 ANSI。
