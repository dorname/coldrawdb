# Delta: core-00-information-architecture.md

> 提案：fix-overflow-menu-room-delete-listview-io

## MODIFIED — 功能树（房间区）

> 锚点：替换「[房间与最近项目]」功能树代码块。

```text
[登录/注册]
     │ 登录成功 / 演示进入
     ▼
[房间与最近项目]
     ├── 创建房间 ───────────────┐
     ├── 打开已有房间 ───────────┤
     ├── 删除房间（owner，卡片入口）│  ← fix-overflow-menu-room-delete-listview-io 新增
     └── 接受邀请 ───────────────┤
                                  ▼
                        [协作 ER 编辑器]
                         ├── 编辑/关系/撤销/保存
                         ├── 导入/导出/代码/分享（列表视图同入口可用）
                         ├── 成员/角色/邀请
                         └── OT/presence/重连模拟
```

## ADDED — IA 注记（本提案入口）

- 房间卡片删除入口（owner 可见）：`room-card-delete-{id}` + 确认模态 `modal-delete-room-list`，语义见 `core-S04-room-lifecycle-design.md` §2.4 补充。
- ListView 工具条 IO 入口：`list-btn-import` / `list-btn-export`，与画布态 IO 抽屉同一面板，见 `core-04-side-panel-tabs.md` §10.5 与 `core-01d-import-export.md` §4.5。
- AppBar 溢出菜单视口边界保护：小视口下菜单体内滚动不裁切，见 `core-05-top-menu-modals.md` §1.1 补充。
