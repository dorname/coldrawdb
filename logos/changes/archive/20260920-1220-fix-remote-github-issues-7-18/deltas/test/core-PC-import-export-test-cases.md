# Delta — core-PC-import-export-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#7（拖放无效）、#8（.ddl 放行） | 规格：`core-01d-import-export.md` §4.3

## ADDED — UT-PC-31 — drop 读取文件文本写入导入内容

- **位置**：`frontend-rs/src/editor_panels.rs`（dropzone drop 处理；读取逻辑抽纯函数/可测闭包）
- **GIVEN**：ImportDrawer 打开，`sql` Tab
- **WHEN**：模拟 `DataTransfer.files[0]`（`.sql` 文件，内容含 2 条 CREATE TABLE）触发 drop
- **THEN**：导入内容 signal == 文件文本；解析摘要更新为非 0；`is-dragover` 态在 drop 后移除；`event.preventDefault` 已调用（浏览器不打开文件）

## ADDED — UT-PC-32 — 扩展名白名单与 Tab 映射

- **位置**：`frontend-rs/src/editor_panels.rs`（扩展名判定纯函数，如 `import_format_for_filename`）
- **断言**：
  - `"schema.sql"` / `"schema.ddl"` → `sql`；`"a.dbml"` → `dbml`；`"a.json"` → `json`（大小写不敏感）
  - `"a.txt"` / `"a.exe"` / 无扩展名 → `None`（拒绝，inline 提示文案含 `.ddl`）
  - dropzone 文案与 `io-file-input` 的 `accept` 属性均含 `.ddl`

## ADDED — UT-PC-33 — 点击 dropzone 打开文件选择器

- **位置**：`frontend-rs/src/editor_panels.rs` 锚点断言（`include_str!`）
- **断言**：存在 `data-testid="io-file-input"` 的隐藏 `<input type="file">`，`accept` 含 `.ddl`；dropzone 点击处理调用其 `click()`

## ADDED — ST-PC-09 — e2e：拖入 .ddl → 摘要 → 导入画布

- **GIVEN**：编辑器已加载；构造 `schema.ddl`（含 1 张带中文 COMMENT 的表）
- **WHEN**：向 `io-dropzone` 派发 dragover + drop（`DataTransfer` 注入文件）→ 点「导入到画布」
- **THEN**：textarea 已填充；摘要非 0；导入后画布新增该表；`Table.comment` 非空
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`scenario: "S01"`，`module: "core"`）
