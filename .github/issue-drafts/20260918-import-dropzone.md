# [BUG] 导入功能拖放区域无效

## Describe the bug

导入抽屉（Import）中的拖放区域（`io-dropzone`）在界面上可见，但拖入 `.sql` / `.dbml` / `.json` 文件后无任何响应：内容不会填入下方文本框，解析摘要也不更新。

## To Reproduce

1. 打开编辑器，打开「导入」抽屉
2. 保持 SQL（或 DBML / JSON）Tab
3. 将本地 `.sql`（或对应格式）文件拖放到虚线框「拖放 .sql / .dbml / .json 或粘贴下方」区域
4. 观察：文件未被读取，文本框仍为空，解析摘要仍为「0 条语句」

## Expected behavior

拖放文件后应读取文件内容并填入下方粘贴区，触发本地解析摘要更新；随后用户可点击「导入到画布」。

## Screenshots

见导入抽屉截图（红色框标出拖放区）。

## Desktop

- OS: Windows / WSL2
- Browser: Cursor 内嵌 / Chromium 系（待补充）

## Additional context

代码侧：`frontend-rs/src/editor_panels.rs` 中 `cdb-io-dropzone` 目前仅为展示用 `<div>`，未见 `dragover` / `drop` 事件处理，也未见关联的隐藏 `file` input。粘贴文本框本身可工作。

建议修复：
1. 为 dropzone 绑定 `dragenter` / `dragover` / `dragleave` / `drop`
2. 读取 `DataTransfer.files` 文本内容写入 `content`
3. 可选：点击 dropzone 打开文件选择器作为兜底
