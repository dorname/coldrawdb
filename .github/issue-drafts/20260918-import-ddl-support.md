# [BUG] 导入不支持上传 .ddl 文件

## Describe the bug

导入抽屉拖放区文案与文件选择仅覆盖 `.sql` / `.dbml` / `.json`，不支持常见的 `.ddl`（SQL DDL）扩展名。用户无法通过上传 `.ddl` 文件导入结构（即便内容本身是合法 SQL DDL）。

## To Reproduce

1. 打开编辑器「导入」抽屉，选 SQL Tab
2. 尝试拖放或选择一个 `.ddl` 文件
3. 观察：拖放无效 / 文件选择器过滤掉 `.ddl` / 或提示不支持该扩展名

## Expected behavior

`.ddl` 应作为 SQL 导入的合法扩展名被接受：
- dropzone 文案包含 `.ddl`
- 文件选择器 `accept` 包含 `.ddl`
- 按 SQL 路径解析（与 `.sql` 相同），引擎选择（generic/mysql/postgresql/sqlite）仍可用

## Screenshots

导入抽屉文案当前为：「拖放 .sql / .dbml / .json 或粘贴下方」——未列出 `.ddl`。

## Desktop

- OS: Windows / WSL2
- Browser: 待补充

## Additional context

相关位置：`frontend-rs/src/editor_panels.rs`（`cdb-io-dropzone` 文案；以及若后续补上 file input / drop handler 的扩展名白名单）。

后端若对 `source`/`filename` 做扩展名校验，需同步放行 `.ddl`（按 SQL DDL 处理）。

建议验收：
1. 拖放 `schema.ddl` → 内容进入文本框 → 解析摘要非 0（对有效 DDL）
2. 点击选择文件可选 `.ddl`
3. 「导入到画布」成功生成表/关系
