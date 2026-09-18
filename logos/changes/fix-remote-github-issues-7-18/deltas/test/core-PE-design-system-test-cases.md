# Delta — core-PE-design-system-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#13/#14（返回按钮样式）、#15（协作芯片截断） | 规格：`core-05-top-menu-modals.md` R-AB-01～09

## ADDED — UT-PE-AB-01 — 返回按钮 ghost 样式锚点

- **位置**：`frontend-rs/src/styles.css` 断言（`include_str!`）
- **断言**：
  - `.cdb-btn` 或 `.cdb-back-to-rooms` 含 `display: inline-flex` + `align-items: center`（图标文字同行居中）
  - `.cdb-back-to-rooms`（或其对 ghost 的覆盖）`border-color: transparent`；hover 态有独立底色规则
  - 可点击区域高度 ≥ 36px（min-height / height 规则存在）

## ADDED — UT-PE-AB-02 — room-badge 截断规则锚点

- **位置**：`frontend-rs/src/styles.css` + `editor_panels.rs` 锚点
- **断言**：
  - `.cdb-room-badge strong` 选择器存在且含 `overflow: hidden` + `text-overflow: ellipsis` + `white-space: nowrap`
  - `.cdb-room-badge` 保留 `max-width`；AppBar 中部容器含 `min-width: 0`
  - 生产 DOM 中 room-badge 按钮带 `title`（全文兜底）；`diagram-title` 带 `title`

## ADDED — ST-PE-07 — e2e：长房间名截断与邀请按钮可达

- **GIVEN**：加入名称 ≥24 字符的房间，进入其编辑器
- **WHEN**：观察顶栏中部；随后将视口宽度降至 1280
- **THEN**：room-badge 省略号截断且 `title` 为完整房间名；图名不溢出；`btn-invite` 始终可见可点；`btn-back-to-rooms` 图标与「空间」文字同行无堆叠
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S04"`）

## MODIFIED — ID 登记表（OpenLogos ledger）

> `scripts/validate-openlogos-ledger.mjs` 仅识别 `| UT-… |` / `| ST-… |` 表格行。

| ID | 层级 | 说明 |
|---|---|---|
| UT-E5-02 | UT | 主题切换写入 `localStorage["cdb-mode"]` |
| UT-E5-06 | UT | 冷启动从 `cdb-mode` 恢复 `data-mode` / theme_mode |
| UT-PE-AB-01 | UT | 返回按钮 ghost 透明边框 + inline-flex 锚点 |
| UT-PE-AB-02 | UT | room-badge 文本节点截断规则锚点 |
| ST-PE-07 | ST | 长房间名截断 + title 全文 + 邀请可达 |
