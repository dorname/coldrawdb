# Delta — core-PU-unified-prototype-test-cases.md（Splitter 分隔条用例）

> 提案：fix-canvas-zoom-invite-comment-resize（问题4：布局面板可拖动调整）
> 目标主文档：`logos/resources/test/core-PU-unified-prototype-test-cases.md`
> 编号续编：全局最大 UT-PU-21、ST-PU-26，本 delta 自 UT-PU-22 / ST-PU-27 起编。

## ADDED — Splitter 分隔条用例

### UT 用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-PU-22 | 生产 `styles.css` 锚点 + splitter 组件测试 | 断言 `--cdb-inspector-w` 初始 330px；模拟 pointermove 序列拖动 `splitter-inspector` | 拖动期间 CSS 变量即时更新（无 160ms 过渡，`is-resizing` 类禁用 grid 过渡）；钳制：拖过 520 / 240 边界时变量停在边界值；`.cdb-main` 三列网格第 3 列随变量变化 |
| UT-PU-23 | localStorage 预置 `cdb.inspector.width=400` | 初始化 splitter 组件后读取 `--cdb-inspector-w` | 变量为 400px；预置非法值（非数字 / 超出 240–520）时回落默认 330px；缺失 key 时 330px |
| UT-PU-24 | ListView 视图 + splitter-list-tree 组件测试 | 模拟拖动树列分隔条，断言 `.cdb-list-view-body` 内联 `grid-template-columns` | 第 1 列实时跟随 pointermove（直写内联 style，不触发 Leptos 重渲染）；钳制 160–420px；`pointerup` 后写入 `cdb.list-tree.width` |

### ST 用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-PU-27 | ≥1024px 视口已进入 room-editor | 在 `splitter-inspector` 上 pointerdown 向右拖动 400px 后松开 | Inspector 实际像素宽随拖动手感实时变化；拖过 520 / 240 边界时停在边界；松手后 Inspector 宽为最终钳制值；`data-testid="splitter-inspector"` 存在 |
| ST-PU-28 | 1000px 视口（或 720px 视口） | 进入 room-editor 与 ListView，查询两条分隔条 | `splitter-inspector` 与 `splitter-list-tree` 均不可见（`display:none` / 不在 DOM 可见层）；Inspector 仍按既有覆盖层/抽屉行为可用 |
| ST-PU-29 | ≥1024px 视口、已有 users→posts 关系 | 拖动 `splitter-inspector` 收窄第 3 列，在拖动中读取画布区尺寸与关系 path | 每次 pointermove 后画布容器 `clientWidth` 随网格重排即时缩小（RAF 尺寸比对生效）；`path[data-relation]` 的 `d` 随画布重排更新；全程 `#app` 不重建（ST-PU-19 不变量保持） |
| ST-PU-30 | ≥1024px 视口、ListView 视图已有表 | 拖动 `splitter-list-tree` 增宽树列后松开 | 树列实时跟随；钳制 160–420px；刷新页面后树列宽从 `cdb.list-tree.width` 恢复 |

### PU-AC 追溯补充

| 验收标准 | 覆盖用例 |
|---|---|
| PU-AC-03 编辑完整性（布局持久化） | UT-PU-22、UT-PU-23、ST-PU-27、ST-PU-30 |
| PU-AC-06 视觉质量（拖动跟手） | ST-PU-27、ST-PU-29 |
| PU-AC-10 主题与响应式（断点行为） | ST-PU-28 |
