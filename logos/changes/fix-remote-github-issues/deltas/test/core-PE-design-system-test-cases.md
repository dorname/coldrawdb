# Delta — core-PE-design-system-test-cases.md（#6 主题持久化落地）

> 变更：fix-remote-github-issues / GitHub #6  
> 说明：UT-E5-02 / core-0b §6 已规定 `localStorage["cdb-mode"]`；生产 `btn-theme-toggle` 当前只改 `data-mode` 未持久化。本 delta 把「必须落地」写进验收合同，并补刷新回归。

## MODIFIED — UT-E5-02 — 持久化（强制生产接线）

### UT-E5-02 — 主题写入 `localStorage["cdb-mode"]`

- **位置**：`frontend-rs/src/editor_panels.rs`（`btn-theme-toggle` 与启动恢复）
- **合同（不变键名）**：`localStorage["cdb-mode"]` ∈ `"light" | "dark" | "system"`
- **步骤 / 断言**：
  1. 点击切换到浅色 → `document.documentElement.dataset.mode === "light"`
  2. `localStorage.getItem("cdb-mode") === "light"`
  3. 再切回深色 → `cdb-mode === "dark"` 且 `data-mode="dark"`
- **实现缺口关闭**：生产路径不得仅改 DOM attribute；必须与 core-0b §6 同步写读

## ADDED — UT-E5-06 — 启动时从 localStorage 恢复主题

### UT-E5-06 — 刷新后主题保持

- **位置**：`read_html_data_mode` / AppRoot 初始化（或等价 boot 函数）
- **前置**：`localStorage["cdb-mode"]="light"`；`index.html` 默认仍可为 `data-mode="dark"`
- **步骤**：模拟冷启动读取
- **断言**：
  1. 启动后 `data-mode="light"`（localStorage 优先于 html 默认）
  2. `theme_mode` signal == `"light"`
  3. 缺省 / 非法值时回落规格默认（与 core-0b 一致；显式 light/dark 优先于 system）

## MODIFIED — ST-PE-06 — 暗色模式切换含刷新保持

### ST-PE-06 — 暗色模式切换

- 既有：点击 `btn-theme-toggle` 切换 light↔dark
- **ADDED**：切换到 light 后刷新（或重新 mount）→ 仍为 light；`cdb-mode` 仍为 `"light"`

## MODIFIED — 验收总览 / 附录（如有 ID 表）

登记 UT-E5-06；UT-E5-02 状态由「规格占位」改为「本提案必须实现」。

## ID 登记表（OpenLogos ledger）

> `scripts/validate-openlogos-ledger.mjs` 仅识别 `| UT-… |` / `| ST-… |` 表格行。

| ID | 层级 | 说明 |
|---|---|---|
| UT-E5-02 | UT | 主题切换写入 `localStorage["cdb-mode"]` |
| UT-E5-06 | UT | 冷启动从 `cdb-mode` 恢复 `data-mode` / theme_mode |
