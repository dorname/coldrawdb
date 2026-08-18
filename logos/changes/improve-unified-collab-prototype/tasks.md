# 实现任务：improve-unified-collab-prototype

> 本变更只产出设计 delta 与单文件静态原型，不修改生产 Rust 源码。

## [delta] 规格变更（先完成设计）

- [x] D1 在 `core-00-information-architecture.md` 增加唯一主原型入口、单文件约束和 S01～S05 页面状态流
- [x] D2 更新 `core-S03-user-auth-design.md` 的原型入口与单文件鉴权演示说明
- [x] D3 更新 `core-S04-room-lifecycle-design.md` 的原型入口与跨视图房间演示说明
- [x] D4 更新 `core-S05-ot-collab-design.md` 的原型入口与协作模拟器说明
- [x] D5 在 `deltas/test/core-PU-unified-prototype-test-cases.md` 建立 PU-AC-01～PU-AC-08 原型验收矩阵，覆盖 ST-PU-01～ST-PU-18，明确交互步骤、预期结果和诊断锚点

## [prototype] 单文件原型实现

- [x] P1 建立内联设计 token、Light/Dark 主题、玻璃态背景、响应式栅格、动效与 reduced-motion 降级
- [x] P2 建立登录/注册/会话续期/退出以及房间列表/邀请接受的单文件视图状态机
- [x] P3 完成 AppBar、ToolRail、Canvas、Inspector、StatusBar、Popover、SideSheet、Modal、Toast 的统一编辑器外壳
- [x] P4 完成表/字段创建编辑、拖拽、删除、关系连线、搜索、撤销/重做、保存 revision 的交互闭环
- [x] P5 完成导入/导出、代码视图、分享、设置、主题切换和命令面板
- [x] P6 完成房间创建、成员邀请、角色修改、成员移除与 Owner/Editor/Viewer 权限矩阵
- [x] P7 完成 presence、远端光标/选区、远端操作、Activity、OT revision、断线排队、重连和失败降级状态机
- [x] P8 补齐 loading/empty/error/disabled/focus 状态、键盘行为、窄屏行为和内置诊断入口

## [code] 代码实现

## [verify] 验证

- [x] V1 静态检查：确认 HTML 无外部 `<link>`、`<script src>`、远程资源引用或共享 CSS 依赖（邀请/分享文本 URL 除外）
- [x] V2 DOM 检查：确认 S01～S05 关键 `data-testid` 与对话框语义均存在且无重复主锚点
- [x] V3 浏览器主链：登录 → 房间 → 编辑 → 邀请 → 远端操作 → 断线重连 → 导出
- [x] V4 浏览器编辑链：新增/拖拽/编辑/关系 → 撤销/重做 → 自动保存/revision
- [x] V5 浏览器权限链：Owner、Editor、Viewer 三角色下按钮、表单和 WS 操作符合权限矩阵
- [x] V6 浏览器视觉检查：Light/Dark、桌面/窄屏、玻璃态对比度、浮层遮挡和 reduced-motion
- [x] V7 运行原型内置诊断并记录 PU-AC-01～PU-AC-08 结果
- [x] V8 从磁盘读回所有 Markdown/HTML delta 修改片段，向用户展示实际原文

## 人类确认点

- [x] H1 用户确认本提案后，才开始产出 delta 与原型
- [x] H2 delta 完成后，等待用户明确授权 `openlogos merge improve-unified-collab-prototype`
- [x] H3 merge 后按流程提交规格文档；本变更无生产代码实现、部署或 smoke
- [ ] H4 verify 通过后，等待用户明确授权 archive；archive 后再询问是否 git push
