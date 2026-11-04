# 合并指令

## 变更提案
- 提案名称：fix-trash-restore-refresh
- 提案目录：logos/changes/fix-trash-restore-refresh/

## 提案内容

# 变更提案：fix-trash-restore-refresh

> module: core | created: 2026-11-04 | 关联 issue: #75

## 变更原因

GitHub issue #75：回收站「恢复房间」成功后主列表不刷新，已恢复房间在列表中消失。
后端完全正确（DB `archived_at=None`、`GET /api/v1/rooms` total=3 且含恢复房间），
前端 `RoomsListPage.restore_room_action` 成功分支只做了两件事：从回收站列表移除
条目（trash_items.retain）+ Toast 通知——**从不触发主房间列表重新拉取**。用户手动
返回列表后 rooms 信号仍是旧数据，已恢复房间不可见。

e2e 未能抓住该 bug 的原因：ST-S04-UI-14 的 installApi mock 用静态 `roomsList`
（恢复前后同一份、恒含 room-arch），「返回列表后房间重现」断言无论前端是否重拉
都通过，掩盖了真实时序。

## 变更类型

代码级变更（前端 UI 刷新时序修复 + 对应 e2e mock 时序化增强；API 契约、后端、
数据结构零变化）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（无规格文档变更；测试用例断言增强登记于测试用例文档 delta）
- 影响的业务场景：S04（房间列表/回收站；恢复成功后的列表同步时序）
- 影响的 API：无（upstream 零变化）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-S04-test-cases.md` MODIFIED
  ST-S04-UI-14（恢复后主列表必须重拉重现，mock 时序化后断言真正依赖刷新）

## 部署影响

- 是否需要部署：否
- 部署原因：本地 trunk serve 开发态验证；无独立部署环境
- 影响环境：无（本地）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **产品修复**：`editor_panels.rs` `RoomsListPage.restore_room_action` 成功分支
   追加 `reload_rooms.update(|n| *n += 1)`——与「删除房间后强制刷新」（p0-fix
   定点 1）同一机制：列表加载 effect 检测 nonce 变化 → 清 token 缓存 → 重新
   GET /api/v1/rooms。同作用域信号，零参数/零结构改动。
2. **e2e mock 时序化**：`test-spec-parity-d.mjs` installApi 增加动态主列表模式
   （恢复后 GET /rooms 动态包含恢复的房间），ST-S04-UI-14 初始 roomsList 改为
   不含 room-arch（真实恢复前状态），使「返回列表后房间重现」断言真正依赖
   reload 刷新而非静态 mock。
3. **验证**：ST-S04-UI-14 单独复跑 ≥2 次 PASS；`openlogos verify` 全量 0 失败。


## 需要合并的 Delta 文件

### 1. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/fix-trash-restore-refresh/deltas/test/core-S04-test-cases.md`
- 目标目录：`logos/resources/test/`
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
   git add -A && git commit -m "docs(fix-trash-restore-refresh): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-trash-restore-refresh`。
