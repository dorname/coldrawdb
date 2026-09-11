# 变更提案：fix-appbar-roomname-back-and-import-merge

> module: core | created: 2026-09-07

## 变更原因

用户实测反馈（2026-09-07，房间编辑器截图）三项问题：

1. **工作区名称缩略缺失**：AppBar 图表标题 / 房间徽章（room-badge）名称过长时无 `text-overflow: ellipsis` 缩略、无悬停全文提示，挤压顶栏布局。
2. **返回能力无感知**：room-badge 虽已绑定「返回房间列表」（`title="返回房间列表"`），但视觉上无返回箭头 / 无文字，用户发现不了；房间列表页才是「创建协作空间」入口。
3. **导入「没有成功」（核心 bug）**：ImportDrawer 提交走 `POST /api/v1/bridge/import/local`，后端**新建一个游离 diagram**（不属于任何房间）后前端 `window.location` 跳转独立编辑器 `/editor/{新id}`——用户被带离房间上下文，当前画布无任何变化，回房间也找不到导入结果。后端链路已实证正常（curl 验证表/字段/约束正确落库），坏在前端产品逻辑：导入语义应为 **drawdb 式合并进当前画布**（与主原型 `importModel` 行为一致——原型本就是本地合并）。

## 变更类型

设计级（交互行为 + 视觉规格变更，无 API / DB 契约变化）

## 变更范围

- 影响的需求文档：无（不改需求条目）
- 影响的功能规格：
  - `core-01d-import-export.md` §4.4（提交行为：bridge 新建游离 diagram + 跳转 → 本地解析合并进当前画布）
  - `core-S04-room-lifecycle-design.md`（AppBar 房间徽章：名称缩略 + 显性返回入口）
  - `core-00-information-architecture.md`（AppBar 布局：返回入口位置）
- 影响的原型：`core-01-editor-prototype.html`（AppBar 缩略/返回入口对齐；importModel 已是本地合并，无需改行为）
- 影响的业务场景：S01（编辑保存链路复用 schedule_save）、S04（房间生命周期 AppBar）
- 影响的 API：无（`POST /api/v1/bridge/import/local` 端点保留但前端导入抽屉不再调用；契约不动）
- 影响的 DB 表：无
- 影响的编排测试：无（非 API 项目编排）
- 影响的测试文档：`core-PC-import-export-test-cases.md`（ST-PC-01 改写 + 新增导入合并 UT）；`core-S04-room-lifecycle-test-cases.md` 或既有 AppBar 用例补充（名称缩略/返回入口断言）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端行为与样式变更，无 API/DB 契约变化，无新端点
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**导入合并（核心）**：ImportDrawer 在编辑器上下文（房间编辑器 + 独立编辑器）提交时，改为本地解析（`parse_sql_import_tables` 等既有纯函数）→ 解析出的表/关系**合并写入当前 store**（新表自动避让现有布局排布）→ 复用 `schedule_save` 既有保存链路落库 → 成功 toast「已导入 N 张表 / M 条关系」；解析失败走抽屉 inline error + ErrorToast。不再创建游离 diagram、不再跳转。bridge import 端点与导入日志面板保留（日志面板仅作历史记录展示）。

**名称缩略**：AppBar 图表标题 input 与 room-badge 名称加 `max-width` + `text-overflow: ellipsis` + `title` 悬停全文。

**返回入口显性化**：房间编辑器 AppBar 在 room-badge 左侧加「← 空间」返回按钮（`data-testid="btn-back-to-rooms"`），点击回房间列表页；room-badge 原有点击行为保留。
